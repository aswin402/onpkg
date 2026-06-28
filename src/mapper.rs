use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Serialize, Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct FileOutline {
    pub file: String,
    pub language: String,
    pub symbols: Vec<Symbol>,
}

pub fn map_project(dir: &Path) -> Result<Vec<FileOutline>> {
    let files = crate::walker::get_project_walker(dir)?;
    let mut outlines = Vec::new();
    
    for f in files {
        let rel_path = f.strip_prefix(dir).unwrap_or(&f).to_string_lossy().to_string();
        if let Some(ext) = f.extension().and_then(|s| s.to_str()) {
            let outline_res = match ext {
                "rs" => parse_file(&f, &rel_path, "rust", tree_sitter_rust::LANGUAGE.into(), "(struct_item name: (type_identifier) @name) @kind (enum_item name: (type_identifier) @name) @kind (function_item name: (identifier) @name) @kind (impl_item) @kind (trait_item name: (type_identifier) @name) @kind"),
                "py" => parse_file(&f, &rel_path, "python", tree_sitter_python::LANGUAGE.into(), "(class_definition name: (identifier) @name) @kind (function_definition name: (identifier) @name) @kind"),
                "ts" | "tsx" => parse_file(&f, &rel_path, "typescript", tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), "(class_declaration name: (type_identifier) @name) @kind (function_declaration name: (identifier) @name) @kind (interface_declaration name: (type_identifier) @name) @kind (lexical_declaration (variable_declarator name: (identifier) @name value: (arrow_function))) @kind"),
                "js" | "jsx" => parse_file(&f, &rel_path, "javascript", tree_sitter_javascript::LANGUAGE.into(), "(class_declaration name: (identifier) @name) @kind (function_declaration name: (identifier) @name) @kind"),
                _ => continue,
            };
            match outline_res {
                Ok(outline) => {
                    if !outline.symbols.is_empty() {
                        outlines.push(outline);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse symbols in {}: {}", rel_path, e);
                }
            }
        }
    }
    Ok(outlines)
}

fn parse_file(
    path: &Path,
    rel_path: &str,
    lang_name: &str,
    lang: tree_sitter::Language,
    query_str: &str,
) -> Result<FileOutline> {
    let source_code = std::fs::read_to_string(path)?;
    let mut parser = Parser::new();
    parser.set_language(&lang)?;
    
    let tree = parser.parse(&source_code, None)
        .ok_or_else(|| anyhow!("Failed to parse {}", rel_path))?;
        
    let query = Query::new(&lang, query_str)?;
    let mut cursor = QueryCursor::new();
    let mut symbols = Vec::new();
    
    let mut matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut kind = "symbol".to_string();
        let mut line = 0;
        
        for capture in m.captures {
            let node = capture.node;
            line = node.start_position().row + 1;
            let text = node.utf8_text(source_code.as_bytes()).unwrap_or("").to_string();
            
            if capture.index == 0 {
                // Name capture
                name = text;
            } else {
                // Kind capture
                kind = node.kind().to_string();
            }
        }
        if !name.is_empty() {
            symbols.push(Symbol { name, kind, line });
        }
    }
    
    Ok(FileOutline {
        file: rel_path.to_string(),
        language: lang_name.to_string(),
        symbols,
    })
}

pub fn format_markdown(outlines: &[FileOutline]) -> String {
    let mut md = String::new();
    for o in outlines {
        md.push_str(&format!("- **{}** ({})\n", o.file, o.language));
        for s in &o.symbols {
            md.push_str(&format!("  - [{}] {} (line {})\n", s.kind, s.name, s.line));
        }
    }
    md
}

pub fn format_json(outlines: &[FileOutline]) -> Result<String> {
    Ok(serde_json::to_string_pretty(outlines)?)
}
