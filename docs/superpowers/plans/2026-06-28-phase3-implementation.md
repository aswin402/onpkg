# Phase 3 — "Make It Indispensable" (v0.3.0) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Phase 3 features including stdio JSON-RPC MCP Server mode, file-system watch sync, template diff comparisons, monorepo/workspace package detection, and secret redaction/redirection in prompt packing.

**Architecture:** Create new modular files (`src/mcp.rs`, `src/watch.rs`, `src/diff.rs`, `src/secrets.rs`), wire workspace parsing into `src/templates/mod.rs`, and register new subcommands in `src/cli.rs` and `src/main.rs`.

**Tech Stack:** Rust, notify, similar, regex, serde_json.

## Global Constraints
- Target Rust version: Edition 2021
- Keep stdio parsing lightweight without heavy HTTP servers.
- Use `spawn_blocking` or async process spawns appropriately.
- Ensure all unit/integration tests compile cleanly and pass.

---

### Task 1: Stdio JSON-RPC MCP Server (`onpkg serve`)

**Files:**
- Modify: `Cargo.toml`
- Create: `src/mcp.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Test: `tests/mcp_test.rs`

**Interfaces:**
- Produces: `mcp::run_mcp_server() -> Result<()>`

- [ ] **Step 1: Create src/mcp.rs**

Create the file `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/mcp.rs` implementing standard JSON-RPC 2.0 stdio server:
```rust
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

#[derive(Deserialize, Debug)]
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
            "capabilities": {},
            "serverInfo": { "name": "onpkg-mcp", "version": "0.1.2" }
        })),
        "tools/list" => Some(serde_json::json!({
            "tools": [
                {
                    "name": "sync",
                    "description": "Sync project dependencies and generate AI AGENTS.md config",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "dir": { "type": "string", "description": "Target project directory" }
                        }
                    }
                },
                {
                    "name": "doctor",
                    "description": "Perform diagnostics checks on environment and templates",
                    "inputSchema": { "type": "object", "properties": {} }
                }
            ]
        })),
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
```

- [ ] **Step 2: Add CLI serve command definitions**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs`, add the `Serve` subcommand:
```rust
    /// Start the standard stdio JSON-RPC Model Context Protocol (MCP) server
    Serve,
```

- [ ] **Step 3: Register and wire Serve command in src/main.rs**

1. Register `pub mod mcp;` in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`.
2. Wire `Command::Serve`:
```rust
        Command::Serve => {
            mcp::run_mcp_server()?;
        }
```

- [ ] **Step 4: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement stdio JSON-RPC MCP server subcommand"`

---

### Task 2: Debounced File Watcher (`onpkg sync --watch`)

**Files:**
- Modify: `Cargo.toml`
- Create: `src/watch.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `watch::watch_project(dir: &Path) -> Result<()>`

- [ ] **Step 1: Add notify dependency to Cargo.toml**

Add the `notify` crate under the dependencies section in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/Cargo.toml`:
```toml
notify = "6.1.1"
```

- [ ] **Step 2: Create src/watch.rs**

Create the file `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/watch.rs`:
```rust
use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch_project(dir: &Path) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(dir, RecursiveMode::Recursive)?;
    
    println!("Watching directory {} for changes... (Ctrl+C to stop)", dir.display());
    
    let exclusions = ["node_modules", "target", ".git", ".venv", "onpkg_docs", "onpkg.json"];
    
    loop {
        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(event)) => {
                let should_sync = event.paths.iter().any(|path| {
                    !path.components().any(|c| {
                        if let std::path::Component::Normal(name) = c {
                            exclusions.contains(&name.to_string_lossy().as_ref())
                        } else {
                            false
                        }
                    })
                });
                
                if should_sync {
                    println!("Change detected. Re-syncing project manifest...");
                    let path_to_sync = PathBuf::from(dir);
                    if let Err(e) = crate::templates::sync_onpkg_project(&path_to_sync) {
                        tracing::warn!("Auto-sync failed: {}", e);
                    }
                }
            }
            Ok(Err(e)) => tracing::error!("Watcher error: {}", e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Add --watch flag to CLI sync command**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs`:
```rust
    /// Sync project files/dependencies to onpkg.json and update docs
    Sync {
        /// Target directory
        dir: Option<String>,
        /// Live watch directory and automatically sync on changes
        #[arg(long, short)]
        watch: bool,
    },
```

- [ ] **Step 4: Wire watch loop in src/main.rs**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`, update `Command::Sync` subcommand handling block:
```rust
        Command::Sync { dir, watch } => {
            let target = dir.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            if watch {
                pub mod watch; // temporary module binding if needed or declare watch mod globally
                watch::watch_project(&target)?;
            } else {
                TUI::logo();
                let sp = TUI::spinner("Synchronizing project manifest...");
                let result = tokio::task::spawn_blocking(move || {
                    templates::sync_onpkg_project(&target)
                }).await??;
                sp.finish_and_clear();
                TUI::success("Project synchronized successfully! onpkg.json & onpkg_docs/ updated", None);
            }
        }
```

- [ ] **Step 5: Register watch module**

Add `pub mod watch;` right after `pub mod walker;` in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`.

- [ ] **Step 6: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement debounced sync --watch mode"`

---

### Task 3: Template Diff & Upgrades (`onpkg stack diff`)

**Files:**
- Modify: `Cargo.toml`
- Create: `src/diff.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `diff::diff_template(dir: &Path, stack_name: &str, apply: bool) -> Result<()>`

- [ ] **Step 1: Add similar dependency to Cargo.toml**

Add the `similar` crate under the dependencies section in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/Cargo.toml`:
```toml
similar = { version = "2.4", features = ["inline"] }
```

- [ ] **Step 2: Create src/diff.rs**

Create `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/diff.rs`:
```rust
use anyhow::{anyhow, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

pub fn diff_template(dir: &Path, stack_name: &str, apply: bool) -> Result<()> {
    // Find the stack definition
    let config = crate::config::Config::load()?;
    let db = crate::db::Database::open(&config)?;
    let template_engine = crate::templates::TemplateEngine::new(db)?;
    
    let tmpl = template_engine.find(stack_name)
        .ok_or_else(|| anyhow!("Stack template '{}' not found", stack_name))?;
        
    for file in &tmpl.files {
        let target_file_path = dir.join(&file.path);
        if !target_file_path.exists() {
            println!("File missing: {}", file.path);
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
        let diff = TextDiff::from_lines(&current_content, &file.content);
        
        let has_changes = diff.iter_all_changes().any(|c| c.tag() != ChangeTag::Equal);
        if has_changes {
            println!("\nDiff for file: {}", file.path);
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                print!("{}{}", sign, change);
            }
            
            if apply {
                fs::write(&target_file_path, &file.content)?;
                println!("  -> Applied template changes to: {}", file.path);
            }
        }
    }
    
    Ok(())
}
```

- [ ] **Step 3: Add CLI stack diff subcommand**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs`, add `Diff` command to `StackSubcommand`:
```rust
    /// Compare workspace files against a stack template
    Diff {
        /// Name of the stack template to diff against
        name: String,
        /// Automatically apply template changes to local workspace
        #[arg(long)]
        apply: bool,
    },
```

- [ ] **Step 4: Wire Command::Stack::Diff in src/main.rs**

1. Register `pub mod diff;` in `src/main.rs`.
2. Handle `StackSubcommand::Diff` inside the stack match loop:
```rust
            StackSubcommand::Diff { name, apply } => {
                let target = std::env::current_dir()?;
                diff::diff_template(&target, &name, apply)?;
            }
```

- [ ] **Step 5: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement stack diff command using similar crate"`

---

### Task 4: Monorepo & Workspace Detection

**Files:**
- Modify: `src/templates/mod.rs`

- [ ] **Step 1: Add workspaces parser logic in sync_onpkg_project**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/templates/mod.rs`, update `sync_onpkg_project()` to check for workspace files and output them in the project manifest `onpkg.json`:
Around line 850, add the workspaces detection block:
```rust
    let mut workspaces = Vec::new();
    
    // Check Cargo.toml workspace
    let cargo_toml_path = target_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(cargo_content) = fs::read_to_string(&cargo_toml_path) {
            if cargo_content.contains("[workspace]") {
                workspaces.push("cargo-workspace".to_string());
            }
        }
    }
    
    // Check pnpm-workspace.yaml
    if target_dir.join("pnpm-workspace.yaml").exists() {
        workspaces.push("pnpm-workspace".to_string());
    }
    
    // Check package.json workspaces
    let package_json_path = target_dir.join("package.json");
    if package_json_path.exists() {
        if let Ok(package_content) = fs::read_to_string(&package_json_path) {
            if package_content.contains("\"workspaces\"") {
                workspaces.push("npm-yarn-workspace".to_string());
            }
        }
    }
    
    // Include workspaces block in manifest serialization
    if !workspaces.is_empty() {
        manifest.insert("workspaces".to_string(), serde_json::json!(workspaces));
    }
```

- [ ] **Step 2: Verify and commit**

Run: `cargo test`
Commit: `git add . && git commit -m "feat: implement cargo, pnpm, and yarn workspaces detection in sync manifest"`

---

### Task 5: Secret Detection & Redaction

**Files:**
- Create: `src/secrets.rs`
- Modify: `src/packer.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `secrets::redact_secrets(content: &str) -> (String, Vec<(usize, String)>)`

- [ ] **Step 1: Create src/secrets.rs**

Create the file `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/secrets.rs`:
```rust
use regex::Regex;

pub fn redact_secrets(content: &str) -> (String, Vec<(usize, String)>) {
    let mut redacted = String::new();
    let mut warnings = Vec::new();
    
    // Basic regex matches for common credentials/tokens
    let openai_re = Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap();
    let github_re = Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap();
    
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let mut new_line = line.to_string();
        
        if openai_re.is_match(line) {
            new_line = openai_re.replace_all(&new_line, "[REDACTED-OPENAI-KEY]").to_string();
            warnings.push((line_num, "OpenAI API Key detected".to_string()));
        }
        
        if github_re.is_match(line) {
            new_line = github_re.replace_all(&new_line, "[REDACTED-GITHUB-PAT]").to_string();
            warnings.push((line_num, "GitHub PAT detected".to_string()));
        }
        
        redacted.push_str(&new_line);
        redacted.push('\n');
    }
    
    (redacted, warnings)
}
```

- [ ] **Step 2: Register secrets module in src/main.rs**

Add `pub mod secrets;` in `src/main.rs`.

- [ ] **Step 3: Integrate secrets scanning inside packer.rs**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/packer.rs`:
Update `pack_project` loop to scan and redact:
```rust
        let (clean_content, warnings) = crate::secrets::redact_secrets(&content);
        for (line, warning_desc) in warnings {
            eprintln!("⚠ WARNING: {} in {} on line {}", warning_desc, rel_path, line);
        }
        let content_to_pack = clean_content;
```

- [ ] **Step 4: Verify and commit**

Run: `cargo test`
Commit: `git add . && git commit -m "feat: implement regular expression secret detection and redaction in context packer"`
