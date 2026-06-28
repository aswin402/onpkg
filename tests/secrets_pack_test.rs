use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_pack_secrets_redaction() {
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
    let path = dir.path();

    // Create a file with secrets
    let secret_file = path.join("config.env");
    let secret_content = "OPENAI_API_KEY=\"sk-proj-abcdefghijklmnopqrstuvwxyz01234567890abcde\";\nGITHUB_TOKEN=\"ghp_1234567890abcdef1234567890abcdef1234\";";
    fs::write(&secret_file, secret_content).unwrap();

    // 1. Pack with default options (should redact secrets and warn on stderr)
    let output_file = path.join("packed-context.md");
    let pack_output = Command::new(&bin_path)
        .arg("pack")
        .arg(&path)
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to run pack");

    if !pack_output.status.success() {
        panic!("pack failed: {}", String::from_utf8_lossy(&pack_output.stderr));
    }
    let stderr_str = String::from_utf8_lossy(&pack_output.stderr);
    assert!(stderr_str.contains("OpenAI API Key detected") || stderr_str.contains("WARNING:"), "Stderr should warn about secrets: {}", stderr_str);

    let packed_content = fs::read_to_string(&output_file).unwrap();
    assert!(packed_content.contains("[REDACTED-OPENAI-KEY]"), "OpenAI key should be redacted");
    assert!(packed_content.contains("[REDACTED-GITHUB-PAT]"), "GitHub PAT should be redacted");

    // 2. Pack with --no-redact (should NOT redact secrets)
    let output_file_no_redact = path.join("packed-context-no-redact.md");
    let pack_output_no_redact = Command::new(&bin_path)
        .arg("pack")
        .arg(&path)
        .arg("--output")
        .arg(&output_file_no_redact)
        .arg("--no-redact")
        .output()
        .expect("Failed to run pack with --no-redact");

    assert!(pack_output_no_redact.status.success());
    let packed_content_no_redact = fs::read_to_string(&output_file_no_redact).unwrap();
    assert!(packed_content_no_redact.contains("sk-proj-abcdefghijklmnopqrstuvwxyz01234567890abcde"), "OpenAI key should not be redacted");
    assert!(packed_content_no_redact.contains("ghp_1234567890abcdef1234567890abcdef1234"), "GitHub PAT should not be redacted");
}
