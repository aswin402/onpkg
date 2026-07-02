# onpkg — Task Tracker (todo.md) 📋

> Version: v0.1.0 → v0.3.0 Roadmap
> Last updated: 2026-06-28

---

## ✅ Completed (v0.1.0)
- [x] Core CLI with clap derive
- [x] Stack scaffolding engine (builtin + custom TOML)
- [x] Skill management (install/remove/list/show)
- [x] Package caching with SQLite
- [x] `onpkg sync` — manifest generation
- [x] `onpkg_docs/` AI skill set generation
- [x] Multi-runtime detection (bun/npm/uv/cargo/flutter)
- [x] AI-powered skill/template generation (`onpkg ai`)
- [x] Registry search/info (read-only)
- [x] TUI with spinners, colors, and ASCII art
- [x] Built-in stacks: react-vite, next, fastapi, flutter, hono, mern, pern
- [x] 14+ built-in technology skills

---

## ✅ Phase 1 — "Make It Complete" (v0.1.1) — DONE
> Completed: 2026-06-28

### 1.1 Self-Update (`onpkg update`)
- [x] Add `self_update` crate to `Cargo.toml` (default-features = false, rustls)
- [x] Create `src/updater.rs` module
  - [x] `check_and_update(current_version)` function
  - [x] GitHub Releases backend configuration
  - [x] Download progress bar integration
- [x] Replace `Command::Update` match arm in `src/main.rs`
- [x] Handle error cases (no network, no releases, permission denied)
- [x] Fix: use `spawn_blocking` to avoid nested tokio runtime panic
- [x] Test: `onpkg update` shows graceful error or updates

### 1.2 `AGENTS.md` Generation
- [x] Add `generate_agents_md()` function in `src/templates/mod.rs`
  - [x] Project name, runtime, package manager, technologies
  - [x] Build/dev/test commands section
  - [x] Architecture paths section
  - [x] User-editable "Agent-Specific Notes" section
  - [x] Preserve existing user content on re-sync
- [x] Call from `sync_onpkg_project()` after INDEX.md write
- [x] Add `--no-agents-md` flag to `Sync` command
- [x] Optionally symlink `CLAUDE.md → AGENTS.md`
- [x] Test: `onpkg sync` creates `AGENTS.md` at project root

### 1.3 Interactive Stack Selection
- [x] Add `dialoguer` crate to `Cargo.toml` (with `fuzzy-select` feature)
- [x] Make `name` optional in `StackSubcommand::Add` (`src/cli.rs`)
- [x] Make `name` optional in `StackSubcommand::Use` (`src/cli.rs`)
- [x] Add fuzzy selector when no name provided (`src/main.rs`)
  - [x] List all templates with name, category, description
  - [x] FuzzySelect prompt with default selection
- [ ] Also add interactive mode for `onpkg skill install` (no args) _(deferred to v0.2.0)_
- [x] Test: `onpkg stack add` (no args) → shows fuzzy picker

### 1.4 `--json` Structured Output
- [x] Add `--json` global flag to `Args` struct in `src/cli.rs`
- [x] Implement JSON output branch for:
  - [x] `onpkg stack list --json`
  - [x] `onpkg skill list --json`
  - [x] `onpkg pkg list --json`
  - [x] `onpkg doctor --json`
  - [ ] `onpkg stack show <name> --json` _(deferred to v0.2.0)_
  - [ ] `onpkg skill show <name> --json` _(deferred to v0.2.0)_
- [x] Suppress TUI chrome when `--json` is active
- [x] Test: `onpkg stack list --json | python3 -m json.tool` → valid JSON

### 1.5 Clean Up Dead Registry Publish
- [x] Replace `TemplateSubcommand::Publish` with roadmap message
- [x] Replace `SkillSubcommand::Publish` with roadmap message
- [x] Update help text to explain publishing timeline (v0.3.0)

---

## 🟡 Phase 2 — "Make It Intelligent" (v0.2.0)
## ✅ Phase 2 — "Make It Intelligent" (v0.2.0) — DONE
> Completed: 2026-06-28

### 2.1 `onpkg map` — Code Structure Analysis
- [x] Add tree-sitter crates to `Cargo.toml`:
  - [x] `tree-sitter`, `tree-sitter-javascript`, `tree-sitter-typescript`
  - [x] `tree-sitter-python`, `tree-sitter-rust`
- [x] Create `src/mapper.rs` module
  - [x] `CodeMap`, `FileOutline`, `Symbol` structs (serializable)
  - [x] `detect_language(path)` — extension → language mapping
  - [x] `parse_file(path, language)` — extract symbols via tree-sitter
  - [x] `map_project(dir)` — walk + parse all supported files
  - [x] `format_markdown(map)` — human-readable outline
  - [x] `format_json(map)` — machine-readable outline
- [x] Tree-sitter queries per language:
  - [x] Rust: `fn`, `pub fn`, `struct`, `enum`, `impl`, `trait`, `mod`
  - [x] TypeScript/JS: `function`, `class`, `export`, `const`, `interface`
  - [x] Python: `def`, `class`, `async def`
- [x] Add `Map` command to `src/cli.rs` (dir, format, output flags)
- [x] Add `Command::Map` handling in `src/main.rs`
- [x] Use `ignore` crate for file walking (see 2.3)
- [x] Test: `onpkg map --format json` → valid structured output

### 2.2 `onpkg pack` — Context Packing
- [x] Add `tiktoken-rs` crate to `Cargo.toml`
- [x] Create `src/packer.rs` module
  - [x] `PackOptions` struct (format, full, max_tokens, exclude)
  - [x] `PackResult` struct (content, token_count, file_count)
  - [x] `pack_project(dir, opts)` — main packing function
  - [x] `count_tokens(text)` — cl100k_base tokenizer
  - [x] Directory tree generation
  - [x] Full content for small files (<200 lines)
  - [x] Outline-only for large files (use mapper)
  - [x] Include `onpkg.json` and `AGENTS.md` if present
  - [x] `--max-tokens` budget enforcement with truncation
- [x] Add `Pack` command to `src/cli.rs`
- [x] Add `Command::Pack` handling in `src/main.rs`
- [x] Output formats: markdown, xml, json
- [x] Add `--exclude` patterns flag
- [x] Display token summary at end
- [x] Test: `onpkg pack` → creates `onpkg-context.md` with token count

### 2.3 Git-Aware File Walking
- [x] Add `ignore` crate to `Cargo.toml`
- [x] Replace `WalkDir` with `WalkBuilder` in `sync_onpkg_project()`
  - [x] Enable `.gitignore` respect
  - [x] Enable global gitignore
  - [x] Enable `.git/info/exclude`
  - [x] Remove hardcoded dir exclusion list
- [x] Use `ignore` in `mapper.rs` for `map_project()`
- [x] Use `ignore` in `packer.rs` for `pack_project()`
- [x] Keep `walkdir` for `add_from_dir` (template ingestion, non-git-aware)
- [x] Test: Files in `.gitignore` are excluded from sync/map/pack

### 2.4 Post-Scaffold Hooks
- [x] Add `StackHook` struct to `src/stacks.rs`:
  - [x] `command: String`
  - [x] `description: String` (optional)
- [x] Add `hooks: Vec<StackHook>` to `Stack` struct
- [x] Add `hooks: Vec<StackHook>` to `TemplateDefinition` struct
- [x] Execute hooks after scaffold in `Command::Stack::Add` handler:
  - [x] Sequential execution with `sh -c` / `cmd /C`
  - [x] Spinner per hook
  - [x] Success/failure reporting
  - [x] Continue on failure (warn, don't abort)
- [x] Add default hooks to built-in stacks:
  - [x] All stacks: `git init`
  - [x] Next/React: `cp .env.example .env` (if example exists)
- [x] Support hooks in custom TOML templates
- [x] Test: `onpkg stack add react-vite` → git init runs automatically

### 2.5 Improved Doctor Command
- [x] Check binary versions, not just existence:
  - [x] Node.js version >= 18
  - [x] Bun version >= 1.0
  - [x] Python version >= 3.10
  - [x] Rust/cargo version
  - [x] Flutter/dart version
- [x] Check for `onpkg.json` in current directory
- [x] Check database integrity
- [x] Warn about outdated onpkg version (compare with latest release)
- [x] Add `--json` output support
- [x] Test: `onpkg doctor` shows version numbers and compatibility warnings

---

## ✅ Phase 3 — "Make It Indispensable" (v0.3.0) — DONE
> Completed: 2026-06-28

### 3.1 MCP Server Mode
- [x] Create `src/mcp.rs` module with stdio JSON-RPC 2.0
- [x] Define MCP tools: `stack_list`, `stack_add`, `skill_list`, `skill_install`, `sync`, `map`, `pack`, `doctor`
- [x] Tool handlers route dynamically to current binary
- [x] Add `Serve` command to `src/cli.rs`
- [x] Implement stdio transport (default)
- [x] Document MCP configuration for Claude Code / Gemini CLI
- [x] Test: Verify JSON-RPC initialize and tool call flows

### 3.2 Watch Mode (`onpkg sync --watch`)
- [x] Add `notify` crate to `Cargo.toml`
- [x] Add `--watch` flag to `Sync` command
- [x] File watcher with 2-second debounce
- [x] Re-run `sync_onpkg_project()` on file changes
- [x] Update `AGENTS.md` and `onpkg.json` in real-time
- [x] Graceful shutdown on Ctrl+C

### 3.3 Template Diff/Upgrade
- [x] Add `similar` crate to `Cargo.toml`
- [x] Add `StackSubcommand::Diff` command
- [x] Compare current project files against template definition
- [x] Show added/removed/modified files using colorized git-style diffs
- [x] Optional `--apply` flag to merge template updates
- [x] Handle conflict resolution (keep user changes vs take template)
- [x] Test: Verify diffing against original template definitions

### 3.4 Monorepo/Workspace Detection
- [x] Detect workspace types in `sync_onpkg_project()`:
  - [x] `pnpm-workspace.yaml` → pnpm workspaces
  - [x] `package.json` "workspaces" field → npm/yarn
  - [x] `Cargo.toml` `[workspace]` → cargo
  - [x] `apps/` + `packages/` dirs → turborepo-style
- [x] Add `"workspaces"` field to `onpkg.json` output
- [x] Per-workspace sync (scan each workspace member)

### 3.5 Secret Detection During Pack/Sync
- [x] Define secret regex patterns:
  - [x] API keys, tokens, passwords
  - [x] GitHub PATs, OpenAI keys, Google API keys
  - [x] AWS credentials, Stripe keys
- [x] Scan file contents before including in pack output
- [x] Warn with file:line references
- [x] Auto-redact secrets in pack output (replace with `[REDACTED]`)
- [x] Optional `--no-redact` flag to skip
- [x] Test: Unit tests verifying OpenAI/GitHub/generic keys redaction

### 3.6 Registry Publishing (v0.3.0) — DONE
- [x] Design registry API contract (REST endpoints)
- [x] Implement `publish_template()` in `src/registry.rs`
- [x] Implement `publish_skill()` in `src/registry.rs`
- [x] Authentication (API key or GitHub OAuth)
- [x] Package validation before publish
- [x] Version conflict detection

---

## 🔧 Infrastructure & CI/CD
- [ ] Set up `cargo-dist` for automated GitHub Releases (Deferred - custom GHA preferred for standalone virtual package)
- [x] GitHub Actions workflow: build → test → release
- [x] Cross-platform builds (Linux, macOS, Windows)
- [x] Release asset naming convention for `self_update`
- [ ] Changelog automation with `git-cliff`
- [x] Add integration tests for new commands
- [x] Add unit tests for mapper, packer, updater modules

---

## 📊 Progress Summary

| Phase | Status | Features | Completion |
|-------|--------|----------|------------|
| v0.1.0 | ✅ Done | Core scaffolding, skills, sync | 100% |
| v0.1.1 (Phase 1) | ✅ Done | Update, AGENTS.md, interactive, JSON, cleanup | 100% |
| v0.2.0 (Phase 2) | ✅ Done | Map, pack, gitignore, hooks, doctor | 100% |
| v0.3.0 (Phase 3) | ✅ Done | MCP, watch, diff, monorepo, secrets (publishing deferred) | 100% |
