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
            .filter(|f| !f.path.starts_with("offpkg_docs/"))
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

        // Load premium stacks from offpkg
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
