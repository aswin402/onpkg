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
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
    #[serde(default)]
    pub technologies: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<crate::stacks::StackHook>,
}

impl TemplateDefinition {
    pub fn get_technologies(&self) -> Vec<String> {
        if !self.technologies.is_empty() {
            return self.technologies.clone();
        }

        match self.name.as_str() {
            "react-vite" | "react-vite-full" | "react-vite-gsap" => vec![
                "react".to_string(),
                "vite".to_string(),
                "tailwind".to_string(),
            ],
            "next-template" | "next-app" | "next-app-full" => vec![
                "next".to_string(),
                "react".to_string(),
                "tailwind".to_string(),
                "prisma".to_string(),
                "postgres".to_string(),
            ],
            "hono-api" => vec!["hono".to_string()],
            "hono-full" => vec![
                "hono".to_string(),
                "prisma".to_string(),
                "postgres".to_string(),
            ],
            "express-api" => vec!["express".to_string()],
            "fastapi" | "fastapi-full" => vec!["fastapi".to_string()],
            "mern" => vec![
                "mongodb".to_string(),
                "express".to_string(),
                "react".to_string(),
            ],
            "pern" => vec![
                "postgres".to_string(),
                "express".to_string(),
                "react".to_string(),
                "prisma".to_string(),
            ],
            "flutter-app" | "flutter-riverpod" => vec!["flutter".to_string()],
            "rust-cli" => vec!["rust".to_string()],
            _ => vec![],
        }
    }
}

impl From<crate::stacks::Stack> for TemplateDefinition {
    fn from(stack: crate::stacks::Stack) -> Self {
        let files = stack
            .files
            .into_iter()
            .filter(|f| !f.path.starts_with("onpkg_docs/"))
            .map(|f| TemplateFile {
                path: f.path,
                content: f.content,
                skip_template: true,
                binary_content: f.binary_content,
            })
            .collect();

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
        if stack.packages.contains(&"prisma".to_string())
            || stack.dev_packages.contains(&"prisma".to_string())
        {
            technologies.push("prisma".to_string());
        }
        if stack.packages.contains(&"express".to_string()) {
            technologies.push("express".to_string());
        }
        if stack.packages.contains(&"mongoose".to_string())
            || stack.packages.contains(&"mongodb".to_string())
        {
            technologies.push("mongodb".to_string());
        }
        if stack.packages.contains(&"pg".to_string())
            || stack.packages.contains(&"postgresql".to_string())
        {
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
            hooks: stack.hooks,
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
                                if let Some(idx) =
                                    templates.iter().position(|t| t.name == tmpl.name)
                                {
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
            let project_name = vars
                .get("project_name")
                .map(|s| s.as_str())
                .unwrap_or(&template.name);
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
            hooks: vec![],
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
            .map(|v| format!(
                "- `{}`: {} (default: `{}`)",
                v.name, v.description, v.default
            ))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

// Helper to run online install based on manifest detection
pub fn install_dependencies_online(target_dir: &Path, custom_manager: Option<&str>) -> Result<()> {
    // 1. Detect which manifest files are in the target directory
    let has_package_json = target_dir.join("package.json").exists();
    let has_pyproject =
        target_dir.join("pyproject.toml").exists() || target_dir.join("requirements.txt").exists();
    let has_pubspec = target_dir.join("pubspec.yaml").exists();
    let has_cargo = target_dir.join("Cargo.toml").exists();

    // 2. Perform online install based on the runtime
    if has_package_json {
        let manager = custom_manager.unwrap_or_else(|| {
            // Check if bun is available, otherwise npm
            if std::process::Command::new("bun")
                .arg("--version")
                .output()
                .is_ok()
            {
                "bun"
            } else if std::process::Command::new("pnpm")
                .arg("--version")
                .output()
                .is_ok()
            {
                "pnpm"
            } else if std::process::Command::new("yarn")
                .arg("--version")
                .output()
                .is_ok()
            {
                "yarn"
            } else {
                "npm"
            }
        });

        println!(
            "  info: package.json found. Running '{} install' online...",
            manager
        );
        let status = std::process::Command::new(manager)
            .arg("install")
            .current_dir(target_dir)
            .status()
            .with_context(|| format!("Failed to run '{} install'", manager))?;

        if !status.success() {
            eprintln!(
                "  warn: '{} install' failed with status: {}",
                manager, status
            );
        } else {
            println!(
                "  done: dependencies installed successfully using '{}'",
                manager
            );
        }
    } else if has_pyproject {
        let manager = custom_manager.unwrap_or_else(|| {
            if std::process::Command::new("uv")
                .arg("--version")
                .output()
                .is_ok()
            {
                "uv"
            } else {
                "pip"
            }
        });

        println!(
            "  info: Python project found. Running installer via '{}'...",
            manager
        );
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
                eprintln!(
                    "  warn: Flutter/Dart dependency installation failed with status: {}",
                    s
                );
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
pub fn generate_agent_docs(
    technologies: &[String],
    target_dir: &Path,
    config: &Config,
) -> Result<()> {
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
        index_content.push_str(&format!(
            "- [{tech}](file://./{tech}.md) / [{tech}/skill.md](file://./{tech}/skill.md)\n"
        ));
    }
    std::fs::write(docs_dir.join("INDEX.md"), index_content)?;

    Ok(())
}

/// Generate the onpkg.json AI Agent project manifest in the project root
pub fn generate_onpkg_manifest(
    tmpl: &TemplateDefinition,
    target_dir: &std::path::Path,
    technologies: &[String],
    variables: Option<&std::collections::HashMap<String, String>>,
) -> Result<()> {
    sync_onpkg_project(target_dir, Some(&tmpl.name), Some(technologies), variables, false, false)
}

/// Scaffold stack template and execute all post-scaffold setups (hooks, docs, manifest generation)
pub fn scaffold_and_setup_stack(
    template_engine: &TemplateEngine,
    tmpl: &TemplateDefinition,
    target: &std::path::Path,
    extra_vars: &std::collections::HashMap<String, String>,
    manager: Option<&str>,
    no_hooks: bool,
    config: &crate::config::Config,
) -> Result<Vec<String>> {
    let created = template_engine.scaffold(tmpl, target, extra_vars)?;
    
    // Install dependencies
    let _ = crate::templates::install_dependencies_online(target, manager);
    
    // Post scaffold hooks
    if !no_hooks && !tmpl.hooks.is_empty() {
        for hook in &tmpl.hooks {
            let _desc = hook.description.as_deref().unwrap_or(&hook.command);
            let _hook_res = if cfg!(target_os = "windows") {
                std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(&hook.command)
                    .current_dir(target)
                    .output()
            } else {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&hook.command)
                    .current_dir(target)
                    .output()
            };
        }
    }
    
    let techs = tmpl.get_technologies();
    let _ = crate::templates::generate_agent_docs(&techs, target, config);
    let _ = crate::templates::generate_onpkg_manifest(tmpl, target, &techs, Some(extra_vars));
    
    Ok(created)
}

/// Generate an AGENTS.md in the project root for AI coding agents.
/// Preserves user-written content under the '## Agent-Specific Notes' section.
pub fn generate_agents_md(
    target_dir: &std::path::Path,
    project_name: &str,
    runtime: &str,
    package_manager: &str,
    architecture: &std::collections::BTreeMap<String, String>,
    scripts: &std::collections::BTreeMap<String, String>,
    active_skills: &[String],
) -> Result<()> {
    let agents_path = target_dir.join("AGENTS.md");

    let techs: Vec<String> = active_skills
        .iter()
        .map(|s| s.trim_end_matches(".md").to_string())
        .collect();

    let mut content = format!(
        "# {} \u{2014} Agent Instructions\n\n\
         > Auto-generated by `onpkg sync`. Edit freely \u{2014} onpkg preserves manual sections on re-sync.\n\n\
         ## Project\n\
         - **Runtime:** {}\n\
         - **Package Manager:** {}\n\
         - **Technologies:** {}\n\n",
        project_name,
        runtime,
        package_manager,
        if techs.is_empty() {
            "N/A".to_string()
        } else {
            techs.join(", ")
        },
    );

    // Build commands
    if !scripts.is_empty() {
        content.push_str("## Commands\n");
        for (name, cmd) in scripts {
            content.push_str(&format!("- **{}:** `{}`\n", name, cmd));
        }
        content.push('\n');
    }

    // Architecture
    if !architecture.is_empty() {
        content.push_str("## Architecture\n");
        for (key, path) in architecture {
            content.push_str(&format!("- **{}:** `{}`\n", key, path));
        }
        content.push('\n');
    }

    // Coding guidelines (user-editable)
    content.push_str(
        "## Coding Guidelines\n\
         - Follow existing patterns in the codebase\n\
         - Read `onpkg_docs/` for technology-specific rules\n\
         - Use the project's established naming conventions\n\n",
    );

    // Agent-Specific Notes section - preserved across re-syncs
    let user_notes_header = "## Agent-Specific Notes";
    let default_notes = format!(
        "{}\n\
         <!-- Add custom instructions for AI agents below this line -->\n",
        user_notes_header
    );

    // If file already exists, preserve everything from '## Agent-Specific Notes' onward
    if agents_path.exists() {
        let existing = fs::read_to_string(&agents_path).unwrap_or_default();
        if let Some(pos) = existing.find(user_notes_header) {
            content.push_str(&existing[pos..]);
        } else {
            content.push_str(&default_notes);
        }
    } else {
        content.push_str(&default_notes);
    }

    fs::write(&agents_path, content)?;
    println!("  synchronized: AGENTS.md in project root");
    Ok(())
}

fn create_claude_md_symlink(target_dir: &std::path::Path) -> Result<()> {
    let agents_path = target_dir.join("AGENTS.md");
    let claude_path = target_dir.join("CLAUDE.md");

    if claude_path.exists() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        if let Err(e) = std::os::unix::fs::symlink("AGENTS.md", &claude_path) {
            println!("  Warning: Failed to create CLAUDE.md symlink: {}. Copying instead...", e);
            std::fs::copy(&agents_path, &claude_path)?;
        } else {
            println!("  created: CLAUDE.md symlink pointing to AGENTS.md");
        }
    }

    #[cfg(windows)]
    {
        if let Err(e) = std::os::windows::fs::symlink_file("AGENTS.md", &claude_path) {
            println!("  Warning: Failed to create CLAUDE.md symlink: {}. Copying instead...", e);
            std::fs::copy(&agents_path, &claude_path)?;
        } else {
            println!("  created: CLAUDE.md symlink pointing to AGENTS.md");
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        std::fs::copy(&agents_path, &claude_path)?;
        println!("  created: CLAUDE.md copied from AGENTS.md");
    }

    Ok(())
}

/// Sync the project files and packages to onpkg.json and update/create workflow docs in onpkg_docs/
pub fn sync_onpkg_project(
    target_dir: &std::path::Path,
    stack_name: Option<&str>,
    technologies: Option<&[String]>,
    variables: Option<&std::collections::HashMap<String, String>>,
    no_agents_md: bool,
    symlink_claude: bool,
) -> Result<()> {
    // 1. Detect project name
    let project_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    // 2. Load existing manifest if present
    let onpkg_path = target_dir.join("onpkg.json");
    let mut existing_manifest: Option<serde_json::Value> = None;
    if onpkg_path.exists() {
        if let Ok(content) = fs::read_to_string(&onpkg_path) {
            existing_manifest = serde_json::from_str(&content).ok();
        }
    }

    // 3. Detect runtime and package manager
    let mut runtime = "node".to_string();
    let mut package_manager = "npm".to_string();

    // Lock/manifest detection
    if target_dir.join("bun.lockb").exists() || target_dir.join("bun.lock").exists() {
        runtime = "bun".to_string();
        package_manager = "bun".to_string();
    } else if target_dir.join("package-lock.json").exists() {
        runtime = "node".to_string();
        package_manager = "npm".to_string();
    } else if target_dir.join("pnpm-lock.yaml").exists() {
        runtime = "node".to_string();
        package_manager = "pnpm".to_string();
    } else if target_dir.join("yarn.lock").exists() {
        runtime = "node".to_string();
        package_manager = "yarn".to_string();
    } else if target_dir.join("package.json").exists() {
        // Fallback checks
        runtime = "node".to_string();
        package_manager = "npm".to_string();
        if let Some(ref m) = existing_manifest {
            if let Some(pm) = m
                .get("project")
                .and_then(|p| p.get("package_manager"))
                .and_then(|p| p.as_str())
            {
                package_manager = pm.to_string();
            }
            if let Some(rt) = m
                .get("project")
                .and_then(|p| p.get("runtime"))
                .and_then(|p| p.as_str())
            {
                runtime = rt.to_string();
            }
        }
    } else if target_dir.join("pyproject.toml").exists()
        || target_dir.join("requirements.txt").exists()
    {
        runtime = "python".to_string();
        package_manager = "uv".to_string();
    } else if target_dir.join("pubspec.yaml").exists() {
        runtime = "flutter".to_string();
        package_manager = "flutter".to_string();
    } else if target_dir.join("Cargo.toml").exists() {
        runtime = "rust".to_string();
        package_manager = "cargo".to_string();
    }

    // 4. Scan the project recursively to build the sources block
    let mut files = Vec::new();
    let mut dirs = std::collections::BTreeSet::new();
    let mut extensions = std::collections::BTreeSet::new();

    let allowed_exts = [
        "rs", "ts", "tsx", "js", "jsx", "py", "dart", "html", "css", "prisma", "sql", "toml",
        "json", "yaml", "yml", "md",
    ];

    let project_files = crate::walker::get_project_walker(target_dir)?;
    for path in project_files {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if allowed_exts.contains(&ext) {
                let rel_path = path.strip_prefix(target_dir).unwrap_or(&path);
                let rel_str = rel_path.to_string_lossy().to_string();
                if rel_str != "onpkg.json" && !rel_str.contains("onpkg_docs/") {
                    files.push(rel_str);
                    extensions.insert(ext.to_string());
                    if let Some(parent) = rel_path.parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if !parent_str.is_empty() {
                            dirs.insert(parent_str);
                        }
                    }
                }
            }
        }
    }

    let mut file_patterns: Vec<String> = extensions
        .into_iter()
        .map(|ext| format!("*.{}", ext))
        .collect();
    file_patterns.sort();
    let mut directories: Vec<String> = dirs.into_iter().collect();
    directories.sort();
    files.sort();

    // 5. Detect architecture components
    let mut architecture = std::collections::BTreeMap::new();

    // Entrypoints
    let entrypoint_patterns = [
        "src/main.rs",
        "src/main.tsx",
        "src/main.ts",
        "src/index.js",
        "src/index.tsx",
        "src/index.ts",
        "lib/main.dart",
        "app/main.py",
        "src/main.py",
        "main.py",
    ];
    for ep in &entrypoint_patterns {
        if target_dir.join(ep).exists() {
            architecture.insert("entrypoint".to_string(), ep.to_string());
            break;
        }
    }

    // Routing directories
    let routing_patterns = [
        "src/routes",
        "src/pages",
        "app/routes",
        "app",
        "src/app",
        "routes",
        "pages",
    ];
    for r in &routing_patterns {
        if target_dir.join(r).exists() {
            architecture.insert("routing".to_string(), r.to_string());
            break;
        }
    }

    // Components
    let components_patterns = [
        "src/components",
        "components",
        "src/ui",
        "src/components/ui",
        "ui",
    ];
    for c in &components_patterns {
        if target_dir.join(c).exists() {
            architecture.insert("components".to_string(), c.to_string());
            break;
        }
    }

    // Styles
    let styles_patterns = [
        "src/index.css",
        "src/App.css",
        "src/styles.css",
        "styles",
        "src/global.css",
        "src/styles/global.css",
    ];
    for s in &styles_patterns {
        if target_dir.join(s).exists() {
            architecture.insert("styles".to_string(), s.to_string());
            break;
        }
    }

    // Database
    let db_patterns = [
        "prisma/schema.prisma",
        "src/db",
        "db",
        "schema.sql",
        "migrations",
        "src/database",
    ];
    for d in &db_patterns {
        if target_dir.join(d).exists() {
            architecture.insert("database".to_string(), d.to_string());
            break;
        }
    }

    // Tests
    let tests_patterns = ["src/tests", "tests", "test", "src/test"];
    for t in &tests_patterns {
        if target_dir.join(t).exists() {
            architecture.insert("tests".to_string(), t.to_string());
            break;
        }
    }

    // 6. Collect scripts and packages
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
                if let Some(dev_deps) = v.get("devDependencies").and_then(|d| d.as_object()) {
                    for (k, _) in dev_deps {
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
                if let Some(dev_deps) = v.get("dev-dependencies").and_then(|d| d.as_table()) {
                    for (k, _) in dev_deps {
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
                                let name = dep_str
                                    .split(&['>', '=', '<', '~', '!'][..])
                                    .next()
                                    .unwrap_or(dep_str)
                                    .trim();
                                core_packages.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        scripts.insert(
            "dev".to_string(),
            "uvicorn app.main:app --reload".to_string(),
        );
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

    // 7. Collect active skills from onpkg_docs/
    let docs_dir = target_dir.join("onpkg_docs");
    let mut active_skills = Vec::new();

    // If technologies were supplied during creation, make sure we pre-populate skills list
    if let Some(techs) = technologies {
        for tech in techs {
            active_skills.push(format!("{}.md", tech));
        }
    }

    if docs_dir.exists() {
        if let Ok(entries) = fs::read_dir(&docs_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if ext == "md" {
                            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                                // Exclude INDEX and workflow files
                                if name != "INDEX.md"
                                    && name != "prd.md"
                                    && name != "content.md"
                                    && name != "design.md"
                                    && name != "implementation.md"
                                    && name != "todo.md"
                                {
                                    active_skills.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    active_skills.sort();
    active_skills.dedup();

    // Preserve stack name
    let final_stack = if let Some(sn) = stack_name {
        sn.to_string()
    } else if let Some(ref m) = existing_manifest {
        m.get("project")
            .and_then(|p| p.get("stack"))
            .and_then(|s| s.as_str())
            .unwrap_or("custom")
            .to_string()
    } else {
        "custom".to_string()
    };

    // Preserve added_by_agent packages
    let mut added_by_agent = Vec::new();
    if let Some(ref m) = existing_manifest {
        if let Some(arr) = m
            .get("packages")
            .and_then(|p| p.get("added_by_agent"))
            .and_then(|a| a.as_array())
        {
            for v in arr {
                if let Some(s) = v.as_str() {
                    added_by_agent.push(s.to_string());
                }
            }
        }
    }

    // Build the manifest JSON
    let mut manifest = std::collections::BTreeMap::new();

    let mut project_info = std::collections::BTreeMap::new();
    project_info.insert("name".to_string(), serde_json::Value::String(project_name));
    project_info.insert("stack".to_string(), serde_json::Value::String(final_stack));
    project_info.insert("runtime".to_string(), serde_json::Value::String(runtime));
    project_info.insert(
        "package_manager".to_string(),
        serde_json::Value::String(package_manager),
    );

    // Merge existing variables and new variables
    let mut vars_map = std::collections::BTreeMap::new();
    if let Some(existing_vars) = existing_manifest
        .as_ref()
        .and_then(|m| m.get("project"))
        .and_then(|p| p.get("variables"))
        .and_then(|v| v.as_object())
    {
        for (k, val) in existing_vars {
            if let Some(s) = val.as_str() {
                vars_map.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(new_vars) = variables {
        for (k, val) in new_vars {
            vars_map.insert(k.clone(), val.clone());
        }
    }
    if !vars_map.is_empty() {
        let vars_json_obj = serde_json::Value::Object(
            vars_map
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
        );
        project_info.insert("variables".to_string(), vars_json_obj);
    }

    manifest.insert(
        "project".to_string(),
        serde_json::Value::Object(project_info.into_iter().collect()),
    );

    let arch_info: serde_json::Map<String, serde_json::Value> = architecture
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    manifest.insert(
        "architecture".to_string(),
        serde_json::Value::Object(arch_info),
    );

    let mut agent_info = std::collections::BTreeMap::new();
    agent_info.insert(
        "docs_directory".to_string(),
        serde_json::Value::String("onpkg_docs/".to_string()),
    );
    agent_info.insert(
        "active_skills".to_string(),
        serde_json::Value::Array(
            active_skills
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
    );
    manifest.insert(
        "agent_instructions".to_string(),
        serde_json::Value::Object(agent_info.into_iter().collect()),
    );

    let scripts_info: serde_json::Map<String, serde_json::Value> = scripts
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    manifest.insert(
        "scripts".to_string(),
        serde_json::Value::Object(scripts_info),
    );

    let mut packages_info = std::collections::BTreeMap::new();
    core_packages.sort();
    core_packages.dedup();
    packages_info.insert(
        "core".to_string(),
        serde_json::Value::Array(
            core_packages
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    packages_info.insert(
        "added_by_agent".to_string(),
        serde_json::Value::Array(
            added_by_agent
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    manifest.insert(
        "packages".to_string(),
        serde_json::Value::Object(packages_info.into_iter().collect()),
    );

    // Add sources block for tree-sitter & code analysis!
    let mut sources_info = std::collections::BTreeMap::new();
    sources_info.insert(
        "directories".to_string(),
        serde_json::Value::Array(
            directories
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    sources_info.insert(
        "files".to_string(),
        serde_json::Value::Array(files.into_iter().map(serde_json::Value::String).collect()),
    );
    sources_info.insert(
        "file_patterns".to_string(),
        serde_json::Value::Array(
            file_patterns
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ),
    );
    manifest.insert(
        "sources".to_string(),
        serde_json::Value::Object(sources_info.into_iter().collect()),
    );

    let mut workspaces = Vec::new();
    let cargo_toml_path = target_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(cargo_content) = fs::read_to_string(&cargo_toml_path) {
            if cargo_content.contains("[workspace]") {
                workspaces.push("cargo-workspace".to_string());
            }
        }
    }
    if target_dir.join("pnpm-workspace.yaml").exists() {
        workspaces.push("pnpm-workspace".to_string());
    }
    let package_json_path = target_dir.join("package.json");
    if package_json_path.exists() {
        if let Ok(package_content) = fs::read_to_string(&package_json_path) {
            if package_content.contains("\"workspaces\"") {
                workspaces.push("npm-yarn-workspace".to_string());
            }
        }
    }
    if target_dir.join("apps").is_dir() && target_dir.join("packages").is_dir() {
        workspaces.push("turborepo-workspace".to_string());
    }
    if !workspaces.is_empty() {
        manifest.insert(
            "workspaces".to_string(),
            serde_json::Value::Array(
                workspaces
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            ),
        );
    }

    // Write to target_dir/onpkg.json
    if let Ok(json_str) = serde_json::to_string_pretty(&manifest) {
        fs::write(&onpkg_path, json_str)?;
        println!("  synchronized: AI Agent Project Manifest in onpkg.json");
    }

    // 8. Ensure docs directory exists and create/update workflow docs
    fs::create_dir_all(&docs_dir)?;

    let prd_path = docs_dir.join("prd.md");
    if !prd_path.exists() {
        let prd_template = r#"# Product Requirements Document (PRD) 🚀

## Project Overview
*Brief summary of the project, target audience, and main goals.*

## Core Features & Scope
- [ ] **Feature 1**: Description and user value.
- [ ] **Feature 2**: Description and user value.

## Success Metrics
- Performance: Sleek UI, fast load times.
- Quality: Visual excellence and rich aesthetics.

## Future Scope (Out of Scope)
*Features that are deferred to later versions.*
"#;
        fs::write(prd_path, prd_template)?;
        println!("  created: Product Requirements Document in onpkg_docs/prd.md");
    }

    let content_path = docs_dir.join("content.md");
    if !content_path.exists() {
        let content_template = r#"# Content & Page Inventory 📝

## Page Structure
- **Home Page** (`/`): Brief description, hero section, features list.
- **About Page** (`/about`): Story, team, value proposition.

## Copywriting & Tone of Voice
- **Tone**: Professional, modern, encouraging.
- **Key Terminology**: *onpkg*, *AI Agent*, *Stack Scaffolding*.

## Dynamic Assets
- Logos, icons, and illustrations used across the pages.
"#;
        fs::write(content_path, content_template)?;
        println!("  created: Content Inventory in onpkg_docs/content.md");
    }

    let design_path = docs_dir.join("design.md");
    if !design_path.exists() {
        let design_template = r#"# UI & Design System 🎨

## Aesthetics & Theme
- Sleek modern dark mode (e.g. HSL tailored color palette).
- Smooth glassmorphism, dynamic gradients, micro-animations.

## Color Palette (CSS Variables)
```css
:root {
  --background: 240 10% 3.9%;
  --foreground: 0 0% 98%;
  --primary: 263.4 90% 50.4%; /* Neon Violet */
  --accent: 180 100% 50%;     /* Neon Cyan */
  --card: 240 10% 10%;
  --border: 240 5.9% 15%;
}
```

## Typography
- Main Font: `Inter` or `Outfit` via Google Fonts.
- Browser default sans-serif as fallback.

## Key UI Components
- **Navbar**: Floating with blur filter (`backdrop-filter: blur(12px)`).
- **Cards**: Glassmorphic borders with linear gradient.
- **Buttons**: Hover glow and micro-zoom effects.
"#;
        fs::write(design_path, design_template)?;
        println!("  created: Design & UI Specification in onpkg_docs/design.md");
    }

    let implementation_path = docs_dir.join("implementation.md");
    if !implementation_path.exists() {
        let implementation_template = r#"# Technical Implementation Plan 🛠️

## Technology Stack
*Active technologies and framework choices.*

## System Architecture & File Layout
*Key directories and entrypoints.*

## API Endpoints & Data Models
*Routes and schema definitions.*

## Key Tasks & File Modifications
*Where code changes occur.*
"#;
        fs::write(implementation_path, implementation_template)?;
        println!("  created: Technical Implementation Plan in onpkg_docs/implementation.md");
    }

    let todo_path = docs_dir.join("todo.md");
    if !todo_path.exists() {
        let todo_template = r#"# Task Tracker (todo.md) 📋

## Setup & Scaffolding
- [x] Initialize project structure with `onpkg`
- [x] Configure tailwind/css modules
- [x] Set up entrypoint and basic router

## Active Work Streams
- [ ] Build core layouts and design system (design.css)
- [ ] Implement pages and navigation
- [ ] Integrate database/storage
- [ ] Add animations and polish transitions

## Verification & Testing
- [ ] Run lint checks and build validation
- [ ] Audit responsive layout on mobile/desktop
"#;
        fs::write(todo_path, todo_template)?;
        println!("  created: Task Tracker checklist in onpkg_docs/todo.md");
    }

    // 9. Re-generate INDEX.md with links to skills and workflow files
    let mut index_content = r#"# Project AI Agent Skills 🧠

This directory contains instructions and guidelines for AI agents working on this project.

## Project Workflow 📋
Use these documents to manage project progress, feature requests, and design alignment:
- [Product Requirements (prd.md)](file://./prd.md)
- [Content & Pages (content.md)](file://./content.md)
- [UI & Design Tokens (design.md)](file://./design.md)
- [Technical Implementation (implementation.md)](file://./implementation.md)
- [Task Tracker (todo.md)](file://./todo.md)

## Technology Skills 🛠️
"#
    .to_string();

    if active_skills.is_empty() {
        index_content.push_str("No technology skills installed yet.\n");
    } else {
        for skill_file in &active_skills {
            let skill_name = skill_file.strip_suffix(".md").unwrap_or(skill_file);
            index_content.push_str(&format!(
                "- [{skill_name}](file://./{skill_file}) / [{skill_name}/skill.md](file://./{skill_name}/skill.md)\n"
            ));
        }
    }

    fs::write(docs_dir.join("INDEX.md"), index_content)?;
    println!("  updated: AI Docs Index in onpkg_docs/INDEX.md");

    // 10. Generate AGENTS.md for universal AI agent context
    let project_name_for_agents = manifest
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("my-project")
        .to_string();
    let runtime_for_agents = manifest
        .get("project")
        .and_then(|p| p.get("runtime"))
        .and_then(|n| n.as_str())
        .unwrap_or("node")
        .to_string();
    let pm_for_agents = manifest
        .get("project")
        .and_then(|p| p.get("package_manager"))
        .and_then(|n| n.as_str())
        .unwrap_or("npm")
        .to_string();
    let mut arch_for_agents = std::collections::BTreeMap::new();
    if let Some(arch_obj) = manifest.get("architecture").and_then(|a| a.as_object()) {
        for (k, v) in arch_obj {
            if let Some(s) = v.as_str() {
                arch_for_agents.insert(k.clone(), s.to_string());
            }
        }
    }
    let mut scripts_for_agents = std::collections::BTreeMap::new();
    if let Some(scripts_obj) = manifest.get("scripts").and_then(|s| s.as_object()) {
        for (k, v) in scripts_obj {
            if let Some(s) = v.as_str() {
                scripts_for_agents.insert(k.clone(), s.to_string());
            }
        }
    }
    if !no_agents_md {
        generate_agents_md(
            target_dir,
            &project_name_for_agents,
            &runtime_for_agents,
            &pm_for_agents,
            &arch_for_agents,
            &scripts_for_agents,
            &active_skills,
        )?;

        if symlink_claude {
            create_claude_md_symlink(target_dir)?;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_template_definition_with_hooks() {
        let toml_str = r#"
name = "test-stack"
category = "test"
description = "A test stack with hooks"
version = "1.0.0"
variables = []

[[files]]
path = "index.js"
content = "console.log('hello');"

[[hooks]]
command = "git init"
description = "Initialize git repository"
"#;
        let tmpl: TemplateDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(tmpl.name, "test-stack");
        assert_eq!(tmpl.hooks.len(), 1);
        assert_eq!(tmpl.hooks[0].command, "git init");
        assert_eq!(tmpl.hooks[0].description.as_deref(), Some("Initialize git repository"));
    }

    #[test]
    fn test_stack_to_template_definition_conversion_with_hooks() {
        let stack = crate::stacks::Stack {
            name: "test-stack".to_string(),
            runtime: "bun".to_string(),
            description: "Test".to_string(),
            packages: vec![],
            dev_packages: vec![],
            transitive_packages: vec![],
            files: vec![],
            hooks: vec![
                crate::stacks::StackHook {
                    command: "echo hello".to_string(),
                    description: Some("Say hello".to_string()),
                }
            ],
        };

        let tmpl: TemplateDefinition = stack.into();
        assert_eq!(tmpl.hooks.len(), 1);
        assert_eq!(tmpl.hooks[0].command, "echo hello");
        assert_eq!(tmpl.hooks[0].description.as_deref(), Some("Say hello"));
    }

    #[test]
    fn test_sync_project_with_no_agents_md_and_symlink_claude() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().to_path_buf();

        let onpkg_json = serde_json::json!({
            "project": {
                "name": "symlink-test",
                "runtime": "rust",
                "package_manager": "cargo"
            }
        });
        std::fs::write(
            project_dir.join("onpkg.json"),
            serde_json::to_string(&onpkg_json).unwrap()
        ).unwrap();

        sync_onpkg_project(&project_dir, None, None, None, true, false).unwrap();
        assert!(!project_dir.join("AGENTS.md").exists());

        sync_onpkg_project(&project_dir, None, None, None, false, true).unwrap();
        assert!(project_dir.join("AGENTS.md").exists());
        let claude_path = project_dir.join("CLAUDE.md");
        assert!(claude_path.exists());

        let content = std::fs::read_to_string(&claude_path).unwrap();
        let expected_name = project_dir.file_name().unwrap().to_str().unwrap();
        assert!(content.contains(expected_name));
        assert!(content.contains("npm"));
    }
}
