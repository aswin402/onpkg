# Spec: onpkg Phase 2 — "Make It Intelligent" (v0.2.0)

This document outlines the detailed architectural design and specifications for Phase 2 of `onpkg` (v0.2.0), introducing tree-sitter codebase mapping, token-aware context packing, git-aware file walking, post-scaffold hooks, and improved environment diagnostics.

---

## 1. Git- & AI-Aware File Walking

### Purpose
Provide a single, consistent way to walk directories across all commands (`sync`, `map`, `pack`), respecting standard `.gitignore` rules, global ignores, and AI-specific exclusions via `.onpkgignore`.

### Behavior & Rules
- Uses the `ignore` crate's `WalkBuilder` to walk directories.
- Always respects:
  - Standard `.gitignore` files.
  - Global gitignore files (e.g. `~/.config/git/ignore`).
  - `.git/info/exclude`.
  - A custom local `.onpkgignore` file located in the project root.
- The `.onpkgignore` file supports standard glob patterns (same syntax as `.gitignore`).
- If no ignore files are present, falls back to default system exclusions:
  - `.git/`
  - `node_modules/`
  - `target/`
  - `dist/` / `build/` / `.next/`
  - `.venv/` / `venv/` / `__pycache__/`

---

## 2. Code Structure Analysis (`onpkg map`)

### Purpose
Generate a high-level representation of a codebase's modules, types, functions, and interfaces using fast tree-sitter parsing. This acts as a map for both developers and AI agents.

### Subcommand CLI Schema
```bash
onpkg map [dir] [options]
```
- `[dir]`: Target directory (defaults to current directory).
- `--format <markdown|json>`: Output format (defaults to `markdown`).
- `-o, --output <file>`: Output path (prints to stdout if omitted).

### Supported Languages & Queries
Utilizes `tree-sitter` and language-specific grammar crates:

1. **Rust (`.rs`)**:
   - Query symbols: `function` (`fn`, `pub fn`), `struct`, `enum`, `impl` block, `trait`, `module`.
2. **JavaScript/TypeScript (`.js`, `.jsx`, `.ts`, `.tsx`)**:
   - Query symbols: `class`, `function` (named declarations & arrow const functions), `interface`, `type` alias, `export` declarations.
3. **Python (`.py`)**:
   - Query symbols: `class`, `def`, `async def` function/method definitions.
4. **Configuration Files (`.json`, `.toml`)**:
   - Custom lightweight parser to list top-level configuration keys and structural nodes.

### Output Formatting
- **Markdown (`markdown`)**: Tree outline with nested bullet points and type labels:
  ```markdown
  - src/main.rs
    - [struct] Args
    - [fn] main()
  ```
- **JSON (`json`)**: Structured array of file outlines:
  ```json
  [
    {
      "file": "src/main.rs",
      "language": "rust",
      "symbols": [
        { "name": "Args", "kind": "struct", "line": 15 },
        { "name": "main", "kind": "function", "line": 28 }
      ]
    }
  ]
  ```

---

## 3. Smart Context Packing (`onpkg pack`)

### Purpose
Compile the files and structures of a project into a single token-budgeted prompt context optimized for LLMs.

### Subcommand CLI Schema
```bash
onpkg pack [dir] [options]
```
- `[dir]`: Target directory (defaults to current directory).
- `--max-tokens <number>`: Maximum token budget (defaults to `100000`).
- `-o, --output <file>`: Output path (defaults to `onpkg-context.md` in current directory).
- `--exclude <patterns>`: Comma-separated list of additional glob patterns to exclude.

### Token Counting
- Utilizes the `tiktoken-rs` library with the `cl100k_base` encoding (compatible with GPT-4, Claude 3/3.5, Gemini 1.5/2.5 tokenization).

### Hybrid Packing Strategy
1. **Walk project** using Git- & AI-aware file walker.
2. **Estimate budget**:
   - Layout a directory tree structure.
   - For each file:
     - If the file is `< 200 lines`, pack the entire contents inside a markdown code block.
     - If the file is `>= 200 lines`, run `onpkg map` to extract its symbol outline and embed only the outline instead of full source code.
3. **Budget enforcement**:
   - If the accumulated tokens exceed the `--max-tokens` limit, stop packing files.
   - Append a summary listing files that were successfully packed and files that were skipped due to the budget limit.

---

## 4. Post-Scaffold Hooks

### Purpose
Automate initial workspace setup steps right after stack scaffolding (such as initializing git repositories or copying default environment variables).

### Manifest Changes (`stacks.rs`)
- Introduce a new struct `StackHook` and update templates:
  ```rust
  pub struct StackHook {
      pub command: String,
      pub description: Option<String>,
  }
  ```
- Add a `hooks: Vec<StackHook>` field to the main `Stack` and `TemplateDefinition` structures.

### Execution Policy
- Hooks run sequentially in the target directory using `std::process::Command` with `sh -c` (or `cmd.exe /c` on Windows).
- A terminal spinner indicates hook progress.
- Opt-out is supported via the `--no-hooks` command-line flag during `onpkg stack add` or `onpkg stack use`.
- Failed hooks output a warning banner, but do not crash the scaffold process.

### Built-in Hooks
- **All templates**: Run `git init` automatically if the target directory is not already a Git repository.
- **Vite & Next templates**: Copy `.env.example` to `.env` if `.env.example` is scaffolded.

---

## 5. Version-Aware Doctor Diagnostics

### Purpose
Verify that installed runtime tools satisfy minimum required versions for reliable execution of scaffolded stacks.

### Check Upgrades
- **Node.js**: Require version `>= 18.0.0`.
- **Bun**: Require version `>= 1.0.0`.
- **Python**: Require version `>= 3.10.0`.
- **Rust/Cargo**: Check version matches minimum requirements.
- **Database**: Perform SQLite integrity check (`PRAGMA integrity_check`) on the package catalog db.

---

## 6. Verification and Testing

1. **Mapping Test**: Run `onpkg map` on the `onpkg` repo itself. Validate that it lists symbols for `main.rs`, `cli.rs`, etc.
2. **Packing Test**: Run `onpkg pack --max-tokens 5000` to verify that larger files are converted to outlines and context budget truncation triggers gracefully.
3. **Ignore Test**: Add a file to `.gitignore` and `.onpkgignore`, verify it is not included in map/pack outputs.
4. **Hook Test**: Scaffold a new stack and verify a `.git` folder is initialized automatically.
