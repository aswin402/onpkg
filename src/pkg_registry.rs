use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

/// Package registry client for npm, PyPI, pub.dev, and cargo
pub struct PkgRegistry {
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct PkgInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub runtime: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

impl PkgRegistry {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("onpkg/0.1.0")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn fetch_info(&self, name: &str, runtime: &str) -> Result<PkgInfo> {
        match runtime {
            "npm" => self.fetch_npm(name).await,
            "pypi" | "pip" => self.fetch_pypi(name).await,
            "pub" | "dart" | "flutter" => self.fetch_pub(name).await,
            "cargo" | "rust" => self.fetch_cargo(name).await,
            _ => Err(anyhow!("Unknown runtime '{}'. Supported: npm, pypi, pub, cargo", runtime)),
        }
    }

    /// Add a package to a project's manifest file
    pub fn add_to_project(&self, name: &str, version: &str, runtime: &str, project_dir: &Path) -> Result<()> {
        match runtime {
            "npm" => add_to_npm_project(name, version, project_dir),
            "pypi" | "pip" => add_to_python_project(name, version, project_dir),
            "pub" | "dart" | "flutter" => add_to_flutter_project(name, version, project_dir),
            "cargo" | "rust" => add_to_cargo_project(name, version, project_dir),
            _ => Err(anyhow!("Unknown runtime '{}'", runtime)),
        }
    }

    async fn fetch_npm(&self, name: &str) -> Result<PkgInfo> {
        let url = format!("https://registry.npmjs.org/{}", name);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("npm package '{}' not found (HTTP {})", name, resp.status()));
        }
        let data: NpmResponse = resp.json().await?;
        let latest = data.dist_tags.get("latest").cloned().unwrap_or_default();
        let pkg = data.versions.get(&latest).cloned().unwrap_or_default();

        Ok(PkgInfo {
            name: name.to_string(),
            version: pkg.version.unwrap_or_else(|| latest.clone()),
            description: pkg.description.unwrap_or_default(),
            runtime: "npm".to_string(),
            homepage: pkg.homepage,
            repository: pkg.repository.map(|r| r.url.unwrap_or_default()),
            license: pkg.license,
        })
    }

    async fn fetch_pypi(&self, name: &str) -> Result<PkgInfo> {
        let url = format!("https://pypi.org/pypi/{}/json", name);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("PyPI package '{}' not found (HTTP {})", name, resp.status()));
        }
        let data: PyPIResponse = resp.json().await?;
        let info = data.info;

        Ok(PkgInfo {
            name: info.name,
            version: info.version,
            description: info.summary.unwrap_or_default(),
            runtime: "pypi".to_string(),
            homepage: info.home_page,
            repository: info.project_urls.and_then(|u| u.get("Source").cloned()),
            license: info.license,
        })
    }

    async fn fetch_pub(&self, name: &str) -> Result<PkgInfo> {
        let url = format!("https://pub.dev/api/packages/{}", name);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("pub.dev package '{}' not found (HTTP {})", name, resp.status()));
        }
        let data: PubResponse = resp.json().await?;
        let latest = data.latest;
        let pubspec = latest.pubspec;

        Ok(PkgInfo {
            name: pubspec.name,
            version: pubspec.version,
            description: pubspec.description.unwrap_or_default(),
            runtime: "pub".to_string(),
            homepage: pubspec.homepage,
            repository: None,
            license: None,
        })
    }

    async fn fetch_cargo(&self, name: &str) -> Result<PkgInfo> {
        let url = format!("https://crates.io/api/v1/crates/{}", name);
        let resp = self.client.get(&url)
            .header("User-Agent", "onpkg/0.1.0")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("crates.io crate '{}' not found (HTTP {})", name, resp.status()));
        }
        let data: CargoResponse = resp.json().await?;
        let crate_data = data.crate_data;
        let max_version = &crate_data.max_version;

        Ok(PkgInfo {
            name: crate_data.name,
            version: max_version.clone(),
            description: crate_data.description.unwrap_or_default(),
            runtime: "cargo".to_string(),
            homepage: crate_data.homepage,
            repository: crate_data.repository,
            license: crate_data.license,
        })
    }
}

// ── NPM response types ─────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct NpmResponse {
    #[serde(rename = "dist-tags")]
    dist_tags: std::collections::HashMap<String, String>,
    versions: std::collections::HashMap<String, NpmVersion>,
}

#[derive(Deserialize, Default, Clone)]
struct NpmVersion {
    version: Option<String>,
    description: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    repository: Option<NpmRepo>,
}

#[derive(Deserialize, Default, Clone)]
struct NpmRepo {
    url: Option<String>,
}

// ── PyPI response types ───────────────────────────────────────────────

#[derive(Deserialize)]
struct PyPIResponse {
    info: PyPIInfo,
}

#[derive(Deserialize)]
struct PyPIInfo {
    name: String,
    version: String,
    summary: Option<String>,
    home_page: Option<String>,
    license: Option<String>,
    project_urls: Option<std::collections::HashMap<String, String>>,
}

// ── pub.dev response types ────────────────────────────────────────────

#[derive(Deserialize)]
struct PubResponse {
    latest: PubVersion,
}

#[derive(Deserialize)]
struct PubVersion {
    pubspec: PubPubspec,
}

#[derive(Deserialize)]
struct PubPubspec {
    name: String,
    version: String,
    description: Option<String>,
    homepage: Option<String>,
}

// ── Cargo response types ──────────────────────────────────────────────

#[derive(Deserialize)]
struct CargoResponse {
    #[serde(rename = "crate")]
    crate_data: CargoCrate,
}

#[derive(Deserialize)]
struct CargoCrate {
    name: String,
    max_version: String,
    description: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    license: Option<String>,
}

// ── Manifest file helpers ─────────────────────────────────────────────

fn add_to_npm_project(name: &str, version: &str, project_dir: &Path) -> Result<()> {
    let pkg_path = project_dir.join("package.json");
    if !pkg_path.exists() {
        return Err(anyhow!("No package.json found in {:?}", project_dir));
    }
    let content = std::fs::read_to_string(&pkg_path)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    // Ensure dependencies object exists
    if !json.is_object() {
        return Err(anyhow!("Invalid package.json"));
    }
    let obj = json.as_object_mut().unwrap();
    if !obj.contains_key("dependencies") {
        obj.insert("dependencies".to_string(), serde_json::json!({}));
    }
    let deps = obj.get_mut("dependencies").unwrap();
    deps.as_object_mut()
        .ok_or_else(|| anyhow!("dependencies is not an object"))?
        .insert(name.to_string(), serde_json::Value::String(format!("^{}", version)));

    std::fs::write(&pkg_path, serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

fn add_to_python_project(name: &str, version: &str, project_dir: &Path) -> Result<()> {
    let pyproject = project_dir.join("pyproject.toml");
    let requirements = project_dir.join("requirements.txt");

    if pyproject.exists() {
        let content = std::fs::read_to_string(&pyproject)?;
        let dep_line = format!("{}>={}", name, version);
        // Append to dependency array in pyproject.toml
        if content.contains("[project]") && content.contains("dependencies = [") {
            let new_content = content.replace("dependencies = [", &format!("dependencies = [\n    \"{}\",", dep_line));
            std::fs::write(&pyproject, new_content)?;
        } else {
            // Append to end of file
            let new_content = format!("{}\n{}\n", content.trim_end(), dep_line);
            std::fs::write(&pyproject, new_content)?;
        }
    } else if requirements.exists() {
        let mut content = std::fs::read_to_string(&requirements)?;
        content.push_str(&format!("{}>={}\n", name, version));
        std::fs::write(&requirements, content)?;
    } else {
        std::fs::write(&requirements, format!("{}>={}\n", name, version))?;
    }
    Ok(())
}

fn add_to_flutter_project(name: &str, version: &str, project_dir: &Path) -> Result<()> {
    let pubspec = project_dir.join("pubspec.yaml");
    if !pubspec.exists() {
        return Err(anyhow!("No pubspec.yaml found in {:?}", project_dir));
    }
    let content = std::fs::read_to_string(&pubspec)?;
    // Add dependency under the dependencies section
    let dep_line = format!("  {}: ^{}", name, version);
    let new_content = if let Some(pos) = content.find("dependencies:") {
        let after = &content[pos..];
        if let Some(end_of_section) = after.find("\n\ndev_dependencies:") {
            let before = &content[..pos + after.len() - after[end_of_section..].len()];
            format!("{}\n{}\n{}", before.trim_end(), dep_line, &content[pos + after.len() - after[end_of_section..].len()..])
        } else {
            format!("{}\n{}\n", content.trim_end(), dep_line)
        }
    } else {
        return Err(anyhow!("No dependencies section in pubspec.yaml"));
    };
    std::fs::write(&pubspec, new_content)?;
    Ok(())
}

fn add_to_cargo_project(name: &str, version: &str, project_dir: &Path) -> Result<()> {
    let cargo_toml = project_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(anyhow!("No Cargo.toml found in {:?}", project_dir));
    }
    let content = std::fs::read_to_string(&cargo_toml)?;

    let new_content = if content.contains("[dependencies]") {
        // Insert after [dependencies] line
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut inserted = false;
        for line in lines {
            result.push(line.to_string());
            if line.trim() == "[dependencies]" && !inserted {
                result.push(format!("{} = \"{}\"", name, version));
                inserted = true;
            }
        }
        result.join("\n")
    } else {
        format!("{}\n[dependencies]\n{} = \"{}\"\n", content.trim_end(), name, version)
    };
    std::fs::write(&cargo_toml, new_content)?;
    Ok(())
}
