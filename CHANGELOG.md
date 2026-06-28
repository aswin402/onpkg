# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-06-28

### Added
- **Codebase Symbol Mapping (`onpkg map`)**: Extracts structure and symbol definitions for Rust, Python, JavaScript, and TypeScript files using `tree-sitter`. Supports formatted Markdown and machine-readable JSON outputs.
- **Smart Context Packing (`onpkg pack`)**: Bundles workspace files into a token-budgeted context file for LLMs using `tiktoken-rs` (cl100k_base). Employs a hybrid packing strategy (substitutes files >= 200 lines with their tree-sitter symbol outlines to conserve token limits).
- **Git- & AI-Aware Walking**: Walker respects `.gitignore`, global git rules, `.git/info/exclude`, and custom local `.onpkgignore` files, with built-in overrides for heavy folders (`node_modules`, `target`, `.venv`, etc.).
- **Post-Scaffold Hooks**: Execution of automated hook commands defined in stacks (e.g. `git init`), featuring unix/windows shell portability and `--no-hooks` opt-out.
- **Improved Doctor Command**: Checks environment versions against semver constraints (Node.js >=18.0.0, Bun >=1.0.0) and runs SQLite database diagnostics (`PRAGMA integrity_check`).

## [0.1.1] - 2026-06-28

### Added
- **Self-Update (`onpkg update`)**: Added true self-updating binary capability via `self_update` crate pulling from GitHub Releases (`aswin402/onpkg`). Integrated with `tokio::task::spawn_blocking` to avoid nested runtime panics.
- **Universal AI Agent Context (`AGENTS.md`)**: Automatically generates `AGENTS.md` in the project root during `onpkg sync`. Includes project runtime, package manager, build/run commands, architecture entrypoints/folders, and a user-editable `## Agent-Specific Notes` section that is preserved across re-syncs.
- **Interactive Stack Selection**: If `onpkg stack add` or `onpkg stack use` is invoked without a stack name, users are presented with an interactive fuzzy-select list of all available stacks utilizing the `dialoguer` crate.
- **Structured JSON Output (`--json`)**: Added a global `--json` flag for machine consumption. Implemented clean JSON outputs for:
  - `onpkg stack list --json`
  - `onpkg skill list --json`
  - `onpkg pkg list --json`
  - `onpkg doctor --json`

### Changed
- **Publish Commands Cleanup**: Replaced incomplete template/skill registry publishing stubs with descriptive warning messages detailing the v0.3.0 roadmap and helpful workarounds for sharing templates/skills.
- **CLI Arg Parsing**: Updated Clap schema to support optional names for stack commands and global `--json` argument.

### Fixed
- Fixed nested Tokio blocking-runtime panics inside asynchronous context during update checks by running `self_update` in a spawned blocking thread.
