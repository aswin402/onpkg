use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Get a customized file walker that respects .gitignore, global gitignores,
/// and local .onpkgignore files, falling back to standard excludes.
pub fn get_project_walker(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(dir);
    
    // Add local .onpkgignore custom override
    let onpkgignore = dir.join(".onpkgignore");
    if onpkgignore.exists() {
        if let Some(err) = builder.add_ignore(onpkgignore) {
            tracing::warn!("Failed to load .onpkgignore: {}", err);
        }
    }
    
    // Configure default builder rules
    builder
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .hidden(true); // Ignore hidden files like .git by default
        
    let mut paths = Vec::new();
    for result in builder.build() {
        match result {
            Ok(entry) => {
                if entry.file_type().is_some_and(|ft| ft.is_file()) {
                    paths.push(entry.path().to_path_buf());
                }
            }
            Err(e) => tracing::warn!("Error walking directory: {}", e),
        }
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_walker_basic_ignores() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        
        fs::write(path.join("keep.rs"), "").unwrap();
        fs::write(path.join("ignore_me.rs"), "").unwrap();
        
        let mut ignore_file = File::create(path.join(".onpkgignore")).unwrap();
        writeln!(ignore_file, "ignore_me.rs").unwrap();
        
        let walker_res = get_project_walker(path).unwrap();
        let file_names: Vec<String> = walker_res
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
            
        assert!(file_names.contains(&"keep.rs".to_string()));
        assert!(!file_names.contains(&"ignore_me.rs".to_string()));
    }
}

