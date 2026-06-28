use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
}

#[derive(Serialize, Debug)]
struct JsonRpcError {
    code: i64,
    message: String,
}

pub fn run_mcp_server() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    
    for line_result in stdin.lock().lines() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }
        
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_res = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError { code: -32700, message: format!("Parse error: {}", e) }),
                    id: None,
                };
                serde_json::to_writer(&mut stdout_lock, &err_res)?;
                stdout_lock.write_all(b"\n")?;
                stdout_lock.flush()?;
                continue;
            }
        };
        
        let response = handle_request(&req);
        serde_json::to_writer(&mut stdout_lock, &response)?;
        stdout_lock.write_all(b"\n")?;
        stdout_lock.flush()?;
    }
    
    Ok(())
}

fn handle_request(req: &JsonRpcRequest) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "initialize" => Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": { "name": "onpkg-mcp", "version": "0.1.2" }
        })),
        "tools/list" => Some(serde_json::json!({
            "tools": [
                {
                    "name": "stack_list",
                    "description": "List all available project stack templates",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "category": { "type": "string", "description": "Filter by category: website, app, frontend, backend, fullstack" }
                        }
                    }
                },
                {
                    "name": "stack_add",
                    "description": "Scaffold a project stack template into the target directory and install dependencies",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Name of the stack template to scaffold" },
                            "dir": { "type": "string", "description": "Target directory path" },
                            "manager": { "type": "string", "description": "Custom package manager (e.g. bun, npm, pnpm, yarn, uv, pip)" },
                            "no_hooks": { "type": "boolean", "description": "Disable running post-scaffold hooks" }
                        },
                        "required": ["name"]
                    }
                },
                {
                    "name": "skill_list",
                    "description": "List all installed AI agent skills",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "skill_install",
                    "description": "Install an AI agent skill from registry or local file path",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Skill name or path to SKILL.md file" }
                        },
                        "required": ["name"]
                    }
                },
                {
                    "name": "sync",
                    "description": "Sync project files/dependencies to onpkg.json and update agent docs",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "dir": { "type": "string", "description": "Target project directory" }
                        }
                    }
                },
                {
                    "name": "map",
                    "description": "Map project directory structure and extract code symbols using tree-sitter",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "dir": { "type": "string", "description": "Target directory to map" },
                            "format": { "type": "string", "description": "Output format: markdown, json", "enum": ["markdown", "json"] }
                        }
                    }
                },
                {
                    "name": "pack",
                    "description": "Pack codebase structure and file contents into a token-budgeted prompt context",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "dir": { "type": "string", "description": "Target directory to pack" },
                            "max_tokens": { "type": "integer", "description": "Max token budget for the packed prompt context" }
                        }
                    }
                },
                {
                    "name": "doctor",
                    "description": "Perform diagnostic health check on onpkg configuration, database, templates, skills and tools",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                }
            ]
        })),
        "tools/call" => {
            if let Some(params) = &req.params {
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or_default();
                let tool_args = params.get("arguments");
                match handle_tool_call(tool_name, tool_args) {
                    Ok(val) => Some(val),
                    Err(e) => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError { code: -32603, message: format!("Tool execution error: {}", e) }),
                            id: req.id.clone(),
                        };
                    }
                }
            } else {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError { code: -32602, message: "Invalid params".to_string() }),
                    id: req.id.clone(),
                };
            }
        }
        _ => None,
    };
    
    if let Some(res) = result {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(res),
            error: None,
            id: req.id.clone(),
        }
    } else {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError { code: -32601, message: format!("Method not found: {}", req.method) }),
            id: req.id.clone(),
        }
    }
}

fn handle_tool_call(name: &str, arguments: Option<&Value>) -> Result<Value> {
    let mut args = Vec::new();
    let current_exe = std::env::current_exe()?;

    match name {
        "stack_list" => {
            args.push("--json".to_string());
            args.push("stack".to_string());
            args.push("list".to_string());
            if let Some(args_map) = arguments.and_then(|a| a.as_object()) {
                if let Some(cat) = args_map.get("category").and_then(|c| c.as_str()) {
                    args.push("--category".to_string());
                    args.push(cat.to_string());
                }
            }
        }
        "stack_add" => {
            args.push("stack".to_string());
            args.push("add".to_string());
            let mut name_val = None;
            if let Some(args_map) = arguments.and_then(|a| a.as_object()) {
                if let Some(n) = args_map.get("name").and_then(|n| n.as_str()) {
                    name_val = Some(n.to_string());
                }
                if let Some(dir) = args_map.get("dir").and_then(|d| d.as_str()) {
                    args.push("--dir".to_string());
                    args.push(dir.to_string());
                }
                if let Some(mgr) = args_map.get("manager").and_then(|m| m.as_str()) {
                    args.push("--manager".to_string());
                    args.push(mgr.to_string());
                }
                if let Some(no_hooks) = args_map.get("no_hooks").and_then(|nh| nh.as_bool()) {
                    if no_hooks {
                        args.push("--no-hooks".to_string());
                    }
                }
            }
            if let Some(nv) = name_val {
                args.push(nv);
            } else {
                return Err(anyhow!("Missing required parameter: name"));
            }
        }
        "skill_list" => {
            args.push("--json".to_string());
            args.push("skill".to_string());
            args.push("list".to_string());
        }
        "skill_install" => {
            args.push("skill".to_string());
            args.push("install".to_string());
            let mut name_val = None;
            if let Some(args_map) = arguments.and_then(|a| a.as_object()) {
                if let Some(n) = args_map.get("name").and_then(|n| n.as_str()) {
                    name_val = Some(n.to_string());
                }
            }
            if let Some(nv) = name_val {
                args.push(nv);
            } else {
                return Err(anyhow!("Missing required parameter: name"));
            }
        }
        "sync" => {
            args.push("sync".to_string());
            if let Some(args_map) = arguments.and_then(|a| a.as_object()) {
                if let Some(dir) = args_map.get("dir").and_then(|d| d.as_str()) {
                    args.push("--dir".to_string());
                    args.push(dir.to_string());
                }
            }
        }
        "map" => {
            args.push("map".to_string());
            if let Some(args_map) = arguments.and_then(|a| a.as_object()) {
                if let Some(dir) = args_map.get("dir").and_then(|d| d.as_str()) {
                    args.push("--dir".to_string());
                    args.push(dir.to_string());
                }
                if let Some(fmt) = args_map.get("format").and_then(|f| f.as_str()) {
                    args.push("--format".to_string());
                    args.push(fmt.to_string());
                }
            }
        }
        "pack" => {
            args.push("pack".to_string());
            if let Some(args_map) = arguments.and_then(|a| a.as_object()) {
                if let Some(dir) = args_map.get("dir").and_then(|d| d.as_str()) {
                    args.push("--dir".to_string());
                    args.push(dir.to_string());
                }
                if let Some(mt) = args_map.get("max_tokens").and_then(|t| t.as_u64()) {
                    args.push("--max-tokens".to_string());
                    args.push(mt.to_string());
                }
            }
        }
        "doctor" => {
            args.push("--json".to_string());
            args.push("doctor".to_string());
        }
        _ => return Err(anyhow!("Unsupported tool: {}", name)),
    }

    let output = std::process::Command::new(current_exe)
        .args(&args)
        .output()?;

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
    
    let full_output = if output.status.success() {
        stdout_str
    } else {
        format!("Error (exit code: {:?}):\n{}\n{}", output.status.code(), stderr_str, stdout_str)
    };

    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": full_output
            }
        ],
        "isError": !output.status.success()
    }))
}
