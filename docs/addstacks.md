# Adding Custom & Built-in Stacks 🏗️

This guide outlines how to define new stacks and templates in `onpkg`.

## 1. Adding a Custom Stack (TOML Definition)

Custom stacks can be created easily by writing a TOML configuration and placing it in `~/.onpkg/templates/`.

### Creating a Stack TOML
Run:
```bash
onpkg stack new my-new-stack
```
This generates a file at `~/.onpkg/templates/my-new-stack.toml`. Open it and define your files, variables, and technologies:

```toml
name = "my-new-stack"
category = "frontend"
description = "My custom frontend stack"
version = "1.0.0"
technologies = ["react", "tailwind"]

[[variables]]
name = "project_name"
description = "Name of the project"
default = "my-app"

[[files]]
path = "package.json"
content = """{
  "name": "{{ project_name }}",
  "version": "1.0.0",
  "dependencies": {
    "react": "^19.0.0"
  }
}"""

[[files]]
path = "src/index.js"
content = "console.log('App scaffolded!');"
```

Running `onpkg stack list` will now display `my-new-stack` under custom stacks, and `onpkg stack add my-new-stack` will scaffold it!

---

## 2. Modifying or Adding a Built-in Stack

Built-in stacks are compiled directly into the binary from `src/templates/builtin/`:

1. Copy/write your stack submodule in [src/templates/builtin/](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/templates/builtin/) (e.g. `my_stack.rs` returning a `crate::stacks::Stack` struct).
2. Register the module inside [src/templates/builtin/stacks_mod.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/templates/builtin/stacks_mod.rs):
   ```rust
   pub mod my_stack;
   ```
3. Append your stack initializer to `builtin_stacks()` inside [src/templates/builtin/stacks_mod.rs](file:///home/aswin/programming/vscode/myProjects/ai_agent_tools/onpkg/src/templates/builtin/stacks_mod.rs):
   ```rust
   pub fn builtin_stacks() -> Vec<Stack> {
       vec![
           ...
           my_stack::my_stack(),
       ]
   }
   ```
4. Rebuild/reinstall `onpkg` using `./localupdate.sh`.
