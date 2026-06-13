# onpkg — Online Package & Template Manager for AI Agents

onpkg is a CLI tool that scaffolds full projects from prebuilt templates, manages AI agent skills, and caches packages. It lets agents (and humans) go from idea to working project in one command.

## Quickstart

```bash
# Install
cargo install --path /path/to/onpkg

# See what's available
onpkg doctor
onpkg template list

# Scaffold a project
onpkg template use react-vite --dir my-app
onpkg template use next-app --dir my-site --var project_name=my-blog

# Install an agent skill
onpkg skill install onpkg
```

## Commands

### `onpkg init [name] [--template <name>]`

Initialize a new onpkg project. Creates `onpkg.json` in the target directory. If `--template` is provided, also scaffolds that template.

```bash
onpkg init my-project --template react-vite-tailwind
```

### `onpkg template list [--category <cat>]`

List all available templates. Filter by category: `website`, `frontend`, `backend`, `fullstack`, `app`.

Built-in templates:

| Template | Category | Description |
|----------|----------|-------------|
| `react-vite` | frontend | React 19 + Vite + TypeScript + ESLint |
| `react-vite-tailwind` | frontend | React 19 + Vite + Tailwind CSS v4 |
| `next-app` | frontend | Next.js 15 App Router + TypeScript |
| `next-app-full` | frontend | Next.js 15 + Tailwind + Prisma + Auth.js |
| `hono-api` | backend | Hono + TypeScript + Zod validation |
| `hono-full` | backend | Hono + Prisma + PostgreSQL + JWT |
| `express-api` | backend | Express.js + TypeScript + middleware |
| `fastapi` | backend | FastAPI + SQLAlchemy + Alembic |
| `fastapi-full` | backend | FastAPI + Auth + Celery + Docker |
| `mern` | fullstack | MongoDB + Express + React + Node.js |
| `pern` | fullstack | PostgreSQL + Express + React + Node.js |
| `flutter-app` | app | Flutter + Riverpod + GoRouter |
| `static-website` | website | HTML5 + CSS3 + JS static site |
| `rust-cli` | app | Minimal Rust CLI with clap |

### `onpkg template show <name>`

Show detailed info about a template including file list and variables.

```bash
onpkg template show react-vite-tailwind
```

### `onpkg template use <name> [--dir <path>] [--var key=value]`

Scaffold a template into a directory (defaults to current dir). Supports variable substitution with `--var`.

```bash
onpkg template use react-vite --dir my-app --var project_name=my-app --var author=bot
```

### `onpkg template add <name> <source>`

Add a custom template from a local directory or git URL.

```bash
onpkg template add my-template ./path/to/template
onpkg template add my-template https://github.com/user/template.git
```

### `onpkg template remove <name>`

Remove a custom template (built-in templates cannot be removed).

### `onpkg skill list`

List installed skills.

### `onpkg skill install <name>`

Install a skill. Checks: (1) local file path, (2) `~/.onpkg/skills/<name>.md`, (3) built-in skills.

```bash
onpkg skill install onpkg       # install built-in onpkg skill
onpkg skill install ./SKILL.md  # install from local path
```

### `onpkg skill add <name> <source>`

Add a custom skill from a local file.

```bash
onpkg skill add my-skill ./path/to/SKILL.md
```

### `onpkg skill remove <name>`

Remove an installed skill.

### `onpkg pkg list [--runtime <rt>]`

List cached packages.

### `onpkg registry search <query> [--type template|skill|pkg]`

Search the online registry for templates, skills, and packages.

```bash
onpkg registry search react --type template
onpkg registry search auth --type skill
```

### `onpkg registry info <name>`

Get detailed info about a registry item.

### `onpkg registry config --url <url>`

Set a custom registry URL (default: `https://registry.onpkg.dev`).

### `onpkg doctor`

Run diagnostics: check config, database, templates, skills, registry, and local runtimes.

### `onpkg update`

Check for updates (not yet implemented).

## Agent Workflow

As an AI agent, use onpkg to scaffold projects, install skills, and manage templates:

1. **Check the environment**: Run `onpkg doctor` to verify installation and available runtimes.
2. **List available templates**: Run `onpkg template list` to see what's available.
3. **Scaffold a project**: Use `onpkg template use <name>` with appropriate `--var` values.
4. **Install skills**: Run `onpkg skill install <name>` to add agent skills.
5. **Add templates**: Run `onpkg template add <name> <path-or-url>` to register new templates.
6. **Publish to registry**: Run `onpkg template publish <name>` to share templates with other agents.

## Configuration

Config lives at `~/.onpkg/config.toml`:

```toml
[registry]
url = "https://registry.onpkg.dev"

[cache]
path = "~/.onpkg/cache"
max_size_gb = 10.0

[network]
timeout_secs = 30
retries = 3
```

Environment variables:
- `ONPKG_CACHE_DIR` — override cache path
- `ONPKG_REGISTRY_URL` — override registry URL

## Adding Templates as an Agent

Agents can create and add templates programmatically:

1. **From an existing project**: `onpkg template add my-template ./existing-project`
2. **From a git repo**: `onpkg template add my-template https://github.com/user/repo.git`
3. **By creating a TOML definition**: Write a `.toml` file in `~/.onpkg/templates/`
4. **Programmatically**: Write files to a directory and use `onpkg template add`

### Template TOML format

```toml
name = "my-template"
category = "frontend"
description = "My custom template"
version = "1.0.0"

[[variables]]
name = "project_name"
description = "Project name"
default = "my-app"

[[files]]
path = "package.json"
content = """
{
  "name": "{{ project_name }}",
  "version": "0.1.0"
}
"""
```

## Publishing Skills as an Agent

1. Write a `SKILL.md` file
2. Add it to onpkg: `onpkg skill add my-skill ./SKILL.md`
3. Publish: `onpkg skill publish my-skill`

## File System Layout

```
~/.onpkg/
├── config.toml          # Global configuration
├── templates/           # Custom template definitions
│   └── my-template.toml
├── skills/              # Installed skills
│   └── my-skill.md
└── cache/               # Package cache
    ├── onpkg.db         # SQLite database
    ├── npm/
    ├── pypi/
    └── pub/
```

## Troubleshooting

- `onpkg doctor` — run diagnostics first
- `onpkg registry search <query>` — search the online registry
- `onpkg registry info <name>` — get details about a registry item
- Registry offline? Set a custom one: `onpkg registry config --url <your-registry>`
- Template not found? Add it: `onpkg template add <name> <source>`
