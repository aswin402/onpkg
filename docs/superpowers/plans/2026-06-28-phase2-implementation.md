# Phase 2 — "Make It Intelligent" (v0.2.0) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Phase 2 features including git-aware file walking, code structure symbol mapping via tree-sitter, token-budgeted prompt packing, post-scaffold automation hooks, and version-aware environment diagnostics.

**Architecture:** Create modular, bounded components (`src/walker.rs`, `src/mapper.rs`, `src/packer.rs`), update the template engine for hooks execution, and add CLI subcommand routing in `src/cli.rs` and `src/main.rs`.

**Tech Stack:** Rust, tree-sitter, tiktoken-rs, ignore, dialoguer, rusqlite.

## Global Constraints
- Target Rust version: Edition 2021
- Avoid nested Tokio runtime panics by spawning blocking library calls in `tokio::task::spawn_blocking`.
- Follow the codebase's existing formatting conventions and async patterns.
- Do not add arbitrary external dependencies beyond those specified in the spec.

---

### Task 1: Git- & AI-Aware File Walking

**Files:**
- Modify: `Cargo.toml`
- Create: `src/walker.rs`
- Modify: `src/templates/mod.rs`
- Test: `tests/walker_test.rs`

**Interfaces:**
- Produces: `walker::get_project_walker(dir: &Path) -> Result<impl Iterator<Item = PathBuf>>`

- [ ] **Step 1: Add dependency to Cargo.toml**

Add the `ignore` crate under the `# File system utilities` section in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/Cargo.toml` (around line 46):
```toml
ignore = "0.4"
```

- [ ] **Step 2: Create src/walker.rs**

Create the file `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/walker.rs` with the following implementation:
```rust
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
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    paths.push(entry.path().to_path_buf());
                }
            }
            Err(e) => tracing::warn!("Error walking directory: {}", e),
        }
    }
    Ok(paths)
}
```

- [ ] **Step 3: Register walker module in src/main.rs**

Add `pub mod walker;` right after `pub mod updater;` in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs` (line 12).

- [ ] **Step 4: Update src/templates/mod.rs to use the new walker**

Modify the recursive file scanning loop in `sync_onpkg_project` in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/templates/mod.rs`. Replace lines 785 to 831 (the WalkDir walker) with the new `walker` call:
```rust
    let project_files = crate::walker::get_project_walker(target_dir)?;
    for path in project_files {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if allowed_exts.contains(&ext) {
                let rel_path = path.strip_prefix(target_dir).unwrap_or(&path);
                let rel_str = rel_path.to_string_lossy().to_string();
                if rel_str != "onpkg.json" && !rel_str.contains("onpkg_docs/") {
                    files.push(rel_str);
                    extensions.insert(ext.to_string());
                    if let Some(parent) = rel_path.parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        if !parent_str.is_empty() {
                            dirs.insert(parent_str);
                        }
                    }
                }
            }
        }
    }
```

- [ ] **Step 5: Write verification test**

Create `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/tests/walker_test.rs`:
```rust
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

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
    
    let walker_res = onpkg::walker::get_project_walker(path).unwrap();
    let file_names: Vec<String> = walker_res
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
        
    assert!(file_names.contains(&"keep.rs".to_string()));
    assert!(!file_names.contains(&"ignore_me.rs".to_string()));
}
```

- [ ] **Step 6: Verify and commit**

Run: `cargo test --test walker_test`
Expected: PASS
Commit: `git add . && git commit -m "feat: implement git- and ignore-aware project walker"`

---

### Task 2: Code Structure Mapping (`onpkg map`)

**Files:**
- Modify: `Cargo.toml`
- Create: `src/mapper.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Test: `tests/mapper_test.rs`

**Interfaces:**
- Produces: `mapper::map_project(dir: &Path) -> Result<Vec<FileOutline>>`
- Produces: `mapper::format_markdown(outlines: &[FileOutline]) -> String`
- Produces: `mapper::format_json(outlines: &[FileOutline]) -> Result<String>`

- [ ] **Step 1: Add tree-sitter dependencies**

Add to `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/Cargo.toml` under dependencies:
```toml
tree-sitter = "0.22"
tree-sitter-rust = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-typescript = "0.21"
tree-sitter-python = "0.21"
```

- [ ] **Step 2: Create src/mapper.rs**

Create the file `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/mapper.rs` containing tree-sitter queries for Rust, TS/JS, and Python:
```rust
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Serialize, Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct FileOutline {
    pub file: String,
    pub language: String,
    pub symbols: Vec<Symbol>,
}

pub fn map_project(dir: &Path) -> Result<Vec<FileOutline>> {
    let files = crate::walker::get_project_walker(dir)?;
    let mut outlines = Vec::new();
    
    for f in files {
        let rel_path = f.strip_prefix(dir).unwrap_or(&f).to_string_lossy().to_string();
        if let Some(ext) = f.extension().and_then(|s| s.to_str()) {
            let outline = match ext {
                "rs" => parse_file(&f, &rel_path, "rust", tree_sitter_rust::LANGUAGE.into(), "(struct_item name: (type_identifier) @name) @kind (enum_item name: (type_identifier) @name) @kind (function_item name: (identifier) @name) @kind (impl_item) @kind (trait_item name: (type_identifier) @name) @kind")?,
                "py" => parse_file(&f, &rel_path, "python", tree_sitter_python::LANGUAGE.into(), "(class_definition name: (identifier) @name) @kind (function_definition name: (identifier) @name) @kind")?,
                "ts" | "tsx" => parse_file(&f, &rel_path, "typescript", tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), "(class_declaration name: (type_identifier) @name) @kind (function_declaration name: (identifier) @name) @kind (interface_declaration name: (type_identifier) @name) @kind (lexical_declaration (variable_declarator name: (identifier) @name value: (arrow_function))) @kind")?,
                "js" | "jsx" => parse_file(&f, &rel_path, "javascript", tree_sitter_javascript::LANGUAGE.into(), "(class_declaration name: (identifier) @name) @kind (function_declaration name: (identifier) @name) @kind")?,
                _ => continue,
            };
            if !outline.symbols.is_empty() {
                outlines.push(outline);
            }
        }
    }
    Ok(outlines)
}

fn parse_file(
    path: &Path,
    rel_path: &str,
    lang_name: &str,
    lang: tree_sitter::Language,
    query_str: &str,
) -> Result<FileOutline> {
    let source_code = std::fs::read_to_string(path)?;
    let mut parser = Parser::new();
    parser.set_language(&lang)?;
    
    let tree = parser.parse(&source_code, None)
        .ok_or_else(|| anyhow!("Failed to parse {}", rel_path))?;
        
    let query = Query::new(&lang, query_str)?;
    let mut cursor = QueryCursor::new();
    let mut symbols = Vec::new();
    
    for m in cursor.matches(&query, tree.root_node(), source_code.as_bytes()) {
        let mut name = String::new();
        let mut kind = "symbol".to_string();
        let mut line = 0;
        
        for capture in m.captures {
            let node = capture.node;
            line = node.start_position().row + 1;
            let text = node.utf8_text(source_code.as_bytes()).unwrap_or("").to_string();
            
            if capture.index == 0 {
                // Name capture
                name = text;
            } else {
                // Kind capture
                kind = node.kind().to_string();
            }
        }
        if !name.is_empty() {
            symbols.push(Symbol { name, kind, line });
        }
    }
    
    Ok(FileOutline {
        file: rel_path.to_string(),
        language: lang_name.to_string(),
        symbols,
    })
}

pub fn format_markdown(outlines: &[FileOutline]) -> String {
    let mut md = String::new();
    for o in outlines {
        md.push_str(&format!("- **{}** ({})\n", o.file, o.language));
        for s in &o.symbols {
            md.push_str(&format!("  - [{}] {} (line {})\n", s.kind, s.name, s.line));
        }
    }
    md
}

pub fn format_json(outlines: &[FileOutline]) -> Result<String> {
    Ok(serde_json::to_string_pretty(outlines)?)
}
```

- [ ] **Step 3: Register mapper module in src/main.rs**

Add `pub mod mapper;` right after `pub mod walker;` in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`.

- [ ] **Step 4: Add CLI mapping command definitions**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs`, add the `Map` command:
1. In `enum Command`, append:
```rust
    /// Map project directory structure and extract code symbols
    Map {
        /// Target directory
        dir: Option<String>,
        /// Output format (markdown, json)
        #[arg(long, default_value = "markdown")]
        format: String,
        /// Save output to file path instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },
```

- [ ] **Step 5: Wire Command::Map in src/main.rs**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`, add the matching arm in `main()` function's subcommand dispatch loop:
```rust
        Command::Map { dir, format, output } => {
            let target = dir.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            let sp = TUI::spinner("Mapping project code structure...");
            let result = tokio::task::spawn_blocking(move || {
                mapper::map_project(&target)
            }).await??;
            sp.finish_and_clear();
            
            let content = if format == "json" {
                mapper::format_json(&result)?
            } else {
                mapper::format_markdown(&result)
            };
            
            if let Some(out_path) = output {
                std::fs::write(&out_path, content)?;
                TUI::success(&format!("Codebase map written to {}", out_path), None);
            } else {
                println!("{}", content);
            }
        }
```

- [ ] **Step 6: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement codebase tree-sitter symbol mapping"`

---

### Task 3: Context Packing (`onpkg pack`)

**Files:**
- Modify: `Cargo.toml`
- Create: `src/packer.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `packer::pack_project(dir: &Path, max_tokens: usize) -> Result<PackResult>`

- [ ] **Step 1: Add tiktoken-rs dependency**

Add to `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/Cargo.toml` under dependencies:
```toml
tiktoken-rs = "0.6"
```

- [ ] **Step 2: Create src/packer.rs**

Create `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/packer.rs`:
```rust
use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use tiktoken_rs::cl100k_base;

#[derive(Serialize, Debug)]
pub struct PackResult {
    pub content: String,
    pub token_count: usize,
    pub file_count: usize,
    pub skipped_files: Vec<String>,
}

pub fn pack_project(dir: &Path, max_tokens: usize) -> Result<PackResult> {
    let bpe = cl100k_base()?;
    let files = crate::walker::get_project_walker(dir)?;
    
    let mut packed_content = String::new();
    let mut file_count = 0;
    let mut skipped_files = Vec::new();
    
    // 1. Generate Folder tree structure
    packed_content.push_str("# Project Directory Structure\n```\n");
    for f in &files {
        let rel_path = f.strip_prefix(dir).unwrap_or(f).to_string_lossy().to_string();
        packed_content.push_str(&format!("{}\n", rel_path));
    }
    packed_content.push_str("```\n\n");
    
    // 2. Pack file contents
    for f in files {
        let rel_path = f.strip_prefix(dir).unwrap_or(&f).to_string_lossy().to_string();
        let content = std::fs::read_to_string(&f)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let file_representation = if lines.len() < 200 {
            format!("## File: {}\n```\n{}\n```\n\n", rel_path, content)
        } else {
            // Outlines representation using mapper
            let relative_dir = f.parent().unwrap_or(Path::new(""));
            let single_outline_res = crate::mapper::map_project(relative_dir)?;
            let matching_outline = single_outline_res.iter().find(|o| o.file == rel_path);
            
            if let Some(out) = matching_outline {
                format!("## File: {} (Symbol Outline only - file is >= 200 lines)\n```\n{}\n```\n\n", 
                    rel_path, crate::mapper::format_markdown(std::slice::from_ref(out)))
            } else {
                format!("## File: {} (Skipped content - file is >= 200 lines)\n\n", rel_path)
            }
        };
        
        let tokens = bpe.encode_with_special_tokens(&file_representation);
        let current_tokens = bpe.encode_with_special_tokens(&packed_content);
        
        if current_tokens.len() + tokens.len() < max_tokens {
            packed_content.push_str(&file_representation);
            file_count += 1;
        } else {
            skipped_files.push(rel_path);
        }
    }
    
    let final_tokens = bpe.encode_with_special_tokens(&packed_content).len();
    Ok(PackResult {
        content: packed_content,
        token_count: final_tokens,
        file_count,
        skipped_files,
    })
}
```

- [ ] **Step 3: Register packer module in src/main.rs**

Add `pub mod packer;` right after `pub mod mapper;` in `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`.

- [ ] **Step 4: Add CLI context packing command definitions**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs`, add the `Pack` command:
1. In `enum Command`, append:
```rust
    /// Pack codebase structure and files into a token-budgeted prompt context
    Pack {
        /// Target directory
        dir: Option<String>,
        /// Token budget limit
        #[arg(long, default_value = "100000")]
        max_tokens: usize,
        /// Save packed context file path
        #[arg(short, long, default_value = "onpkg-context.md")]
        output: String,
    },
```

- [ ] **Step 5: Wire Command::Pack in src/main.rs**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`, add the subcommand handling for `Command::Pack`:
```rust
        Command::Pack { dir, max_tokens, output } => {
            let target = dir.map(std::path::PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
            let sp = TUI::spinner("Packing project context...");
            
            let result = tokio::task::spawn_blocking(move || {
                packer::pack_project(&target, max_tokens)
            }).await??;
            sp.finish_and_clear();
            
            std::fs::write(&output, &result.content)?;
            TUI::success(&format!("Context packed successfully into {}", output), None);
            TUI::info(&format!("Tokens: {} | Files embedded: {} | Skipped: {}", 
                result.token_count, result.file_count, result.skipped_files.len()));
        }
```

- [ ] **Step 6: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement smart context packing command"`

---

### Task 4: Post-Scaffold Hooks

**Files:**
- Modify: `src/stacks.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `StackSubcommand::Add` scaffolding workflow

- [ ] **Step 1: Add StackHook structure and field definitions**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/stacks.rs`:
1. Add `StackHook` struct:
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StackHook {
    pub command: String,
    pub description: Option<String>,
}
```
2. Modify `Stack` struct. Add `#[serde(default)] pub hooks: Vec<StackHook>` (around line 20):
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stack {
    pub name: String,
    pub runtime: String,
    pub description: String,
    pub packages: Vec<String>,
    pub dev_packages: Vec<String>,
    #[serde(default)]
    pub transitive_packages: Vec<String>,
    pub files: Vec<StackFile>,
    #[serde(default)]
    pub hooks: Vec<StackHook>,
}
```

- [ ] **Step 2: Add --no-hooks flag to CLI commands**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs`, add `--no-hooks` flag to `StackSubcommand::Add` and `StackSubcommand::Use` (around lines 238-265):
```rust
        /// Disable running post-scaffold hooks (e.g. git init)
        #[arg(long)]
        no_hooks: bool,
```

- [ ] **Step 3: Execute hooks sequentially after stack add**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`, modify `StackSubcommand::Add` / `StackSubcommand::Use` destructuring to bind `no_hooks`:
```rust
            StackSubcommand::Add {
                name,
                dir,
                var,
                manager,
                no_hooks,
            }
            | StackSubcommand::Use {
                name,
                dir,
                var,
                no_hooks,
                manager,
            } => {
```

Then, after scaffolding completes (right before docs generation, around line 665), execute the hooks:
```rust
                if !no_hooks && !tmpl.hooks.is_empty() {
                    println!();
                    TUI::info("Executing post-scaffold hooks...");
                    for hook in &tmpl.hooks {
                        let desc = hook.description.as_deref().unwrap_or(&hook.command);
                        let hook_sp = TUI::spinner(&format!("Running hook: {}...", desc));
                        
                        let hook_res = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(&hook.command)
                            .current_dir(&target)
                            .output();
                            
                        hook_sp.finish_and_clear();
                        match hook_res {
                            Ok(output) if output.status.success() => {
                                TUI::success(&format!("Hook completed: {}", desc), None);
                            }
                            Ok(output) => {
                                TUI::warn(&format!("Hook failed: {} (code: {:?})", desc, output.status.code()));
                            }
                            Err(e) => {
                                TUI::warn(&format!("Failed to spawn hook: {} ({})", desc, e));
                            }
                        }
                    }
                }
```

- [ ] **Step 4: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement post-scaffold stack hooks execution"`

---

### Task 5: Version-Aware Doctor Diagnostics

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Check runtime version requirements**

In `/home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs`, update the Doctor subcommand (around lines 510-530) to parse and validate semantic versions:
Replace the loop checking node/bun/python/cargo with checks that parse version requirements:
```rust
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
                        // Parse semver if requirement exists
                        let status = if let Some(req) = min_req {
                            if let Ok(parsed_ver) = semver::Version::parse(&raw_ver.split('-').next().unwrap_or("0.0.0")) {
                                if req.matches(&parsed_ver) {
                                    "ok"
                                } else {
                                    "warn"
                                }
                            } else {
                                "ok" // If parsing fails (e.g. customized versions), default to ok
                            }
                        } else {
                            "ok"
                        };
                        diagnostics.push(serde_json::json!({"check": name, "status": status, "detail": raw_ver}));
                    }
                    Err(_) => diagnostics.push(serde_json::json!({"check": name, "status": "missing", "detail": "not found on PATH"})),
                }
            }
```

- [ ] **Step 2: Add Database integrity check to Doctor**

Also add a SQLite integrity check block (inside the database check segment):
```rust
            // Database integrity check
            match db.count_packages() {
                Ok(count) => {
                    // Check PRAGMA integrity_check
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
                Err(e) => diagnostics.push(serde_json::json!({"check": "database", "status": "error", "detail": e.to_string()})),
            }
```

- [ ] **Step 3: Verify and commit**

Run: `cargo check`
Commit: `git add . && git commit -m "feat: implement semantic version checking & db integrity in doctor command"`
