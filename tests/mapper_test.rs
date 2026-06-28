use std::fs;
use tempfile::tempdir;

#[path = "../src/walker.rs"]
pub mod walker;

#[path = "../src/mapper.rs"]
pub mod mapper;

#[test]
fn test_mapper_rust_python_typescript() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    // 1. Rust file setup
    let rust_code = r#"
struct MyStruct {
    a: i32,
}

enum MyEnum {
    VarA,
}

fn hello_world() {}

trait MyTrait {}
"#;
    fs::write(path.join("main.rs"), rust_code).unwrap();

    // 2. Python file setup
    let python_code = r#"
class CoolClass:
    pass

def cool_func():
    return 42
"#;
    fs::write(path.join("app.py"), python_code).unwrap();

    // 3. TS file setup
    let ts_code = r#"
class Greeter {
    greet() {}
}

function speak() {}

interface Animal {}

const bark = () => {
    console.log("woof");
};
"#;
    fs::write(path.join("index.ts"), ts_code).unwrap();

    // Run mapper
    let outlines = mapper::map_project(path).unwrap();
    assert_eq!(outlines.len(), 3);

    // Verify Rust outline
    let rust_outline = outlines.iter().find(|o| o.file == "main.rs").unwrap();
    assert_eq!(rust_outline.language, "rust");
    let rust_symbols: Vec<(&str, &str)> = rust_outline.symbols.iter().map(|s| (s.name.as_str(), s.kind.as_str())).collect();
    assert!(rust_symbols.contains(&("MyStruct", "struct_item")));
    assert!(rust_symbols.contains(&("MyEnum", "enum_item")));
    assert!(rust_symbols.contains(&("hello_world", "function_item")));
    assert!(rust_symbols.contains(&("MyTrait", "trait_item")));

    // Verify Python outline
    let py_outline = outlines.iter().find(|o| o.file == "app.py").unwrap();
    assert_eq!(py_outline.language, "python");
    let py_symbols: Vec<(&str, &str)> = py_outline.symbols.iter().map(|s| (s.name.as_str(), s.kind.as_str())).collect();
    assert!(py_symbols.contains(&("CoolClass", "class_definition")));
    assert!(py_symbols.contains(&("cool_func", "function_definition")));

    // Verify TypeScript outline
    let ts_outline = outlines.iter().find(|o| o.file == "index.ts").unwrap();
    assert_eq!(ts_outline.language, "typescript");
    let ts_symbols: Vec<(&str, &str)> = ts_outline.symbols.iter().map(|s| (s.name.as_str(), s.kind.as_str())).collect();
    assert!(ts_symbols.contains(&("Greeter", "class_declaration")));
    assert!(ts_symbols.contains(&("speak", "function_declaration")));
    assert!(ts_symbols.contains(&("Animal", "interface_declaration")));
    assert!(ts_symbols.contains(&("bark", "lexical_declaration")));

    // Verify formats
    let md = mapper::format_markdown(&outlines);
    assert!(md.contains("main.rs"));
    assert!(md.contains("app.py"));
    assert!(md.contains("index.ts"));

    let json = mapper::format_json(&outlines).unwrap();
    assert!(json.contains("main.rs"));
    assert!(json.contains("app.py"));
    assert!(json.contains("index.ts"));
}
