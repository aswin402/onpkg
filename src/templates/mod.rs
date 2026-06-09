use crate::config::Config;
use crate::skill::BuiltinSkills;
use anyhow::{anyhow, Context, Result};
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub mod builtin;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    #[serde(default)]
    pub skip_template: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_content: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateDefinition {
    pub name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub files: Vec<TemplateFile>,
    pub variables: Vec<TemplateVariable>,
    #[serde(default)]
    pub technologies: Vec<String>,
}

impl TemplateDefinition {
    pub fn get_technologies(&self) -> Vec<String> {
        if !self.technologies.is_empty() {
            return self.technologies.clone();
        }
        
        match self.name.as_str() {
            "react-vite" | "react-vite-full" | "react-vite-gsap" => vec!["react".to_string(), "vite".to_string(), "tailwind".to_string()],
            "next-template" | "next-app" | "next-app-full" => vec!["next".to_string(), "react".to_string(), "tailwind".to_string(), "prisma".to_string(), "postgres".to_string()],
            "hono-api" => vec!["hono".to_string()],
            "hono-full" => vec!["hono".to_string(), "prisma".to_string(), "postgres".to_string()],
            "express-api" => vec!["express".to_string()],
            "fastapi" | "fastapi-full" => vec!["fastapi".to_string()],
            "mern" => vec!["mongodb".to_string(), "express".to_string(), "react".to_string()],
            "pern" => vec!["postgres".to_string(), "express".to_string(), "react".to_string(), "prisma".to_string()],
            "flutter-app" | "flutter-riverpod" => vec!["flutter".to_string()],
            "rust-cli" => vec!["rust".to_string()],
            _ => vec![],
        }
    }
}

impl From<crate::stacks::Stack> for TemplateDefinition {
    fn from(stack: crate::stacks::Stack) -> Self {
        let files = stack.files.into_iter()
            .filter(|f| !f.path.starts_with("onpkg_docs/"))
            .map(|f| TemplateFile {
                path: f.path,
                content: f.content,
                skip_template: true,
                binary_content: f.binary_content,
            }).collect();
        
        let mut technologies = vec![];
        if stack.name.contains("react") {
            technologies.push("react".to_string());
        }
        if stack.name.contains("next") {
            technologies.push("next".to_string());
        }
        if stack.name.contains("hono") {
            technologies.push("hono".to_string());
        }
        if stack.name.contains("fastapi") {
            technologies.push("fastapi".to_string());
        }
        if stack.name.contains("flutter") {
            technologies.push("flutter".to_string());
        }
        if stack.name.contains("tailwind") {
            technologies.push("tailwind".to_string());
        }
        if stack.packages.contains(&"prisma".to_string()) || stack.dev_packages.contains(&"prisma".to_string()) {
            technologies.push("prisma".to_string());
        }
        if stack.packages.contains(&"express".to_string()) {
            technologies.push("express".to_string());
        }
        if stack.packages.contains(&"mongoose".to_string()) || stack.packages.contains(&"mongodb".to_string()) {
            technologies.push("mongodb".to_string());
        }
        if stack.packages.contains(&"pg".to_string()) || stack.packages.contains(&"postgresql".to_string()) {
            technologies.push("postgres".to_string());
        }

        match stack.runtime.as_str() {
            "bun" | "npm" => {
                technologies.push("vite".to_string());
            }
            "uv" | "python" => {
                technologies.push("fastapi".to_string());
            }
            "flutter" => {
                technologies.push("flutter".to_string());
            }
            _ => {}
        }
        
        technologies.dedup();

        TemplateDefinition {
            name: stack.name,
            category: match stack.runtime.as_str() {
                "bun" => "frontend".to_string(),
                "uv" => "backend".to_string(),
                "flutter" => "app".to_string(),
                _ => "general".to_string(),
            },
            description: stack.description,
            version: "1.0.0".to_string(),
            files,
            variables: vec![],
            technologies,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: String,
}

pub struct TemplateEngine {
    config: Config,
    jinja: Environment<'static>,
}

impl TemplateEngine {
    pub fn new(config: Config) -> Self {
        let mut jinja = Environment::new();
        jinja.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
        Self { config, jinja }
    }

    /// Get all available templates (builtin + custom)
    pub fn all_templates(&self) -> Vec<TemplateDefinition> {
        let mut templates = Vec::new();

        // Load premium stacks from onpkg
        for stack in crate::stacks::builtin::builtin_stacks() {
            templates.push(TemplateDefinition::from(stack));
        }

        // Load standard built-in templates (only if they don't overwrite stacks by name)
        for tmpl in builtin::builtin_templates() {
            if !templates.iter().any(|t| t.name == tmpl.name) {
                templates.push(tmpl);
            }
        }

        // Load custom templates from ~/.onpkg/templates/
        let custom_dir = self.config.templates_dir();
        if custom_dir.exists() {
            if let Ok(entries) = fs::read_dir(&custom_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(tmpl) = toml::from_str::<TemplateDefinition>(&content) {
                                // De-duplicate by name
                                if let Some(idx) = templates.iter().position(|t| t.name == tmpl.name) {
                                    templates[idx] = tmpl;
                                } else {
                                    templates.push(tmpl);
                                }
                            }
                        }
                    }
                }
            }
        }
        templates
    }

    pub fn find(&self, name: &str) -> Option<TemplateDefinition> {
        self.all_templates().into_iter().find(|t| t.name == name)
    }

    /// Scaffold a template into the target directory
    pub fn scaffold(
        &self,
        template: &TemplateDefinition,
        target_dir: &Path,
        extra_vars: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create target directory {:?}", target_dir))?;

        let mut created = Vec::new();
        let mut vars = HashMap::new();

        // Set default variables
        for v in &template.variables {
            vars.insert(v.name.clone(), v.default.clone());
        }
        // Override with user-provided variables
        for (k, v) in extra_vars {
            vars.insert(k.clone(), v.clone());
        }

        for file in &template.files {
            let dest = target_dir.join(&file.path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            if dest.exists() {
                continue;
            }

            if let Some(binary) = &file.binary_content {
                fs::write(&dest, binary)
                    .with_context(|| format!("Failed to write binary {:?}", dest))?;
            } else {
                let content = if file.skip_template {
                    file.content.clone()
                } else {
                    match self.jinja.render_str(&file.content, &vars) {
                        Ok(rendered) => rendered,
                        Err(e) => {
                            // If template rendering fails, use raw content
                            eprintln!(
                                "  warn: template var substitution failed for {}: {}",
                                file.path, e
                            );
                            file.content.clone()
                        }
                    }
                };

                fs::write(&dest, &content)
                    .with_context(|| format!("Failed to write {:?}", dest))?;
            }
            created.push(file.path.clone());
        }

        // Generate README.md documentation if one doesn't exist
        let readme_path = target_dir.join("README.md");
        if !readme_path.exists() && !created.iter().any(|p| p.to_lowercase() == "readme.md") {
            let project_name = vars.get("project_name").map(|s| s.as_str()).unwrap_or(&template.name);
            let readme = generate_readme(project_name, template);
            fs::write(&readme_path, &readme)?;
            created.push("README.md".to_string());
        }

        // Automatically inject transparent onpkg logo.svg
        let root_assets = target_dir.join("assets");
        if let Ok(_) = fs::create_dir_all(&root_assets) {
            let _ = fs::write(root_assets.join("logo.svg"), logo_svg_transparent());
        }
        
        let src_assets = target_dir.join("src").join("assets");
        if let Ok(_) = fs::create_dir_all(&src_assets) {
            let _ = fs::write(src_assets.join("logo.svg"), logo_svg_transparent());
        }

        Ok(created)
    }

    /// Add a custom template from a local directory
    pub fn add_from_dir(&self, name: &str, source_dir: &Path) -> Result<TemplateDefinition> {
        if !source_dir.exists() {
            return Err(anyhow!("Source directory {:?} does not exist", source_dir));
        }

        let mut files = Vec::new();
        for entry in WalkDir::new(source_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(source_dir)
                    .map_err(|e| anyhow!("Failed to strip prefix: {}", e))?
                    .to_string_lossy()
                    .to_string();

                let content = fs::read_to_string(entry.path())
                    .with_context(|| format!("Failed to read {:?}", entry.path()))?;

                files.push(TemplateFile {
                    path: relative,
                    content,
                    skip_template: false,
                    binary_content: None,
                });
            }
        }

        if files.is_empty() {
            return Err(anyhow!("No files found in {:?}", source_dir));
        }

        let tmpl = TemplateDefinition {
            name: name.to_string(),
            category: "custom".to_string(),
            description: format!("Custom template from {:?}", source_dir),
            version: "1.0.0".to_string(),
            files,
            variables: vec![],
            technologies: vec![],
        };

        // Save to ~/.onpkg/templates/
        let custom_dir = self.config.templates_dir();
        fs::create_dir_all(&custom_dir)?;
        let path = custom_dir.join(format!("{}.toml", name));
        let content = toml::to_string_pretty(&tmpl)?;
        fs::write(&path, &content)?;

        Ok(tmpl)
    }

    /// Remove a custom template
    pub fn remove(&self, name: &str) -> Result<()> {
        let path = self.config.templates_dir().join(format!("{}.toml", name));
        if !path.exists() {
            return Err(anyhow!(
                "'{}' is a built-in template and cannot be removed.",
                name
            ));
        }
        fs::remove_file(&path)?;
        Ok(())
    }
}

/// Generate README.md documentation for a scaffolded project
fn generate_readme(project_name: &str, template: &TemplateDefinition) -> String {
    let dev_cmd = match template.category.as_str() {
        "frontend" | "website" => "npm run dev",
        "backend" => match template.name.as_str() {
            "fastapi" | "fastapi-full" => "uvicorn src.main:app --reload",
            _ => "npm run dev",
        },
        "app" => match template.name.as_str() {
            "flutter-app" => "flutter run",
            "rust-cli" => "cargo run",
            _ => "npm run dev",
        },
        "fullstack" => "npm run dev",
        _ => "npm run dev",
    };

    let build_cmd = match template.name.as_str() {
        "fastapi" | "fastapi-full" => "pip install .",
        "flutter-app" => "flutter build",
        "rust-cli" => "cargo build",
        _ => "npm run build",
    };

    format!(
        r#"# {name}

{description}

## Quick Start

```bash
cd {name}
```

### Install dependencies

```bash
{build_cmd}
```

### Development

```bash
{dev_cmd}
```

## Generated with onpkg

This project was scaffolded using the `{template_name}` template v{template_version}.

## Project Structure

```
{name}/
{files}
```

## Variables

{var_table}
"#,
        name = project_name,
        description = template.description,
        template_name = template.name,
        template_version = template.version,
        build_cmd = build_cmd,
        dev_cmd = dev_cmd,
        files = template
            .files
            .iter()
            .map(|f| format!("├── {}", f.path))
            .collect::<Vec<_>>()
            .join("\n"),
        var_table = template
            .variables
            .iter()
            .map(|v| format!("- `{}`: {} (default: `{}`)", v.name, v.description, v.default))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

// Helper to run online install based on manifest detection
pub fn install_dependencies_online(target_dir: &Path, custom_manager: Option<&str>) -> Result<()> {
    // 1. Detect which manifest files are in the target directory
    let has_package_json = target_dir.join("package.json").exists();
    let has_pyproject = target_dir.join("pyproject.toml").exists() || target_dir.join("requirements.txt").exists();
    let has_pubspec = target_dir.join("pubspec.yaml").exists();
    let has_cargo = target_dir.join("Cargo.toml").exists();

    // 2. Perform online install based on the runtime
    if has_package_json {
        let manager = custom_manager.unwrap_or_else(|| {
            // Check if bun is available, otherwise npm
            if std::process::Command::new("bun").arg("--version").output().is_ok() {
                "bun"
            } else if std::process::Command::new("pnpm").arg("--version").output().is_ok() {
                "pnpm"
            } else if std::process::Command::new("yarn").arg("--version").output().is_ok() {
                "yarn"
            } else {
                "npm"
            }
        });

        println!("  info: package.json found. Running '{} install' online...", manager);
        let status = std::process::Command::new(manager)
            .arg("install")
            .current_dir(target_dir)
            .status()
            .with_context(|| format!("Failed to run '{} install'", manager))?;

        if !status.success() {
            eprintln!("  warn: '{} install' failed with status: {}", manager, status);
        } else {
            println!("  done: dependencies installed successfully using '{}'", manager);
        }
    } else if has_pyproject {
        let manager = custom_manager.unwrap_or_else(|| {
            if std::process::Command::new("uv").arg("--version").output().is_ok() {
                "uv"
            } else {
                "pip"
            }
        });

        println!("  info: Python project found. Running installer via '{}'...", manager);
        let status = if manager == "uv" {
            if target_dir.join("pyproject.toml").exists() {
                std::process::Command::new("uv")
                    .arg("sync")
                    .current_dir(target_dir)
                    .status()
            } else {
                std::process::Command::new("uv")
                    .args(&["pip", "install", "-r", "requirements.txt"])
                    .current_dir(target_dir)
                    .status()
            }
        } else {
            if target_dir.join("pyproject.toml").exists() {
                std::process::Command::new("pip")
                    .args(&["install", "."])
                    .current_dir(target_dir)
                    .status()
            } else {
                std::process::Command::new("pip")
                    .args(&["install", "-r", "requirements.txt"])
                    .current_dir(target_dir)
                    .status()
            }
        };

        match status {
            Ok(s) if s.success() => println!("  done: Python dependencies installed successfully"),
            Ok(s) => eprintln!("  warn: Python installer failed with status: {}", s),
            Err(e) => eprintln!("  warn: Failed to execute Python installer: {}", e),
        }
    } else if has_pubspec {
        println!("  info: pubspec.yaml found. Running 'flutter pub get' online...");
        let status = std::process::Command::new("flutter")
            .args(&["pub", "get"])
            .current_dir(target_dir)
            .status();

        match status {
            Ok(s) if s.success() => println!("  done: Flutter dependencies installed successfully"),
            Ok(s) => {
                // Try dart pub get
                println!("  info: 'flutter' command failed, trying 'dart pub get'...");
                let fallback = std::process::Command::new("dart")
                    .args(&["pub", "get"])
                    .current_dir(target_dir)
                    .status();
                if let Ok(fs) = fallback {
                    if fs.success() {
                        println!("  done: Dart dependencies installed successfully");
                        return Ok(());
                    }
                }
                eprintln!("  warn: Flutter/Dart dependency installation failed with status: {}", s);
            }
            Err(e) => eprintln!("  warn: Failed to run flutter pub get: {}", e),
        }
    } else if has_cargo {
        println!("  info: Cargo.toml found. Running 'cargo check' to fetch crates online...");
        let status = std::process::Command::new("cargo")
            .arg("check")
            .current_dir(target_dir)
            .status();

        match status {
            Ok(s) if s.success() => println!("  done: Cargo dependencies fetched successfully"),
            Ok(s) => eprintln!("  warn: Cargo check failed with status: {}", s),
            Err(e) => eprintln!("  warn: Failed to execute cargo check: {}", e),
        }
    }

    Ok(())
}

// Generate the onpkg_docs directory and skills
pub fn generate_agent_docs(technologies: &[String], target_dir: &Path, config: &Config) -> Result<()> {
    let docs_dir = target_dir.join("onpkg_docs");
    std::fs::create_dir_all(&docs_dir)?;

    for tech in technologies {
        let skill_content = if let Some(content) = BuiltinSkills::get(tech) {
            Some(content)
        } else {
            // Check in skills directory ~/.onpkg/skills/
            let local_path = config.skills_dir().join(format!("{}.md", tech));
            if local_path.exists() {
                std::fs::read_to_string(&local_path).ok()
            } else {
                None
            }
        };

        if let Some(content) = skill_content {
            // Write to onpkg_docs/<tech>.md
            let tech_file = docs_dir.join(format!("{}.md", tech));
            std::fs::write(&tech_file, &content)?;
            
            // Also write to onpkg_docs/<tech>/skill.md
            let tech_sub_dir = docs_dir.join(tech);
            std::fs::create_dir_all(&tech_sub_dir)?;
            std::fs::write(tech_sub_dir.join("skill.md"), &content)?;
            println!("  created: agent skill in onpkg_docs/{}", tech);
        } else {
            // Write a generic agent skill template
            let generic_skill = format!(
                r#"---
name: {name}
description: "AI Agent Skill for {name}"
---

# {name} Agent Skill

This is a generated AI agent skill document for `{name}`.

## Guidelines
- Follow standard project conventions for `{name}`.
- Optimize for performance and clean architecture.

## Typical Commands
- See project documentation and config files.
"#,
                name = tech
            );
            
            let tech_file = docs_dir.join(format!("{}.md", tech));
            std::fs::write(&tech_file, &generic_skill)?;
            
            let tech_sub_dir = docs_dir.join(tech);
            std::fs::create_dir_all(&tech_sub_dir)?;
            std::fs::write(tech_sub_dir.join("skill.md"), &generic_skill)?;
            println!("  created: agent skill template in onpkg_docs/{}", tech);
        }
    }

    // Generate INDEX.md
    let mut index_content = "# Project AI Agent Skills 🧠\n\nThis directory contains instructions and guidelines for AI agents working on this project.\n\n## Available Skills\n".to_string();
    for tech in technologies {
        index_content.push_str(&format!("- [{tech}](file://./{tech}.md) / [{tech}/skill.md](file://./{tech}/skill.md)\n"));
    }
    std::fs::write(docs_dir.join("INDEX.md"), index_content)?;

    Ok(())
}

/// Generate the onpkg.json AI Agent project manifest in the project root
pub fn generate_onpkg_manifest(
    tmpl: &TemplateDefinition,
    target_dir: &std::path::Path,
    technologies: &[String],
) -> Result<()> {
    // 1. Detect project name
    let project_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    // 2. Detect runtime and package manager
    let mut runtime = "node".to_string();
    let mut package_manager = "npm".to_string();
    
    // Check files to determine runtime/package manager
    if target_dir.join("package.json").exists() {
        runtime = "node".to_string();
        package_manager = "npm".to_string();
        if tmpl.name.contains("react") || tmpl.name.contains("next") || tmpl.name.contains("hono") || tmpl.name.contains("mern") || tmpl.name.contains("pern") {
            runtime = "bun".to_string();
            package_manager = "bun".to_string();
        }
    } else if target_dir.join("pyproject.toml").exists() || target_dir.join("requirements.txt").exists() {
        runtime = "python".to_string();
        package_manager = "uv".to_string();
    } else if target_dir.join("pubspec.yaml").exists() {
        runtime = "flutter".to_string();
        package_manager = "flutter".to_string();
    } else if target_dir.join("Cargo.toml").exists() {
        runtime = "rust".to_string();
        package_manager = "cargo".to_string();
    }

    // 3. Locate entrypoint and routing directories
    let mut architecture = std::collections::BTreeMap::new();
    let entrypoints = [
        "src/main.tsx",
        "src/main.ts",
        "src/index.js",
        "src/main.rs",
        "lib/main.dart",
        "app/main.py",
        "src/main.py",
        "main.py",
    ];
    for ep in &entrypoints {
        if target_dir.join(ep).exists() {
            architecture.insert("entrypoint".to_string(), ep.to_string());
            break;
        }
    }

    let dirs = [
        ("routing", vec!["src/routes", "src/pages", "app/"]),
        ("components", vec!["src/components", "components/"]),
        ("styles", vec!["src/index.css", "src/App.css", "src/styles.css", "styles/"]),
        ("database", vec!["prisma/schema.prisma", "src/db", "db/"]),
    ];

    for (key, paths) in &dirs {
        for path in paths {
            if target_dir.join(path).exists() {
                architecture.insert(key.to_string(), path.to_string());
                break;
            }
        }
    }

    // 4. Collect scripts and packages
    let mut scripts = std::collections::BTreeMap::new();
    let mut core_packages = Vec::new();

    // Parse package.json if it exists
    let pkg_json_path = target_dir.join("package.json");
    if pkg_json_path.exists() {
        if let Ok(content) = fs::read_to_string(&pkg_json_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(s) = v.get("scripts").and_then(|s| s.as_object()) {
                    for (k, val) in s {
                        if let Some(val_str) = val.as_str() {
                            scripts.insert(k.clone(), val_str.to_string());
                        }
                    }
                }
                if let Some(deps) = v.get("dependencies").and_then(|d| d.as_object()) {
                    for (k, _) in deps {
                        core_packages.push(k.clone());
                    }
                }
            }
        }
    }

    // Parse Cargo.toml if it exists
    let cargo_toml_path = target_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
            if let Ok(v) = toml::from_str::<toml::Value>(&content) {
                if let Some(deps) = v.get("dependencies").and_then(|d| d.as_table()) {
                    for (k, _) in deps {
                        core_packages.push(k.clone());
                    }
                }
            }
        }
        scripts.insert("dev".to_string(), "cargo run".to_string());
        scripts.insert("build".to_string(), "cargo build".to_string());
        scripts.insert("test".to_string(), "cargo test".to_string());
    }

    // Parse pyproject.toml if it exists
    let pyproject_toml_path = target_dir.join("pyproject.toml");
    if pyproject_toml_path.exists() {
        if let Ok(content) = fs::read_to_string(&pyproject_toml_path) {
            if let Ok(v) = toml::from_str::<toml::Value>(&content) {
                if let Some(project) = v.get("project") {
                    if let Some(deps) = project.get("dependencies").and_then(|d| d.as_array()) {
                        for dep in deps {
                            if let Some(dep_str) = dep.as_str() {
                                let name = dep_str.split(&['>', '=', '<', '~', '!'][..]).next().unwrap_or(dep_str).trim();
                                core_packages.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        scripts.insert("dev".to_string(), "uvicorn app.main:app --reload".to_string());
    }

    // Default scripts if empty
    if scripts.is_empty() {
        if runtime == "flutter" {
            scripts.insert("dev".to_string(), "flutter run".to_string());
            scripts.insert("build".to_string(), "flutter build apk".to_string());
        } else {
            scripts.insert("dev".to_string(), "npm run dev".to_string());
            scripts.insert("build".to_string(), "npm run build".to_string());
        }
    }

    // 5. Collect active skills
    let mut active_skills = Vec::new();
    for tech in technologies {
        let tech_filename = format!("{}.md", tech);
        if target_dir.join("onpkg_docs").join(&tech_filename).exists() {
            active_skills.push(tech_filename);
        }
    }

    // 6. Build the manifest JSON structure
    let mut manifest = std::collections::BTreeMap::new();
    
    let mut project_info = std::collections::BTreeMap::new();
    project_info.insert("name".to_string(), serde_json::Value::String(project_name));
    project_info.insert("stack".to_string(), serde_json::Value::String(tmpl.name.clone()));
    project_info.insert("runtime".to_string(), serde_json::Value::String(runtime));
    project_info.insert("package_manager".to_string(), serde_json::Value::String(package_manager));
    manifest.insert("project".to_string(), serde_json::Value::Object(project_info.into_iter().collect()));

    let arch_info: serde_json::Map<String, serde_json::Value> = architecture.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect();
    manifest.insert("architecture".to_string(), serde_json::Value::Object(arch_info));

    let mut agent_info = std::collections::BTreeMap::new();
    agent_info.insert("docs_directory".to_string(), serde_json::Value::String("onpkg_docs/".to_string()));
    agent_info.insert("active_skills".to_string(), serde_json::Value::Array(active_skills.into_iter().map(serde_json::Value::String).collect()));
    manifest.insert("agent_instructions".to_string(), serde_json::Value::Object(agent_info.into_iter().collect()));

    let scripts_info: serde_json::Map<String, serde_json::Value> = scripts.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect();
    manifest.insert("scripts".to_string(), serde_json::Value::Object(scripts_info));

    let mut packages_info = std::collections::BTreeMap::new();
    packages_info.insert("core".to_string(), serde_json::Value::Array(core_packages.into_iter().map(serde_json::Value::String).collect()));
    packages_info.insert("added_by_agent".to_string(), serde_json::Value::Array(vec![]));
    manifest.insert("packages".to_string(), serde_json::Value::Object(packages_info.into_iter().collect()));

    // 7. Write to target_dir/onpkg.json
    let manifest_path = target_dir.join("onpkg.json");
    if let Ok(json_str) = serde_json::to_string_pretty(&manifest) {
        fs::write(&manifest_path, json_str)?;
        println!("  created: AI Agent Project Manifest in onpkg.json");
    }

    Ok(())
}

/// Returns the raw content of the transparent animated onpkg logo in SVG format
pub fn logo_svg_transparent() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 800" width="100%" height="100%">
  <defs>
    <!-- Neon Gradients -->
    <linearGradient id="neonCyan" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00f3ff" />
      <stop offset="100%" stop-color="#0066ff" />
    </linearGradient>
    
    <linearGradient id="neonPurple" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#ff00cc" />
      <stop offset="100%" stop-color="#9900ff" />
    </linearGradient>

    <linearGradient id="cyanPurple" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00f3ff" />
      <stop offset="50%" stop-color="#ab00ff" />
      <stop offset="100%" stop-color="#ff00cc" />
    </linearGradient>

    <!-- Glow Filters -->
    <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="8" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>

    <filter id="glow-intense" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="15" result="blur1" />
      <feGaussianBlur stdDeviation="5" result="blur2" />
      <feMerge>
        <feMergeNode in="blur1" />
        <feMergeNode in="blur2" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
  </defs>

  <style>
    /* Isometric Box Style */
    .box-edge {
      stroke: url(#cyanPurple);
      stroke-width: 3.5;
      fill: none;
      stroke-linecap: round;
      stroke-linejoin: round;
      filter: url(#glow);
      opacity: 0.9;
    }
    
    .box-back {
      stroke: url(#neonPurple);
      stroke-dasharray: 6 6;
      opacity: 0.4;
      stroke-width: 2;
    }
    
    /* Circuits */
    .circuit-line {
      fill: none;
      stroke-width: 3;
      stroke-linecap: round;
      filter: url(#glow);
      stroke-dasharray: 300;
      stroke-dashoffset: 300;
      animation: drawCircuit 4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
    }

    .cyan-line {
      stroke: url(#neonCyan);
    }

    .purple-line {
      stroke: url(#neonPurple);
    }
    
    /* Glowing Dots */
    .glow-dot {
      fill: #fff;
      filter: url(#glow-intense);
      opacity: 0;
      animation: pulseDot 4s ease-in-out infinite;
    }
    
    /* Particles */
    .particle {
      fill: #00f3ff;
      filter: url(#glow);
      opacity: 0;
      animation: floatUp 5s ease-in-out infinite;
    }

    /* Keyframes */
    @keyframes drawCircuit {
      0% {
        stroke-dashoffset: 300;
      }
      50% {
        stroke-dashoffset: 0;
      }
      100% {
        stroke-dashoffset: -300;
      }
    }

    @keyframes pulseDot {
      0%, 100% {
        opacity: 0;
        transform: scale(0.3);
      }
      30%, 70% {
        opacity: 1;
        transform: scale(1.1);
      }
    }

    @keyframes floatUp {
      0% {
        transform: translate(0, 0) scale(0.8);
        opacity: 0;
      }
      20% {
        opacity: 0.8;
      }
      80% {
        opacity: 0.8;
      }
      100% {
        transform: translate(var(--tx), var(--ty)) scale(0.3);
        opacity: 0;
      }
    }
    
    /* Typography */
    .text-title {
      font-family: 'Outfit', 'Montserrat', 'Inter', sans-serif;
      font-size: 76px;
      font-weight: 800;
      fill: url(#cyanPurple);
      letter-spacing: 5px;
      filter: url(#glow);
    }

    .text-subtitle {
      font-family: 'Inter', sans-serif;
      font-size: 18px;
      font-weight: 600;
      fill: #94a3b8;
      letter-spacing: 7px;
      opacity: 0.8;
    }
  </style>

  <g transform="translate(400, 360)">
    <!-- Circuit Lines (Emerging from the Box) -->
    <!-- Center Left -->
    <path class="circuit-line cyan-line" d="M 0, -5 L -40, -65 L -120, -65 L -150, -115" style="animation-delay: 0s;" />
    <!-- Center Right -->
    <path class="circuit-line purple-line" d="M 0, -5 L 40, -65 L 120, -65 L 150, -115" style="animation-delay: 0.5s;" />
    <!-- Mid Left Up -->
    <path class="circuit-line purple-line" d="M -20, 15 L -80, -45 L -80, -125 L -130, -175" style="animation-delay: 1s;" />
    <!-- Mid Right Up -->
    <path class="circuit-line cyan-line" d="M 20, 15 L 80, -45 L 80, -125 L 130, -175" style="animation-delay: 1.5s;" />
    <!-- Center Straight Up -->
    <path class="circuit-line cyan-line" d="M 0, -10 L 0, -155 L -30, -205" style="animation-delay: 0.8s;" />
    <path class="circuit-line purple-line" d="M 0, -10 L 0, -125 L 30, -175" style="animation-delay: 1.2s;" />

    <!-- Glowing Dots at endpoints -->
    <circle class="glow-dot" cx="-150" cy="-115" r="6" style="animation-delay: 2.0s; fill: #00f3ff;" />
    <circle class="glow-dot" cx="150" cy="-115" r="6" style="animation-delay: 2.5s; fill: #ff00cc;" />
    <circle class="glow-dot" cx="-130" cy="-175" r="6" style="animation-delay: 3.0s; fill: #ff00cc;" />
    <circle class="glow-dot" cx="130" cy="-175" r="6" style="animation-delay: 3.5s; fill: #00f3ff;" />
    <circle class="glow-dot" cx="-30" cy="-205" r="5" style="animation-delay: 2.8s; fill: #00f3ff;" />
    <circle class="glow-dot" cx="30" cy="-175" r="5" style="animation-delay: 3.2s; fill: #ff00cc;" />

    <!-- Floating Particles -->
    <circle class="particle" cx="-20" cy="-50" r="3.5" style="--tx: -60px; --ty: -150px; animation-delay: 0s;" />
    <circle class="particle" cx="30" cy="-30" r="2.5" style="--tx: 90px; --ty: -180px; animation-delay: 1.2s; fill: #ff00cc;" />
    <circle class="particle" cx="-10" cy="-70" r="3" style="--tx: -40px; --ty: -200px; animation-delay: 2.4s;" />
    <circle class="particle" cx="15" cy="-60" r="3.5" style="--tx: 50px; --ty: -130px; animation-delay: 3.6s; fill: #ff00cc;" />

    <!-- Isometric Box (Package) -->
    <!-- Back wireframe (inside the box) -->
    <path class="box-edge box-back" d="M 0, -30 L 0, 60 M -90, 110 L 0, 60 L 90, 110" />
    
    <!-- Front Face Left -->
    <path class="box-edge" d="M -90, 20 L -90, 110 L 0, 160 M 0, 160 L 0, 70" />
    <!-- Front Face Right -->
    <path class="box-edge" d="M 90, 20 L 90, 110 L 0, 160" />
    <!-- Top Rim (Opening) -->
    <path class="box-edge" d="M -90, 20 L 0, 70 L 90, 20 L 0, -30 Z" />

    <!-- Left-Back Flap -->
    <path class="box-edge" d="M -90, 20 L -140, -10 L -50, -60 L 0, -30" />
    <!-- Right-Back Flap -->
    <path class="box-edge" d="M 90, 20 L 140, -10 L 50, -60 L 0, -30" />
    <!-- Front-Left Flap -->
    <path class="box-edge" d="M -90, 20 L -140, 50 L -50, 100 L 0, 70" />
    <!-- Front-Right Flap -->
    <path class="box-edge" d="M 90, 20 L 140, 50 L 50, 100 L 0, 70" />
  </g>

  <!-- Text Branding -->
  <g transform="translate(400, 610)" text-anchor="middle">
    <!-- Stylized "onpkg" text -->
    <text class="text-title" x="0" y="0">onpkg</text>
    <!-- Subtitle -->
    <text class="text-subtitle" x="0" y="45">AI AGENT PACKAGE MANAGER</text>
  </g>
</svg>"##
}

