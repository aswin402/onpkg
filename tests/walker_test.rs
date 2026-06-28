use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[path = "../src/walker.rs"]
mod walker;

#[test]
fn test_walker_ignores() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    
    // Create test files
    fs::write(path.join("keep.rs"), "").unwrap();
    fs::write(path.join("ignore_me.rs"), "").unwrap();
    
    // Create a .onpkgignore
    let mut ignore_file = File::create(path.join(".onpkgignore")).unwrap();
    writeln!(ignore_file, "ignore_me.rs").unwrap();
    
    let walker_res = walker::get_project_walker(path).unwrap();
    let file_names: Vec<String> = walker_res
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
        
    assert!(file_names.contains(&"keep.rs".to_string()));
    assert!(!file_names.contains(&"ignore_me.rs".to_string()));
}
