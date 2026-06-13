# Core Features 🌟

This document details the main features of **onpkg**.

## 1. Stack Scaffolding (`onpkg stack add`)
Scaffolds premium, fully-configured architectures for modern stacks (like React, Next.js, FastAPI, Flutter, etc.). Unlike basic CLI tools that output a single placeholder file, `onpkg` sets up complete folder structures (routes, stores, components, services) and config files (`tsconfig.json`, `eslint.config.js`, `docker-compose.yml`, etc.).

## 2. Dynamic Online Installation
When a stack is scaffolded:
1. `onpkg` detects the presence of manifest files (`package.json`, `pyproject.toml`, etc.).
2. It automatically determines and executes the best package installer (`bun`, `npm`, `pnpm`, `uv`, `pip`, etc.) to fetch the latest dependencies online.
3. This guarantees the project is immediately ready to run (`npm run dev`) upon scaffolding.

## 3. AI Agent Skills (`onpkg_docs/`)
For every technology associated with a stack, `onpkg` extracts/copies its agent skill instructions to `onpkg_docs/` in the project root:
- `onpkg_docs/react.md` & `onpkg_docs/react/skill.md`
- `onpkg_docs/tailwind.md` & `onpkg_docs/tailwind/skill.md`
- `onpkg_docs/INDEX.md` (links all skills)

AI coding agents (like Claude/Gemini) can instantly read these documents to match best practices, naming conventions, directory structure, and preferred packages.

## 4. AI-Driven Extension (`onpkg ai`)
Integrates the Gemini API to let users dynamically expand the capability of `onpkg`:
- **`onpkg ai skill <name>`**: Generates a custom technology skill Markdown file and registers it in the local database.
- **`onpkg ai template <name> --description "<desc>"`**: Generates a custom TOML template definition with files, structure, and configurations based on a description.

## 5. Doctor Diagnostics (`onpkg doctor`)
Checks the health of the local setup:
- SQLite Database integrity check.
- Configuration file availability.
- Available stacks and installed skills count.
- PATH verification for Node.js, Bun, Python 3, and Cargo with their active versions.

## 6. Spec-Driven AI Workflows & Project Syncing (`onpkg sync`)
Maintains alignment between the repository codebase, dependencies, and AI assistant specifications:
- **`onpkg sync`**: Scans the project recursively to build a complete `sources` index (directories, files, and extension patterns) and update the project architecture layout (`entrypoint`, `routing`, `components`, `styles`, `database`, `tests`) in `onpkg.json`.
- **AI Agent Specifications**: Scaffolds living requirements, design system templates, engineering plans, page layouts, and checklists (`prd.md`, `content.md`, `design.md`, `implementation.md`, `todo.md`) under `onpkg_docs/` to organize requirements and facilitate SDD (Spec-Driven Development) across agents like `openz`.
