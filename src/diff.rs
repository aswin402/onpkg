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
    // Attempt to read the actual project name from onpkg.json to substitute project_name
    if let Ok(onpkg_content) = fs::read_to_string(dir.join("onpkg.json")) {
        if let Ok(manifest) = serde_json::from_str::<Value>(&onpkg_content) {
            if let Some(proj) = manifest.get("project") {
                if let Some(name) = proj.get("name").and_then(|n| n.as_str()) {
                    vars.insert("project_name".to_string(), name.to_string());
                }
            }
        }
    }

    let mut has_any_changes = false;

    for file in &tmpl.files {
        let target_file_path = dir.join(&file.path);
        
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

        let current_content = fs::read_to_string(&target_file_path)?;
        
        // Let's perform line-by-line diff
        let diff = TextDiff::from_lines(&current_content, &expected_content);
        let has_changes = diff.iter_all_changes().any(|c| c.tag() != ChangeTag::Equal);

        if has_changes {
            has_any_changes = true;
            println!("\nDiff for file: {}", file.path);
            for change in diff.iter_all_changes() {
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
