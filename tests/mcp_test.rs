use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn test_mcp_initialize_and_list_tools() {
    let mut bin_path = std::env::current_exe().unwrap();
    // current_exe is typically target/debug/deps/mapper_test-xxxx
    // We pop deps/ and the test executable name to get to target/debug/
    bin_path.pop(); // pop test bin name
    if bin_path.file_name().and_then(|s| s.to_str()) == Some("deps") {
        bin_path.pop(); // pop deps
    }
    bin_path.push("onpkg");
    if !bin_path.exists() {
        bin_path.set_extension("exe");
    }

    let mut child = Command::new(bin_path)
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn onpkg serve");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Send initialize request
    let init_req = r#"{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0"}},"id":1}"#;
    writeln!(stdin, "{}", init_req).unwrap();
    stdin.flush().unwrap();

    let mut init_res = String::new();
    reader.read_line(&mut init_res).unwrap();
    assert!(init_res.contains("onpkg-mcp"), "Response should contain onpkg-mcp: {}", init_res);
    assert!(init_res.contains(r#""id":1"#), "Response should match id 1: {}", init_res);

    // 2. Send tools/list request
    let list_req = r#"{"jsonrpc":"2.0","method":"tools/list","id":2}"#;
    writeln!(stdin, "{}", list_req).unwrap();
    stdin.flush().unwrap();

    let mut list_res = String::new();
    reader.read_line(&mut list_res).unwrap();
    assert!(list_res.contains("stack_list"), "Response should contain stack_list: {}", list_res);
    assert!(list_res.contains("doctor"), "Response should contain doctor: {}", list_res);
    assert!(list_res.contains(r#""id":2"#), "Response should match id 2: {}", list_res);

    let _ = child.kill();
}
