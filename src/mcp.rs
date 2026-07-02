use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::collections::HashMap;

use crate::config::Config;
use crate::db::Database;
use crate::registry::Registry;
use crate::templates::TemplateEngine;
use crate::skill::SkillManager;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub(crate) struct JsonRpcRequest {
    pub(crate) jsonrpc: String,
    pub(crate) method: String,
    pub(crate) params: Option<Value>,
    pub(crate) id: Option<Value>,
}

#[derive(Serialize, Debug)]
pub(crate) struct JsonRpcResponse {
    pub(crate) jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<JsonRpcError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<Value>,
}

#[derive(Serialize, Debug)]
pub(crate) struct JsonRpcError {
    pub(crate) code: i64,
    pub(crate) message: String,
}

pub async fn run_mcp_server(
    config: Config,
    db: Database,
    registry: Registry,
    template_engine: TemplateEngine,
    skill_manager: SkillManager,
) -> Result<()> {
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
        
        let response = handle_request(&req, &config, &db, &registry, &template_engine, &skill_manager).await;
        serde_json::to_writer(&mut stdout_lock, &response)?;
        stdout_lock.write_all(b"\n")?;
        stdout_lock.flush()?;
    }
    
    Ok(())
}

pub(crate) async fn handle_request(
    req: &JsonRpcRequest,
    config: &Config,
    db: &Database,
    registry: &Registry,
    template_engine: &TemplateEngine,
    skill_manager: &SkillManager,
) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "initialize" => Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": { "name": "onpkg-mcp", "version": env!("CARGO_PKG_VERSION") }
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
                            "dir": { "type": "string", "description": "Target project directory" },
                            "no_agents_md": { "type": "boolean", "description": "Skip generating AGENTS.md workflow file" },
                            "symlink_claude": { "type": "boolean", "description": "Create a symlink CLAUDE.md pointing to AGENTS.md if CLAUDE.md doesn't exist" }
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
                },
                {
                    "name": "stack_show",
                    "description": "Show details of a specific project stack template",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Name of the stack template" }
                        },
                        "required": ["name"]
                    }
                },
                {
                    "name": "stack_new",
                    "description": "Create a new custom stack template definition",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Name of the stack template" },
                            "category": { "type": "string", "description": "Category for the stack template (default: custom)" }
                        },
                        "required": ["name"]
                    }
                },
                {
                    "name": "stack_diff",
                    "description": "Compare workspace files against a stack template and optionally apply changes",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Name of the stack template (defaults to current stack)" },
                            "apply": { "type": "boolean", "description": "Automatically apply template changes to the workspace" }
                        }
                    }
                },
                {
                    "name": "ai_template",
                    "description": "Generate a new template definition TOML using AI",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Template name (e.g. rust-wasm, nextjs-sqlite)" },
                            "description": { "type": "string", "description": "Description of the stack, architecture, and files needed" }
                        },
                        "required": ["name", "description"]
                    }
                },
                {
                    "name": "ai_skill",
                    "description": "Generate a new AI agent skill markdown file using AI",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Technology name (e.g. react, tailwind, docker)" },
                            "prompt": { "type": "string", "description": "Optional prompt or guidance for the skill content" }
                        },
                        "required": ["name"]
                    }
                }
            ]
        })),
        "tools/call" => {
            if let Some(params) = &req.params {
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or_default();
                let tool_args = params.get("arguments");
                match handle_tool_call(tool_name, tool_args, config, db, registry, template_engine, skill_manager).await {
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

async fn handle_tool_call(
    name: &str,
    arguments: Option<&Value>,
    config: &Config,
    db: &Database,
    registry: &Registry,
    template_engine: &TemplateEngine,
    skill_manager: &SkillManager,
) -> Result<Value> {
    match name {
        "stack_list" => {
            let category = arguments.and_then(|a| a.get("category")).and_then(|c| c.as_str());
            let templates = template_engine.all_templates();
            let mut list = Vec::new();
            for t in templates {
                if let Some(cat) = category {
                    if t.category != cat {
                        continue;
                    }
                }
                list.push(serde_json::json!({
                    "name": t.name,
                    "category": t.category,
                    "description": t.description,
                    "files_count": t.files.len(),
                }));
            }
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&list)? }],
                "isError": false
            }))
        }
        "stack_add" => {
            let stack_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str()).ok_or_else(|| anyhow!("Missing parameter: name"))?;
            let dir_str = arguments.and_then(|a| a.get("dir")).and_then(|d| d.as_str());
            let manager = arguments.and_then(|a| a.get("manager")).and_then(|m| m.as_str());
            let no_hooks = arguments.and_then(|a| a.get("no_hooks")).and_then(|nh| nh.as_bool()).unwrap_or(false);
            
            let target = dir_str.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            let tmpl = template_engine.find(stack_name).ok_or_else(|| anyhow!("Stack '{}' not found", stack_name))?;
            
            let created = crate::templates::scaffold_and_setup_stack(template_engine, &tmpl, &target, &HashMap::new(), manager, no_hooks, config)?;
            
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("Stack '{}' scaffolded successfully. {} files created.", tmpl.name, created.len()) }],
                "isError": false
            }))
        }
        "skill_list" => {
            let skills = skill_manager.list()?;
            let list: Vec<serde_json::Value> = skills
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "name": s.name,
                        "version": s.version,
                        "description": s.description,
                    })
                })
                .collect();
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&list)? }],
                "isError": false
            }))
        }
        "skill_install" => {
            let skill_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str()).ok_or_else(|| anyhow!("Missing parameter: name"))?;
            skill_manager.install(skill_name)?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("Skill '{}' installed successfully.", skill_name) }],
                "isError": false
            }))
        }
        "sync" => {
            let dir_str = arguments.and_then(|a| a.get("dir")).and_then(|d| d.as_str());
            let target = dir_str.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            let no_agents_md = arguments.and_then(|a| a.get("no_agents_md")).and_then(|n| n.as_bool()).unwrap_or(false);
            let symlink_claude = arguments.and_then(|a| a.get("symlink_claude")).and_then(|s| s.as_bool()).unwrap_or(false);
            crate::templates::sync_onpkg_project(&target, None, None, None, no_agents_md, symlink_claude)?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": "Sync completed successfully." }],
                "isError": false
            }))
        }
        "map" => {
            let dir_str = arguments.and_then(|a| a.get("dir")).and_then(|d| d.as_str());
            let target = dir_str.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            let format = arguments.and_then(|a| a.get("format")).and_then(|f| f.as_str()).unwrap_or("markdown");
            
            let map = crate::mapper::map_project(&target)?;
            let text = if format == "json" {
                serde_json::to_string_pretty(&map)?
            } else {
                crate::mapper::format_markdown(&map)
            };
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": text }],
                "isError": false
            }))
        }
        "pack" => {
            let dir_str = arguments.and_then(|a| a.get("dir")).and_then(|d| d.as_str());
            let target = dir_str.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            let max_tokens = arguments.and_then(|a| a.get("max_tokens")).and_then(|t| t.as_u64()).unwrap_or(100000) as usize;
            
            let result = crate::packer::pack_project(&target, max_tokens, false)?;
            Ok(serde_json::json!({
                "content": [
                    { "type": "text", "text": result.content },
                    { "type": "text", "text": format!("Token count: {}\nFiles embedded: {}\nSkipped files: {:?}", result.token_count, result.file_count, result.skipped_files) }
                ],
                "isError": false
            }))
        }
        "doctor" => {
            let mut diagnostics = Vec::new();
            diagnostics.push(serde_json::json!({"check": "config", "status": "ok", "detail": "~/.onpkg/config.toml"}));
            
            if let Ok(count) = db.count_packages() {
                let mut is_healthy = true;
                if let Ok(conn) = rusqlite::Connection::open(config.db_path()) {
                    if let Ok(mut stmt) = conn.prepare("PRAGMA integrity_check") {
                        if let Ok(mut rows) = stmt.query([]) {
                            if let Ok(Some(row)) = rows.next() {
                                let status: String = row.get(0).unwrap_or_default();
                                if status != "ok" {
                                    is_healthy = false;
                                }
                            }
                        }
                    }
                }
                if is_healthy {
                    diagnostics.push(serde_json::json!({"check": "database", "status": "ok", "detail": format!("{} packages cached (integrity check passed)", count)}));
                } else {
                    diagnostics.push(serde_json::json!({"check": "database", "status": "error", "detail": "database integrity check failed!"}));
                }
            }
            
            let tmpl_count = template_engine.all_templates().len();
            diagnostics.push(serde_json::json!({"check": "templates", "status": "ok", "detail": format!("{} available", tmpl_count)}));
            
            if let Ok(skills) = skill_manager.list() {
                diagnostics.push(serde_json::json!({"check": "skills", "status": "ok", "detail": format!("{} installed", skills.len())}));
            }
            
            if let Ok(status) = registry.check_health().await {
                let s = status.get("status").map(|s| s.as_str()).unwrap_or("unknown");
                diagnostics.push(serde_json::json!({"check": "registry", "status": "ok", "detail": s}));
            }
            
            for (cmd, name, min_req) in &[
                ("node", "Node.js", Some(semver::VersionReq::parse(">=18.0.0").unwrap())),
                ("bun", "Bun", Some(semver::VersionReq::parse(">=1.0.0").unwrap())),
                ("python3", "Python 3", None),
                ("cargo", "Cargo", None),
            ] {
                match std::process::Command::new(cmd).arg("--version").output() {
                    Ok(out) => {
                        let raw_ver = String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .next()
                            .unwrap_or("0.0.0")
                            .trim_start_matches('v')
                            .to_string();
                        let status = if let Some(req) = min_req {
                            if let Ok(parsed_ver) = semver::Version::parse(&raw_ver.split('-').next().unwrap_or("0.0.0")) {
                                if req.matches(&parsed_ver) { "ok" } else { "warn" }
                            } else {
                                "ok"
                            }
                        } else {
                            "ok"
                        };
                        diagnostics.push(serde_json::json!({"check": name, "status": status, "detail": raw_ver}));
                    }
                    Err(_) => diagnostics.push(serde_json::json!({"check": name, "status": "missing", "detail": "not found on PATH"})),
                }
            }
            
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&diagnostics)? }],
                "isError": false
            }))
        }
        "stack_show" => {
            let stack_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str()).ok_or_else(|| anyhow!("Missing parameter: name"))?;
            let tmpl = template_engine.find(stack_name).ok_or_else(|| anyhow!("Stack '{}' not found", stack_name))?;
            let info = serde_json::json!({
                "name": tmpl.name,
                "category": tmpl.category,
                "description": tmpl.description,
                "files": tmpl.files.iter().map(|f| &f.path).collect::<Vec<_>>(),
                "variables": tmpl.variables.iter().map(|v| serde_json::json!({
                    "name": v.name,
                    "default": v.default,
                    "description": v.description,
                })).collect::<Vec<_>>(),
            });
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&info)? }],
                "isError": false
            }))
        }
        "stack_new" => {
            let stack_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str()).ok_or_else(|| anyhow!("Missing parameter: name"))?;
            let category = arguments.and_then(|a| a.get("category")).and_then(|c| c.as_str()).unwrap_or("custom");
            let custom_dir = config.templates_dir();
            std::fs::create_dir_all(&custom_dir)?;
            let path = custom_dir.join(format!("{}.toml", stack_name));
            if path.exists() {
                return Err(anyhow!("Stack definition already exists at {:?}", path));
            }
            let template_toml = format!(
                "name = \"{}\"\ncategory = \"{}\"\ndescription = \"Custom template created with onpkg\"\n\n[[files]]\npath = \"example.txt\"\ncontent = \"Hello from custom stack!\"\n\n[[variables]]\nname = \"project_name\"\ndefault = \"my-project\"\ndescription = \"Name of the project\"\n",
                stack_name, category
            );
            std::fs::write(&path, template_toml)?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("Stack definition created at {:?}", path) }],
                "isError": false
            }))
        }
        "stack_diff" => {
            let stack_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str());
            let apply = arguments.and_then(|a| a.get("apply")).and_then(|ap| ap.as_bool()).unwrap_or(false);
            let target = std::env::current_dir()?;
            crate::diff::diff_template(&target, stack_name, apply)?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": "Template diff comparison complete." }],
                "isError": false
            }))
        }
        "ai_template" => {
            let stack_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str()).ok_or_else(|| anyhow!("Missing parameter: name"))?;
            let description = arguments.and_then(|a| a.get("description")).and_then(|d| d.as_str()).ok_or_else(|| anyhow!("Missing parameter: description"))?;
            let ai = crate::ai::AiGenerator::new()?;
            let content = ai.generate_template(stack_name, description).await?;
            let path = config.templates_dir().join(format!("{}.toml", stack_name));
            std::fs::create_dir_all(config.templates_dir())?;
            std::fs::write(&path, &content)?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("Custom template '{}' generated and saved to {:?}", stack_name, path) }],
                "isError": false
            }))
        }
        "ai_skill" => {
            let skill_name = arguments.and_then(|a| a.get("name")).and_then(|n| n.as_str()).ok_or_else(|| anyhow!("Missing parameter: name"))?;
            let prompt = arguments.and_then(|a| a.get("prompt")).and_then(|p| p.as_str());
            let ai = crate::ai::AiGenerator::new()?;
            let content = ai.generate_skill(skill_name, prompt).await?;
            let path = config.skills_dir().join(format!("{}.md", skill_name));
            std::fs::create_dir_all(config.skills_dir())?;
            std::fs::write(&path, &content)?;
            skill_manager.install_from_path(skill_name, &path)?;
            Ok(serde_json::json!({
                "content": [{ "type": "text", "text": format!("Skill '{}' generated and installed to {:?}", skill_name, path) }],
                "isError": false
            }))
        }
        _ => Err(anyhow!("Unsupported tool: {}", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_initialize() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut config = crate::config::Config::default();
        config.home_override = Some(temp_dir.path().to_path_buf());
        
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {},
            "id": 1
        });
        
        let req_struct: JsonRpcRequest = serde_json::from_value(req).unwrap();
        let db = crate::db::Database::open(&config).unwrap();
        let registry = crate::registry::Registry::new(config.clone());
        let template_engine = crate::templates::TemplateEngine::new(config.clone());
        let skill_manager = crate::skill::SkillManager::new(config.clone(), db.clone());

        let response = handle_request(&req_struct, &config, &db, &registry, &template_engine, &skill_manager).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(serde_json::json!(1)));
        
        let result_val = response.result.unwrap();
        assert_eq!(result_val["serverInfo"]["name"], "onpkg-mcp");
    }

    #[tokio::test]
    async fn test_mcp_tools_list() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut config = crate::config::Config::default();
        config.home_override = Some(temp_dir.path().to_path_buf());
        
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": "my-id"
        });
        
        let req_struct: JsonRpcRequest = serde_json::from_value(req).unwrap();
        let db = crate::db::Database::open(&config).unwrap();
        let registry = crate::registry::Registry::new(config.clone());
        let template_engine = crate::templates::TemplateEngine::new(config.clone());
        let skill_manager = crate::skill::SkillManager::new(config.clone(), db.clone());

        let response = handle_request(&req_struct, &config, &db, &registry, &template_engine, &skill_manager).await;
        
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert_eq!(response.id, Some(serde_json::json!("my-id")));
        
        let result_val = response.result.unwrap();
        let tools = result_val["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "sync"));
        assert!(tools.iter().any(|t| t["name"] == "doctor"));
    }
}
