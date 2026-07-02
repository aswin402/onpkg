#[path = "../src/mcp.rs"]
pub mod mcp;

#[test]
fn test_mcp_initialize() {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {},
        "id": 1
    });
    
    let req_struct: mcp::JsonRpcRequest = serde_json::from_value(req).unwrap();
    let response = mcp::handle_request(&req_struct);
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    assert_eq!(response.id, Some(serde_json::json!(1)));
    
    let result_val = response.result.unwrap();
    assert_eq!(result_val["serverInfo"]["name"], "onpkg-mcp");
}

#[test]
fn test_mcp_tools_list() {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": "my-id"
    });
    
    let req_struct: mcp::JsonRpcRequest = serde_json::from_value(req).unwrap();
    let response = mcp::handle_request(&req_struct);
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert_eq!(response.id, Some(serde_json::json!("my-id")));
    
    let result_val = response.result.unwrap();
    let tools = result_val["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "sync"));
    assert!(tools.iter().any(|t| t["name"] == "doctor"));
}
