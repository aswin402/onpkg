# Modifying the Codebase 🛠️

This guide outlines how to modify the `onpkg` code, test changes, and re-compile.

## Development Workflow

1. Make edits to the source files in `src/`.
2. Run `cargo check` to verify there are no compilation errors or lints.
3. Test your changes by compiling in debug mode:
   ```bash
   cargo build
   # Run the debug binary
   ../../target/debug/onpkg stack list
   ```
4. Reinstall the CLI globally to apply changes:
   ```bash
   ./localupdate.sh
   ```

## Adding Dependencies

If you need to add new crates/packages to `onpkg`, edit `Cargo.toml`:

```toml
[dependencies]
my_new_crate = "1.0"
```

Then run `cargo check` or `cargo build` to fetch and compile the new dependency.

## Modifying CLI Commands

To add or modify subcommands, edit [src/cli.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/cli.rs):

1. Add the enum variant under `Command` or the specific `Subcommand` enums.
2. Edit [src/main.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/main.rs) inside the `match command` statement to handle the new command logic.

## Modifying AI Prompts

To tune the AI skill or template generation behavior, modify the prompts in [src/ai.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/ai.rs):
- `generate_skill` handles tech-skill instruction prompt tuning.
- `generate_template` handles TOML stack structure generation prompt tuning.
