# Spec: onpkg Phase 3 — "Make It Indispensable" (v0.3.0)

This document outlines the detailed architectural design and specifications for Phase 3 of `onpkg` (v0.3.0), introducing a custom Model Context Protocol (MCP) server transport, debounced file watch synchronization, template diffing & upgrading, workspace/monorepo detection, and token context secret redaction.

---

## 1. Custom stdio MCP Server (`onpkg serve`)

### Purpose
Expose the capabilities of `onpkg` directly to LLM agents (e.g. Cursor, Claude Desktop, Gemini CLI) as standard tool definitions over standard input/output.

### Protocol Details
- Standard JSON-RPC 2.0 protocol over stdio.
- Implements:
  - **`initialize`**: Exchange client info, capabilities, and tool names.
  - **`tools/list`**: Returns the list of tools containing description and parameter schemas matching CLI arguments.
  - **`tools/call`**: Executes the matching command handler, redirects stdout/stderr internally, and returns output as text content inside the standard JSON-RPC envelope.
- Tools exposed:
  - `stack_list`: Parameters matching `stack list`
  - `stack_add`: Parameters matching `stack add` (name, dir, manager, no_hooks)
  - `skill_list`: Parameters matching `skill list`
  - `skill_install`: Parameters matching `skill install` (name)
  - `sync`: Parameters matching `sync` (dir)
  - `map`: Parameters matching `map` (dir, format)
  - `pack`: Parameters matching `pack` (dir, max_tokens)
  - `doctor`: Parameters matching `doctor`

---

## 2. Debounced File Watcher (`onpkg sync --watch`)

### Purpose
Keep the `onpkg.json` project manifest and `AGENTS.md` automatically updated in real-time as the developer/agent modifies project files.

### Design
- Integrates the `notify` crate to listen for file system events in the target project directory.
- Filters out events from ignored folders (`node_modules/`, `target/`, `.git/`, `.venv/`, `dist/`, `.next/`, `onpkg_docs/`) to prevent loop triggers.
- Debounces file events: when an event triggers, schedules a 2-second idle window timer. If new events arrive within the window, resets the timer. Once the 2-second idle window passes without new events, runs `sync_onpkg_project` to update manifest files.
- Graceful shutdown handles `Ctrl+C` cleanly to restore terminal state.

---

## 3. Template Diff & Upgrades (`onpkg stack diff`)

### Purpose
Allow developers to see differences between their current scaffolded workspace files and the original stack definition, with the option to pull in updates.

### CLI Schema
```bash
onpkg stack diff [name] [options]
```
- `[name]`: Optional stack template name (defaults to current project stack name resolved from `onpkg.json`).
- `--apply`: Apply template updates, overwriting differences in the workspace.

### Behavior
- Resolves the matching stack definition.
- For each file in the stack definition:
  - If it exists in the workspace, calculates line-by-line diff using the `similar` crate.
  - Displays added/deleted/modified hunks with standard unified-diff coloring (green/red).
- When `--apply` is passed:
  - Writes modified files to disk.
  - Warns before overwriting files with user modifications.

---

## 4. Monorepo & Workspace Detection

### Purpose
Support workspace monorepos by listing all sub-packages and directories in `onpkg.json`.

### Scanning Logic
Inside `sync_onpkg_project`:
- Detects the following configuration files:
  - `pnpm-workspace.yaml` (pnpm)
  - `package.json` with `"workspaces"` field (npm/yarn)
  - `Cargo.toml` with `[workspace]` block (Rust)
- Parses the member paths.
- Adds a `"workspaces"` field containing a list of sub-package directories to `onpkg.json`.

---

## 5. Secret Detection & Redaction

### Purpose
Ensure that sensitive tokens, keys, and credentials are never packed into context files generated for LLM prompts.

### Detection Patterns
Scan file contents line-by-line using regular expressions for common API key signatures:
- OpenAI API Keys: `sk-[a-zA-Z0-9]{48}`
- GitHub PATs: `ghp_[a-zA-Z0-9]{36}` or `github_pat_[a-zA-Z0-9_]{82}`
- Generic API Keys / Tokens: `[a-zA-Z0-9_-]{32,64}` assigned to keys like `api_key`, `token`, `secret`, `password`.
- AWS Credentials / Private Keys.

### Behavior during `onpkg pack`
- When a secret is detected:
  - Output warning message to `stderr` indicating the filename and line number.
  - Replace the secret text with `[REDACTED]` in the output context payload.
  - Supports `--no-redact` option to skip the check.
