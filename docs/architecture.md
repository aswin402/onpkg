# Architecture Documentation 🏗️

This document describes the high-level architecture, module flow, and schema layout of **onpkg**.

## System Diagram

```
                 onpkg CLI (clap)
                         │
                         ▼
        main.rs ───────────────── Config (TOML)
           │                         └── ~/.onpkg/config.toml
           ├─► Database (SQLite)
           │    └── ~/.onpkg/cache/onpkg.db
           │
           ├─► TemplateEngine (Jinja2)
           │    ├─► Built-in Stacks (templates/builtin/)
           │    └─► Custom Stacks (~/.onpkg/templates/)
           │
           ├─► PkgRegistry (fetch npm/pypi/pub/cargo APIs)
           │
           ├─► SkillManager (Built-in + ~/.onpkg/skills/)
           │
           └─► AiGenerator (Gemini API integrations)
```

## Core Modules

| Module | Location | Purpose |
|---|---|---|
| `main` | `src/main.rs` | CLI entrypoint, matches commands, coordinates scaffolding and installations. |
| `cli` | `src/cli.rs` | Command line argument parsing using `clap` (handles `stack`, `ai`, `skill`, etc.). |
| `config` | `src/config.rs` | Reads, validates, and writes global settings in `~/.onpkg/config.toml`. |
| `db` | `src/db.rs` | SQLite database manager, saves catalog records for packages, skills, and templates. |
| `templates` | `src/templates/mod.rs` | Scaffolds files, runs online package installers, and populates project `onpkg_docs/`. |
| `stacks` | `src/stacks.rs` | Loads and matches premium templates and submodules under `src/templates/builtin/`. |
| `ai` | `src/ai.rs` | Handles communication with Gemini API (`gemini-2.5-flash`) for AI generation. |
| `skill` | `src/skill.rs` | Manages AI agent instructions (skills) in `~/.onpkg/skills/`. |
| `pkg_registry` | `src/pkg_registry.rs` | Resolves packages from online registries and appends them to project manifests. |
| `tui` | `src/tui.rs` | UI logs, colored symbols, animated loading spinners. |

## Data Schemas

`onpkg` maintains a database at `~/.onpkg/cache/onpkg.db` to track installed resources:

```sql
-- Packages Cache Table
CREATE TABLE IF NOT EXISTS packages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    runtime     TEXT NOT NULL DEFAULT 'npm',
    type        TEXT NOT NULL DEFAULT 'pkg',
    source      TEXT NOT NULL DEFAULT '',
    cache_path  TEXT NOT NULL,
    checksum    TEXT NOT NULL,
    size_bytes  INTEGER,
    cached_at   DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, version, runtime, type)
);

-- Custom/Cached Templates Table
CREATE TABLE IF NOT EXISTS templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    category    TEXT NOT NULL DEFAULT 'general',
    description TEXT NOT NULL DEFAULT '',
    version     TEXT NOT NULL DEFAULT '1.0.0',
    source      TEXT NOT NULL DEFAULT '',
    files_count INTEGER DEFAULT 0,
    installed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- AI Agent Skills Table
CREATE TABLE IF NOT EXISTS skills (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    version     TEXT NOT NULL DEFAULT '1.0.0',
    source      TEXT NOT NULL DEFAULT '',
    path        TEXT NOT NULL,
    installed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```
