pub mod ai;
pub mod cli;
pub mod config;
pub mod db;
pub mod pkg_registry;
pub mod registry;
pub mod skill;
pub mod stacks;
pub mod templates;
pub mod tui;

use crate::cli::{
    Args, Command, PkgSubcommand, RegistrySubcommand, SkillSubcommand, TemplateSubcommand,
    StackSubcommand, AiSubcommand,
};
use crate::ai::AiGenerator;
use crate::config::Config;
use crate::db::Database;
use crate::pkg_registry::PkgRegistry;
use crate::registry::Registry;
use crate::skill::SkillManager;
use crate::templates::TemplateEngine;
use crate::tui::TUI;
use anyhow::{anyhow, Context, Result};
use clap::{CommandFactory, Parser};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();
    let config = Config::load().context("Failed to load config")?;
    let _tui = TUI::new();
    let db = Database::open(&config).context("Failed to open database")?;
    let registry = Registry::new(config.clone());
    let pkg_registry = PkgRegistry::new();
    let template_engine = TemplateEngine::new(config.clone());
    let skill_manager = SkillManager::new(config.clone(), db.clone());

    if cli.version {
        TUI::logo();
        return Ok(());
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            TUI::logo();
            Args::command().print_help()?;
            println!();
            return Ok(());
        }
    };

    match command {
        Command::Init { name, template } => {
            TUI::logo();

            let dir = if let Some(ref n) = name {
                std::fs::create_dir_all(n)?;
                std::path::PathBuf::from(n)
            } else {
                std::env::current_dir()?
            };

            let project_config = serde_json::json!({
                "name": name.clone().unwrap_or_else(|| "my-project".to_string()),
                "version": "0.1.0",
                "onpkg": {
                    "templates": [],
                    "skills": [],
                    "packages": []
                }
            });
            let onpkg_path = dir.join("onpkg.json");
            if !onpkg_path.exists() {
                std::fs::write(
                    &onpkg_path,
                    serde_json::to_string_pretty(&project_config)?,
                )?;
                TUI::success("Initialized onpkg project", Some("onpkg.json"));
            } else {
                TUI::info("Project already initialized (onpkg.json exists)");
            }

            if let Some(ref tmpl_name) = template {
                let tmpl = template_engine
                    .find(tmpl_name)
                    .ok_or_else(|| anyhow!("Template '{}' not found. Run: onpkg template list", tmpl_name))?;

                let sp = TUI::spinner(&format!("Scaffolding {}...", tmpl.name));
                let created = template_engine.scaffold(&tmpl, &dir, &HashMap::new())?;
                sp.finish_and_clear();

                TUI::success(
                    &format!("Template '{}' scaffolded", tmpl.name),
                    Some(&format!("{} files created", created.len())),
                );
                for f in created {
                    TUI::label("  created", &f);
                }
            }
        }

        Command::Template { subcmd } => match subcmd {
            TemplateSubcommand::List { category } => {
                TUI::logo();
                let templates = template_engine.all_templates();
                TUI::info(&format!("{} templates available:", templates.len()));
                println!();

                for t in templates {
                    if let Some(ref cat) = category {
                        if &t.category != cat {
                            continue;
                        }
                    }
                    TUI::success(
                        &t.name,
                        Some(&format!(
                            "[{}] {} \u{00b7} {} files",
                            t.category,
                            t.description,
                            t.files.len()
                        )),
                    );
                }
            }

            TemplateSubcommand::Show { name } => {
                let tmpl = template_engine
                    .find(&name)
                    .ok_or_else(|| anyhow!("Template '{}' not found. Run: onpkg template list", name))?;

                TUI::label("name", &tmpl.name);
                TUI::label("category", &tmpl.category);
                TUI::label("description", &tmpl.description);
                TUI::label("version", &tmpl.version);
                println!();

                if !tmpl.files.is_empty() {
                    TUI::info("files:");
                    for f in &tmpl.files {
                        println!("  {}", f.path);
                    }
                }

                if !tmpl.variables.is_empty() {
                    println!();
                    TUI::info("variables:");
                    for v in &tmpl.variables {
                        println!("  {} = {}  ({})", v.name, v.default, v.description);
                    }
                }
            }

            TemplateSubcommand::Use { name, dir, var } => {
                let tmpl = template_engine
                    .find(&name)
                    .ok_or_else(|| anyhow!("Template '{}' not found. Run: onpkg template list", name))?;

                let target = if let Some(ref d) = dir {
                    std::path::PathBuf::from(d)
                } else {
                    std::env::current_dir()?
                };

                let mut extra_vars = HashMap::new();
                for v in &var {
                    if let Some(pos) = v.find('=') {
                        let key = &v[..pos];
                        let val = &v[pos + 1..];
                        extra_vars.insert(key.to_string(), val.to_string());
                    }
                }

                let sp = TUI::spinner(&format!("Scaffolding {}...", tmpl.name));
                let created = template_engine.scaffold(&tmpl, &target, &extra_vars)?;
                sp.finish_and_clear();

                TUI::success(
                    &format!("Template '{}' scaffolded", tmpl.name),
                    Some(&format!("{} files created", created.len())),
                );
                for f in created {
                    TUI::label("  created", &f);
                }
            }

            TemplateSubcommand::Add { name, source } => {
                let source_path = std::path::Path::new(&source);
                if source_path.exists() {
                    let tmpl = template_engine.add_from_dir(&name, source_path)?;
                    TUI::success(
                        &format!("Template '{}' added", name),
                        Some(&format!("{} files", tmpl.files.len())),
                    );
                } else {
                    let sp = TUI::spinner(&format!("Cloning {}...", source));
                    let tmp_dir = std::env::temp_dir().join("onpkg_clone");
                    let _ = std::fs::remove_dir_all(&tmp_dir);

                    match git2::Repository::clone(&source, &tmp_dir) {
                        Ok(_) => {
                            sp.finish_and_clear();
                            let tmpl = template_engine.add_from_dir(&name, &tmp_dir)?;
                            TUI::success(
                                &format!("Template '{}' added from git", name),
                                Some(&format!("{} files", tmpl.files.len())),
                            );
                        }
                        Err(e) => {
                            sp.finish_and_clear();
                            return Err(anyhow!("Failed to clone '{}': {}", source, e));
                        }
                    }
                }
            }

            TemplateSubcommand::Remove { name } => {
                template_engine.remove(&name)?;
                TUI::success(&format!("Template '{}' removed", name), None);
            }

            TemplateSubcommand::Publish { name } => {
                let tmpl_path = config.templates_dir().join(format!("{}.toml", name));
                if !tmpl_path.exists() {
                    return Err(anyhow!(
                        "Template '{}' not found locally. Only custom templates can be published.",
                        name
                    ));
                }
                match registry.publish_template(&name, &tmpl_path).await {
                    Ok(_) => TUI::success(&format!("Template '{}' published", name), None),
                    Err(e) => TUI::warn(&e.to_string()),
                }
            }
        },

        Command::Skill { subcmd } => match subcmd {
            SkillSubcommand::List => {
                TUI::logo();
                let skills = skill_manager.list()?;
                TUI::info(&format!("{} skills installed:", skills.len()));
                println!();
                for s in skills {
                    TUI::success(&s.name, Some(&format!("v{} \u{00b7} {}", s.version, s.description)));
                }
            }

            SkillSubcommand::Install { name } => {
                let sp = TUI::spinner(&format!("Installing skill '{}'...", name));
                skill_manager.install(&name)?;
                sp.finish_and_clear();
                TUI::success(&format!("Skill '{}' installed", name), None);
                TUI::info(&format!("Path: ~/.onpkg/skills/{}.md", name));
            }

            SkillSubcommand::Show { name } => {
                let skill = skill_manager
                    .show(&name)?
                    .ok_or_else(|| anyhow!("Skill '{}' not found", name))?;
                TUI::label("name", &skill.name);
                TUI::label("description", &skill.description);
                TUI::label("version", &skill.version);
                TUI::label("source", &skill.source);
                TUI::label("path", &skill.path);
            }

            SkillSubcommand::Remove { name } => {
                skill_manager.remove(&name)?;
                TUI::success(&format!("Skill '{}' removed", name), None);
            }

            SkillSubcommand::Add { name, source } => {
                skill_manager.add_from_path(&name, &source)?;
                TUI::success(&format!("Skill '{}' added", name), None);
            }

            SkillSubcommand::Publish { name } => {
                let skill_path = config.skills_dir().join(format!("{}.md", name));
                if !skill_path.exists() {
                    return Err(anyhow!("Skill '{}' not found locally", name));
                }
                match registry.publish_skill(&name, &skill_path).await {
                    Ok(_) => TUI::success(&format!("Skill '{}' published", name), None),
                    Err(e) => TUI::warn(&e.to_string()),
                }
            }
        },

        Command::Pkg { subcmd } => match subcmd {
            PkgSubcommand::List { runtime } => {
                TUI::logo();
                let pkgs = db.list_packages(runtime.as_deref())?;
                TUI::info(&format!("{} packages cached:", pkgs.len()));
                println!();
                for pkg in pkgs {
                    TUI::success(
                        &format!("{}@{} ({})", pkg.name, pkg.version, pkg.runtime),
                        Some(&pkg.cache_path),
                    );
                }
            }

            PkgSubcommand::Install { name, runtime } => {
                let rt = runtime.clone().unwrap_or_else(|| detect_runtime(&name));
                let sp = TUI::spinner(&format!("Fetching {} ({})...", name, rt));
                let info = pkg_registry.fetch_info(&name, &rt).await;
                sp.finish_and_clear();

                match info {
                    Ok(pkg_info) => {
                        TUI::success(
                            &format!("{}@{} ({})", pkg_info.name, pkg_info.version, pkg_info.runtime),
                            Some(&pkg_info.description),
                        );

                        // Cache in database
                        let cache_path = config
                            .cache_path()
                            .join(pkg_info.runtime.as_str())
                            .join(format!("{}-{}", pkg_info.name, pkg_info.version));
                        let pkg = db::Package {
                            id: 0,
                            name: pkg_info.name.clone(),
                            version: pkg_info.version.clone(),
                            runtime: pkg_info.runtime.clone(),
                            r#type: "pkg".to_string(),
                            source: format!("{}:{}", pkg_info.runtime, pkg_info.name),
                            cache_path: cache_path.to_string_lossy().to_string(),
                            checksum: String::new(),
                            size_bytes: None,
                            cached_at: chrono::Utc::now().to_rfc3339(),
                        };
                        db.insert_package(&pkg)?;
                        TUI::info(&format!("Cached at ~/.onpkg/cache/{}/", pkg_info.runtime));

                        if let Some(h) = pkg_info.homepage {
                            TUI::label("homepage", &h);
                        }
                        if let Some(l) = pkg_info.license {
                            TUI::label("license", &l);
                        }
                    }
                    Err(e) => {
                        TUI::warn(&format!("Could not fetch '{}': {}", name, e));
                        TUI::info("Use npm/pip/pub/cargo directly for now.");
                    }
                }
            }

            PkgSubcommand::Add { name, runtime } => {
                let rt = runtime.clone().unwrap_or_else(|| detect_runtime(&name));
                let sp = TUI::spinner(&format!("Looking up {} ({})...", name, rt));
                let info = pkg_registry.fetch_info(&name, &rt).await;
                sp.finish_and_clear();

                match info {
                    Ok(pkg_info) => {
                        let project_dir = std::env::current_dir()?;
                        match pkg_registry.add_to_project(
                            &pkg_info.name,
                            &pkg_info.version,
                            &pkg_info.runtime,
                            &project_dir,
                        ) {
                            Ok(_) => {
                                TUI::success(
                                    &format!("Added {}@{}", pkg_info.name, pkg_info.version),
                                    Some(&format!("to project ({})", pkg_info.runtime)),
                                );
                            }
                            Err(e) => {
                                TUI::warn(&format!("{}", e));
                            }
                        }
                    }
                    Err(e) => {
                        TUI::warn(&format!("Could not find '{}': {}", name, e));
                        TUI::info("Use npm/pip/pub/cargo directly for now.");
                    }
                }
            }

            PkgSubcommand::Remove { name, runtime } => {
                let rt = runtime.unwrap_or_else(|| "npm".to_string());
                db.delete_package(&name, &rt)?;
                TUI::success(&format!("Package '{}' removed from cache", name), None);
            }
        },

        Command::Registry { subcmd } => match subcmd {
            RegistrySubcommand::Search { query, r#type } => {
                let sp = TUI::spinner(&format!("Searching registry for '{}'...", query));
                let results = registry.search(&query, r#type.as_deref()).await?;
                sp.finish_and_clear();

                if !results.templates.is_empty() {
                    TUI::info(&format!("{} templates found:", results.templates.len()));
                    for t in &results.templates {
                        TUI::success(&t.name, Some(&format!("v{} \u{00b7} {}", t.version, t.description)));
                    }
                    println!();
                }
                if !results.skills.is_empty() {
                    TUI::info(&format!("{} skills found:", results.skills.len()));
                    for s in &results.skills {
                        TUI::success(&s.name, Some(&format!("v{} \u{00b7} {}", s.version, s.description)));
                    }
                    println!();
                }
                if !results.packages.is_empty() {
                    TUI::info(&format!("{} packages found:", results.packages.len()));
                    for p in &results.packages {
                        TUI::success(&p.name, Some(&format!("{}  v{}", p.runtime, p.version)));
                    }
                }

                if results.templates.is_empty()
                    && results.skills.is_empty()
                    && results.packages.is_empty()
                {
                    TUI::warn("No results found. The registry may be offline.");
                }
            }

            RegistrySubcommand::Info { name } => {
                let sp = TUI::spinner(&format!("Fetching info for '{}'...", name));
                match registry.get_template(&name).await {
                    Ok(t) => {
                        sp.finish_and_clear();
                        TUI::label("name", &t.name);
                        TUI::label("category", &t.category);
                        TUI::label("description", &t.description);
                        TUI::label("version", &t.version);
                        TUI::label("author", &t.author);
                        TUI::label("source", &t.source);
                    }
                    Err(_) => match registry.get_skill(&name).await {
                        Ok(s) => {
                            sp.finish_and_clear();
                            TUI::label("name", &s.name);
                            TUI::label("description", &s.description);
                            TUI::label("version", &s.version);
                            TUI::label("author", &s.author);
                            TUI::label("source", &s.source);
                        }
                        Err(e) => {
                            sp.finish_and_clear();
                            TUI::warn(&format!("'{}' not found in registry: {}", name, e));
                        }
                    },
                }
            }

            RegistrySubcommand::Config { url } => {
                if let Some(new_url) = url {
                    let mut config = Config::load()?;
                    config.registry.url = new_url.clone();
                    config.save()?;
                    TUI::success(&format!("Registry URL set to {}", new_url), None);
                } else {
                    TUI::label("registry URL", &config.registry_url());
                }
            }
        },

        Command::Doctor => {
            TUI::logo();
            TUI::info("Running diagnostics...");
            println!();

            TUI::success("config", Some("~/.onpkg/config.toml"));

            match db.count_packages() {
                Ok(count) => TUI::success("database", Some(&format!("{} packages cached", count))),
                Err(e) => TUI::error(&format!("database: {}", e)),
            }

            let tmpl_count = template_engine.all_templates().len();
            TUI::success("templates", Some(&format!("{} available", tmpl_count)));

            match skill_manager.list() {
                Ok(skills) => TUI::success("skills", Some(&format!("{} installed", skills.len()))),
                Err(e) => TUI::error(&format!("skills: {}", e)),
            }

            match registry.check_health().await {
                Ok(status) => {
                    let s = status.get("status").map(|s| s.as_str()).unwrap_or("unknown");
                    TUI::success("registry", Some(s));
                }
                Err(e) => TUI::warn(&format!("registry: {}", e)),
            }

            for (cmd, name) in &[("node", "Node.js"), ("bun", "Bun"), ("python3", "Python 3"), ("cargo", "Cargo")] {
                match std::process::Command::new(cmd).arg("--version").output() {
                    Ok(out) => {
                        let ver = String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .next()
                            .unwrap_or("unknown")
                            .to_string();
                        TUI::success(name, Some(&ver));
                    }
                    Err(_) => TUI::warn(&format!("{} not found on PATH", name)),
                }
            }

            println!();
            TUI::success("Doctor complete", None);
        }

        Command::Stack { subcmd } => match subcmd {
            StackSubcommand::List { category } => {
                TUI::logo();
                let templates = template_engine.all_templates();
                TUI::info(&format!("{} stacks available:", templates.len()));
                println!();

                for t in templates {
                    if let Some(ref cat) = category {
                        if &t.category != cat {
                            continue;
                        }
                    }
                    TUI::success(
                        &t.name,
                        Some(&format!(
                            "[{}] {} \u{00b7} {} files",
                            t.category,
                            t.description,
                            t.files.len()
                        )),
                    );
                }
            }

            StackSubcommand::Show { name } => {
                let tmpl = find_template_fuzzy(&template_engine, &name)
                    .ok_or_else(|| anyhow!("Stack '{}' not found. Run: onpkg stack list", name))?;

                TUI::label("name", &tmpl.name);
                TUI::label("category", &tmpl.category);
                TUI::label("description", &tmpl.description);
                TUI::label("version", &tmpl.version);
                TUI::label("technologies", &tmpl.get_technologies().join(", "));
                println!();

                if !tmpl.files.is_empty() {
                    TUI::info("files:");
                    for f in &tmpl.files {
                        println!("  {}", f.path);
                    }
                }

                if !tmpl.variables.is_empty() {
                    println!();
                    TUI::info("variables:");
                    for v in &tmpl.variables {
                        println!("  {} = {}  ({})", v.name, v.default, v.description);
                    }
                }
            }

            StackSubcommand::Add { name, dir, var, manager } | StackSubcommand::Use { name, dir, var, manager } => {
                let tmpl = find_template_fuzzy(&template_engine, &name)
                    .ok_or_else(|| anyhow!("Stack '{}' not found. Run: onpkg stack list", name))?;

                let target = if let Some(ref d) = dir {
                    std::path::PathBuf::from(d)
                } else {
                    std::env::current_dir()?
                };

                let mut extra_vars = HashMap::new();
                for v in &var {
                    if let Some(pos) = v.find('=') {
                        let key = &v[..pos];
                        let val = &v[pos + 1..];
                        extra_vars.insert(key.to_string(), val.to_string());
                    }
                }

                let sp = TUI::spinner(&format!("Scaffolding stack {}...", tmpl.name));
                let created = template_engine.scaffold(&tmpl, &target, &extra_vars)?;
                sp.finish_and_clear();

                TUI::success(
                    &format!("Stack '{}' scaffolded", tmpl.name),
                    Some(&format!("{} files created", created.len())),
                );
                for f in &created {
                    TUI::label("  created", f);
                }

                // Install dependencies online
                println!();
                let install_sp = TUI::spinner("Installing dependencies from the internet...");
                let install_res = templates::install_dependencies_online(&target, manager.as_deref());
                install_sp.finish_and_clear();
                if let Err(e) = install_res {
                    TUI::warn(&format!("Dependency installation failed: {}", e));
                }

                // Generate agent documentation (onpkg_docs/)
                println!();
                let docs_sp = TUI::spinner("Generating AI agent skills in onpkg_docs/...");
                let techs = tmpl.get_technologies();
                let docs_res = templates::generate_agent_docs(&techs, &target, &config);
                docs_sp.finish_and_clear();
                if let Err(e) = docs_res {
                    TUI::warn(&format!("Failed to generate onpkg_docs/: {}", e));
                } else {
                    TUI::success("AI agent skills created under onpkg_docs/", None);
                    TUI::info("Agents can read onpkg_docs/INDEX.md to get started.");
                    
                    // Generate AI Agent Manifest onpkg.json
                    if let Err(e) = templates::generate_onpkg_manifest(&tmpl, &target, &techs) {
                        TUI::warn(&format!("Failed to generate onpkg.json: {}", e));
                    } else {
                        TUI::success("AI agent manifest created in onpkg.json", None);
                    }
                }
            }

            StackSubcommand::New { name, category } => {
                let custom_dir = config.templates_dir();
                std::fs::create_dir_all(&custom_dir)?;
                let path = custom_dir.join(format!("{}.toml", name));
                if path.exists() {
                    return Err(anyhow!("Custom stack template '{}' already exists.", name));
                }

                let template_toml = format!(
                    r#"name = "{name}"
category = "{category}"
description = "My custom stack template"
version = "1.0.0"

[[variables]]
name = "project_name"
description = "Name of the project"
default = "my-app"

[[files]]
path = "package.json"
content = """{{
  "name": "{{{{ project_name }}}}",
  "version": "1.0.0",
  "dependencies": {{}}
}}"""
"#,
                    name = name,
                    category = category
                );

                std::fs::write(&path, template_toml)?;
                TUI::success(&format!("Created new stack definition for '{}'", name), Some(&path.to_string_lossy()));
                TUI::info("Edit this TOML to customize files, packages, and technologies.");
            }
        }

        Command::Ai { subcmd } => match subcmd {
            AiSubcommand::Skill { name, prompt } => {
                TUI::info(&format!("Generating AI skill for '{}' using Gemini...", name));
                let ai = AiGenerator::new()?;
                let sp = TUI::spinner("Thinking...");
                let content = ai.generate_skill(&name, prompt.as_deref()).await?;
                sp.finish_and_clear();

                let path = config.skills_dir().join(format!("{}.md", name));
                std::fs::create_dir_all(config.skills_dir())?;
                std::fs::write(&path, &content)?;

                // Register skill in database
                skill_manager.install_from_path(&name, &path)?;

                TUI::success(&format!("Generated and installed skill '{}'", name), Some(&path.to_string_lossy()));
            }

            AiSubcommand::Template { name, description } => {
                TUI::info(&format!("Generating custom template '{}' using Gemini...", name));
                let ai = AiGenerator::new()?;
                let sp = TUI::spinner("Designing stack and TOML config...");
                let content = ai.generate_template(&name, &description).await?;
                sp.finish_and_clear();

                let path = config.templates_dir().join(format!("{}.toml", name));
                std::fs::create_dir_all(config.templates_dir())?;
                std::fs::write(&path, &content)?;

                TUI::success(&format!("Generated and saved template '{}'", name), Some(&path.to_string_lossy()));
                TUI::info("You can now run: onpkg stack add <name> to use it!");
            }
        }

        Command::Update => {
            TUI::info("Checking for updates...");
            TUI::warn("Self-update not yet implemented. Use 'cargo install --path .' to rebuild.");
        }
    }

    Ok(())
}

fn find_template_fuzzy(template_engine: &TemplateEngine, name: &str) -> Option<crate::templates::TemplateDefinition> {
    if let Some(t) = template_engine.find(name) {
        return Some(t);
    }
    // Try case-insensitive substring match
    let lower_name = name.to_lowercase();
    let all = template_engine.all_templates();
    
    // First try exact name match ignoring case
    if let Some(t) = all.iter().find(|t| t.name.to_lowercase() == lower_name) {
        return Some(t.clone());
    }
    
    // Then try prefix match
    if let Some(t) = all.iter().find(|t| t.name.to_lowercase().starts_with(&lower_name)) {
        return Some(t.clone());
    }
    
    // Then try substring match
    if let Some(t) = all.iter().find(|t| t.name.to_lowercase().contains(&lower_name)) {
        return Some(t.clone());
    }
    
    None
}

/// Detect the package runtime based on the project's manifest files
fn detect_runtime(_name: &str) -> String {
    let cwd = std::env::current_dir().ok();
    if let Some(dir) = cwd {
        if dir.join("package.json").exists() {
            return "npm".to_string();
        }
        if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
            return "pypi".to_string();
        }
        if dir.join("pubspec.yaml").exists() {
            return "pub".to_string();
        }
        if dir.join("Cargo.toml").exists() {
            return "cargo".to_string();
        }
    }
    "npm".to_string()
}
