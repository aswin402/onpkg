use anyhow::{anyhow, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;
use serde_json::Value;

pub fn diff_template(dir: &Path, stack_name: Option<&str>, apply: bool) -> Result<()> {
    let resolved_stack_name = match stack_name {
        Some(name) => name.to_string(),
        None => {
            let onpkg_path = dir.join("onpkg.json");
            if !onpkg_path.exists() {
                return Err(anyhow!(
                    "No stack template name provided and no onpkg.json found in target directory"
                ));
            }
            let manifest_content = fs::read_to_string(&onpkg_path)?;
            let manifest: Value = serde_json::from_str(&manifest_content)?;
            manifest
                .get("project")
                .and_then(|p| p.get("stack"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("Could not resolve project.stack from onpkg.json"))?
        }
    };

    println!("Comparing workspace against stack template: {}", resolved_stack_name);

    let config = crate::config::Config::load()?;
    let template_engine = crate::templates::TemplateEngine::new(config);

    let tmpl = template_engine
        .find(&resolved_stack_name)
        .ok_or_else(|| anyhow!("Stack template '{}' not found", resolved_stack_name))?;

    let env = minijinja::Environment::new();
    let mut vars = std::collections::HashMap::new();
    for v in &tmpl.variables {
        vars.insert(v.name.clone(), v.default.clone());
    }
    // Read project details and custom variables from onpkg.json
    if let Ok(onpkg_content) = fs::read_to_string(dir.join("onpkg.json")) {
        if let Ok(manifest) = serde_json::from_str::<Value>(&onpkg_content) {
            if let Some(proj) = manifest.get("project") {
                if let Some(name) = proj.get("name").and_then(|n| n.as_str()) {
                    vars.insert("project_name".to_string(), name.to_string());
                }
                if let Some(saved_vars) = proj.get("variables").and_then(|v| v.as_object()) {
                    for (k, val) in saved_vars {
                        if let Some(s) = val.as_str() {
                            vars.insert(k.clone(), s.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut has_any_changes = false;

    for file in &tmpl.files {
        let target_file_path = dir.join(&file.path);
        
        // 1. Handle binary files first
        if let Some(expected_bytes) = &file.binary_content {
            if !target_file_path.exists() {
                println!("\nFile missing: {}", file.path);
                has_any_changes = true;
                if apply {
                    if let Some(parent) = target_file_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&target_file_path, expected_bytes)?;
                    println!("  -> Re-scaffolded: {}", file.path);
                }
                continue;
            }
            let current_bytes = fs::read(&target_file_path)?;
            if current_bytes != *expected_bytes {
                has_any_changes = true;
                println!("\nBinary file differs: {}", file.path);
                if apply {
                    fs::write(&target_file_path, expected_bytes)?;
                    println!("  -> Applied template changes to: {}", file.path);
                }
            }
            continue;
        }
        
        // 2. Text files
        let expected_content = if file.skip_template {
            file.content.clone()
        } else {
            env.render_str(&file.content, &vars).unwrap_or_else(|_| file.content.clone())
        };

        if !target_file_path.exists() {
            println!("\nFile missing: {}", file.path);
            has_any_changes = true;
            if apply {
                if let Some(parent) = target_file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target_file_path, &expected_content)?;
                println!("  -> Re-scaffolded: {}", file.path);
            }
            continue;
        }

        let current_content = match fs::read_to_string(&target_file_path) {
            Ok(c) => c,
            Err(_) => {
                // Fallback to byte comparison if it's not valid UTF-8
                let current_bytes = fs::read(&target_file_path)?;
                if current_bytes != expected_content.as_bytes() {
                    has_any_changes = true;
                    println!("\nBinary/non-UTF8 file differs: {}", file.path);
                    if apply {
                        fs::write(&target_file_path, expected_content.as_bytes())?;
                        println!("  -> Applied template changes to: {}", file.path);
                    }
                }
                continue;
            }
        };
        
        // Let's perform line-by-line diff using context grouped ops
        let diff = TextDiff::from_lines(&current_content, &expected_content);
        let has_changes = diff.iter_all_changes().any(|c| c.tag() != ChangeTag::Equal);

        if has_changes {
            has_any_changes = true;
            println!("\nDiff for file: {}", file.path);
            for group in diff.grouped_ops(3) {
                println!("@@ ... @@");
                for op in group {
                    for change in diff.iter_changes(&op) {
                        match change.tag() {
                            ChangeTag::Delete => {
                                // Deletions are shown in red
                                print!("\x1b[31m-{}\x1b[0m", change);
                            }
                            ChangeTag::Insert => {
                                // Insertions are shown in green
                                print!("\x1b[32m+{}\x1b[0m", change);
                            }
                            ChangeTag::Equal => {
                                print!(" {}", change);
                            }
                        }
                    }
                }
            }

            if apply {
                fs::write(&target_file_path, &expected_content)?;
                println!("  -> Applied template changes to: {}", file.path);
            }
        }
    }

    if !has_any_changes {
        println!("Workspace is up-to-date with stack template '{}'. No differences found.", resolved_stack_name);
    }

    Ok(())
}
