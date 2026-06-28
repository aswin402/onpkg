use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_onpkg_stack_diff_and_apply() {
    let mut bin_path = std::env::current_exe().unwrap();
    bin_path.pop(); // pop test bin
    if bin_path.file_name().and_then(|s| s.to_str()) == Some("deps") {
        bin_path.pop(); // pop deps
    }
    bin_path.push("onpkg");
    if !bin_path.exists() {
        bin_path.set_extension("exe");
    }

    let dir = tempdir().unwrap();
    let path = dir.path().join("my-site");
    fs::create_dir_all(&path).unwrap();

    // 1. Scaffold a simple built-in stack in the temp dir
    let scaffold_output = Command::new(&bin_path)
        .arg("stack")
        .arg("add")
        .arg("static-website")
        .arg("--dir")
        .arg(&path)
        .arg("--no-hooks")
        .output()
        .expect("Failed to scaffold static-website");
    
    assert!(scaffold_output.status.success(), "Scaffolding failed: {}", String::from_utf8_lossy(&scaffold_output.stderr));

    // 2. Run diff. It should show no changes.
    let diff_output = Command::new(&bin_path)
        .arg("stack")
        .arg("diff")
        .arg("static-website")
        .current_dir(&path)
        .output()
        .expect("Failed to run diff");

    let diff_str = String::from_utf8_lossy(&diff_output.stdout);
    assert!(diff_str.contains("Workspace is up-to-date"), "Diff output should state workspace is up-to-date: {}", diff_str);

    // 3. Modify a file in the workspace
    let index_html = path.join("index.html");
    if index_html.exists() {
        fs::write(&index_html, "<!-- modified -->").unwrap();
    }

    // 4. Run diff again. It should show changes.
    let diff_output_2 = Command::new(&bin_path)
        .arg("stack")
        .arg("diff")
        .arg("static-website")
        .current_dir(&path)
        .output()
        .expect("Failed to run diff 2");

    let diff_str_2 = String::from_utf8_lossy(&diff_output_2.stdout);
    assert!(diff_str_2.contains("Diff for file: index.html"), "Diff output should show changes for index.html: {}", diff_str_2);

    // 5. Run diff with --apply. It should overwrite the modified file back to original.
    let apply_output = Command::new(&bin_path)
        .arg("stack")
        .arg("diff")
        .arg("static-website")
        .arg("--apply")
        .current_dir(&path)
        .output()
        .expect("Failed to apply diff");

    let apply_str = String::from_utf8_lossy(&apply_output.stdout);
    assert!(apply_str.contains("Applied template changes to: index.html") || apply_str.contains("Applied template changes"), "Apply output should confirm file overwrite: {}", apply_str);

    // 6. Verify file is reverted
    let reverted_content = fs::read_to_string(&index_html).unwrap();
    assert!(!reverted_content.contains("<!-- modified -->"), "File should be reverted");
}
