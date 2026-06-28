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

    let mut has_any_changes = false;

    for file in &tmpl.files {
        let target_file_path = dir.join(&file.path);
        if !target_file_path.exists() {
            println!("\nFile missing: {}", file.path);
            has_any_changes = true;
            if apply {
                if let Some(parent) = target_file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&target_file_path, &file.content)?;
                println!("  -> Re-scaffolded: {}", file.path);
            }
            continue;
        }

        let current_content = fs::read_to_string(&target_file_path)?;
        
        // Let's perform line-by-line diff
        let diff = TextDiff::from_lines(&current_content, &file.content);
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
                fs::write(&target_file_path, &file.content)?;
                println!("  -> Applied template changes to: {}", file.path);
            }
        }
    }

    if !has_any_changes {
        println!("Workspace is up-to-date with stack template '{}'. No differences found.", resolved_stack_name);
    }

    Ok(())
}
