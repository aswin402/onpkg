use crate::config::Config;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryTemplate {
    pub name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistrySkill {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryPackage {
    pub name: String,
    pub runtime: String,
    pub description: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResults {
    pub templates: Vec<RegistryTemplate>,
    pub skills: Vec<RegistrySkill>,
    pub packages: Vec<RegistryPackage>,
}

pub struct Registry {
    config: Config,
    client: reqwest::Client,
}

impl Registry {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.network.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    pub async fn search(&self, query: &str, r#type: Option<&str>) -> Result<SearchResults> {
        let base = self.config.registry_url();
        let mut url = format!("{}/api/search?q={}", base, urlencoding(query));
        if let Some(t) = r#type {
            url.push_str(&format!("&type={}", t));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Registry search failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(anyhow!("Registry returned HTTP {}", resp.status()));
        }

        let results: SearchResults = resp
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search results: {}", e))?;
        Ok(results)
    }

    pub async fn get_template(&self, name: &str) -> Result<RegistryTemplate> {
        let url = format!("{}/api/templates/{}", self.config.registry_url(), name);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch template '{}': {}", name, e))?;
        if !resp.status().is_success() {
            return Err(anyhow!(
                "Template '{}' not found (HTTP {})",
                name,
                resp.status()
            ));
        }
        let tmpl: RegistryTemplate = resp.json().await?;
        Ok(tmpl)
    }

    pub async fn get_skill(&self, name: &str) -> Result<RegistrySkill> {
        let url = format!("{}/api/skills/{}", self.config.registry_url(), name);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch skill '{}': {}", name, e))?;
        if !resp.status().is_success() {
            return Err(anyhow!(
                "Skill '{}' not found (HTTP {})",
                name,
                resp.status()
            ));
        }
        let skill: RegistrySkill = resp.json().await?;
        Ok(skill)
    }

    pub async fn download_template_source(
        &self,
        _source: &str,
        _dest: &std::path::Path,
    ) -> Result<()> {
        Err(anyhow!("Template download not yet implemented"))
    }

    pub async fn publish_template(&self, name: &str) -> Result<()> {
        let template_engine = crate::templates::TemplateEngine::new(self.config.clone());
        let tmpl = template_engine
            .find(name)
            .ok_or_else(|| anyhow!("Local stack template '{}' not found", name))?;

        // 1. Package validation
        if tmpl.name.trim().is_empty() {
            return Err(anyhow!("Template name cannot be empty"));
        }
        if tmpl.description.trim().is_empty() {
            return Err(anyhow!("Template description cannot be empty"));
        }
        let version = semver::Version::parse(&tmpl.version)
            .map_err(|e| anyhow!("Invalid template version '{}': {}", tmpl.version, e))?;
        if tmpl.files.is_empty() {
            return Err(anyhow!("Template must contain at least one file"));
        }

        // 2. Version conflict detection
        if let Ok(existing) = self.get_template(name).await {
            if let Ok(existing_ver) = semver::Version::parse(&existing.version) {
                if version <= existing_ver {
                    return Err(anyhow!(
                        "Version conflict: Template version v{} must be greater than the registry version v{}",
                        version,
                        existing_ver
                    ));
                }
            }
        }

        // 3. HTTP request with token auth
        let url = format!("{}/api/templates", self.config.registry_url());
        let mut req = self.client.post(&url);
        if let Some(token) = &self.config.registry.token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .json(&tmpl)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send publish request: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Registry rejected publish (HTTP {}): {}",
                status,
                err_body
            ));
        }

        Ok(())
    }

    pub async fn publish_skill(&self, name: &str) -> Result<()> {
        let skill_manager = crate::skill::SkillManager::new(
            self.config.clone(),
            crate::db::Database::open(&self.config)?,
        );
        let skill = skill_manager
            .show(name)?
            .ok_or_else(|| anyhow!("Local skill '{}' not found", name))?;

        // Load the SKILL.md content from skill.path
        let content = std::fs::read_to_string(&skill.path)
            .map_err(|e| anyhow!("Failed to read skill file '{}': {}", skill.path, e))?;

        // 1. Package validation
        if skill.name.trim().is_empty() {
            return Err(anyhow!("Skill name cannot be empty"));
        }
        if skill.description.trim().is_empty() {
            return Err(anyhow!("Skill description cannot be empty"));
        }
        let version = semver::Version::parse(&skill.version)
            .map_err(|e| anyhow!("Invalid skill version '{}': {}", skill.version, e))?;
        if content.trim().is_empty() {
            return Err(anyhow!("Skill file cannot be empty"));
        }

        // 2. Version conflict detection
        if let Ok(existing) = self.get_skill(name).await {
            if let Ok(existing_ver) = semver::Version::parse(&existing.version) {
                if version <= existing_ver {
                    return Err(anyhow!(
                        "Version conflict: Skill version v{} must be greater than the registry version v{}",
                        version,
                        existing_ver
                    ));
                }
            }
        }

        // 3. HTTP request with token auth
        let payload = serde_json::json!({
            "name": skill.name,
            "description": skill.description,
            "version": skill.version,
            "content": content,
        });

        let url = format!("{}/api/skills", self.config.registry_url());
        let mut req = self.client.post(&url);
        if let Some(token) = &self.config.registry.token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send publish request: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Registry rejected publish (HTTP {}): {}",
                status,
                err_body
            ));
        }

        Ok(())
    }

    pub async fn check_health(&self) -> Result<HashMap<String, String>> {
        let url = format!("{}/api/health", self.config.registry_url());
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let status: HashMap<String, String> = resp.json().await?;
                Ok(status)
            }
            Ok(resp) => {
                let mut map = HashMap::new();
                map.insert("status".to_string(), format!("HTTP {}", resp.status()));
                Ok(map)
            }
            Err(e) => {
                let mut map = HashMap::new();
                map.insert("status".to_string(), format!("offline ({})", e));
                Ok(map)
            }
        }
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn run_mock_server<F>(mut handler: F, count: usize) -> SocketAddr
    where
        F: FnMut(String) -> (String, String) + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            for _ in 0..count {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = vec![0; 4096];
                    match socket.read(&mut buf).await {
                        Ok(n) if n > 0 => {
                            let req_str = String::from_utf8_lossy(&buf[..n]).to_string();
                            let (status, body) = handler(req_str);
                            let resp = format!(
                                "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
                                status,
                                body.len(),
                                body
                            );
                            let _ = socket.write_all(resp.as_bytes()).await;
                            let _ = socket.flush().await;
                        }
                        _ => {}
                    }
                }
            }
        });

        addr
    }

    #[tokio::test]
    async fn test_publish_template_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().to_path_buf();

        let templates_dir = home.join(".onpkg/templates");
        fs::create_dir_all(&templates_dir).unwrap();
        let template_toml = r#"
name = "custom_stack"
category = "website"
description = "A custom stack for testing"
version = "1.1.0"

[[files]]
path = "test.txt"
content = "hello"
"#;
        fs::write(templates_dir.join("custom_stack.toml"), template_toml).unwrap();

        let handler = |req: String| {
            if req.starts_with("GET /api/templates/custom_stack") {
                ("404 Not Found".to_string(), r#"{"error": "not found"}"#.to_string())
            } else if req.starts_with("POST /api/templates") {
                assert!(req.to_lowercase().contains("authorization: bearer test-token"));
                assert!(req.contains(r#""name":"custom_stack""#));
                assert!(req.contains(r#""version":"1.1.0""#));
                ("200 OK".to_string(), r#"{"status": "ok"}"#.to_string())
            } else {
                ("500 Internal Error".to_string(), "{}".to_string())
            }
        };

        let addr = run_mock_server(handler, 2).await;

        let mut config = Config::default();
        config.home_override = Some(home);
        config.registry.url = format!("http://127.0.0.1:{}", addr.port());
        config.registry.token = Some("test-token".to_string());
        let registry = Registry::new(config);

        let res = registry.publish_template("custom_stack").await;
        assert!(res.is_ok(), "Expected template publish to succeed, got {:?}", res);
    }

    #[tokio::test]
    async fn test_publish_template_version_conflict() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().to_path_buf();

        let templates_dir = home.join(".onpkg/templates");
        fs::create_dir_all(&templates_dir).unwrap();
        let template_toml = r#"
name = "custom_stack"
category = "website"
description = "A custom stack for testing"
version = "1.1.0"

[[files]]
path = "test.txt"
content = "hello"
"#;
        fs::write(templates_dir.join("custom_stack.toml"), template_toml).unwrap();

        let handler = |req: String| {
            if req.starts_with("GET /api/templates/custom_stack") {
                ("200 OK".to_string(), r#"{"name": "custom_stack", "category": "website", "description": "desc", "version": "1.1.0", "author": "author", "source": "source"}"#.to_string())
            } else {
                ("500 Internal Error".to_string(), "{}".to_string())
            }
        };

        let addr = run_mock_server(handler, 1).await;

        let mut config = Config::default();
        config.home_override = Some(home);
        config.registry.url = format!("http://127.0.0.1:{}", addr.port());
        let registry = Registry::new(config);

        let res = registry.publish_template("custom_stack").await;
        assert!(res.is_err(), "Expected version conflict error");
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("Version conflict"),
            "Error message should mention Version conflict, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_publish_skill_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let home = temp_dir.path().to_path_buf();

        let skill_path = temp_dir.path().join("my_skill.md");
        let skill_md = r#"---
name: my_skill
description: Custom skill for testing
version: 2.0.0
---
# My Skill content
"#;
        fs::write(&skill_path, skill_md).unwrap();

        let mut config = Config::default();
        config.home_override = Some(home);
        config.registry.token = Some("skill-token".to_string());
        
        let db = crate::db::Database::open(&config).unwrap();
        let skill_manager = crate::skill::SkillManager::new(config.clone(), db);
        skill_manager.install_from_path("my_skill", &skill_path).unwrap();

        let handler = |req: String| {
            if req.starts_with("GET /api/skills/my_skill") {
                ("404 Not Found".to_string(), r#"{"error": "not found"}"#.to_string())
            } else if req.starts_with("POST /api/skills") {
                assert!(req.to_lowercase().contains("authorization: bearer skill-token"));
                assert!(req.contains(r#""name":"my_skill""#));
                assert!(req.contains(r#""version":"1.0.0""#));
                assert!(req.contains(r#""content":"---\nname: my_skill"#));
                ("200 OK".to_string(), r#"{"status": "ok"}"#.to_string())
            } else {
                ("500 Internal Error".to_string(), "{}".to_string())
            }
        };

        let addr = run_mock_server(handler, 2).await;
        config.registry.url = format!("http://127.0.0.1:{}", addr.port());

        let registry = Registry::new(config);
        let res = registry.publish_skill("my_skill").await;
        assert!(res.is_ok(), "Expected skill publish to succeed, got {:?}", res);
    }
}
