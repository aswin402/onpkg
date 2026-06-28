use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use tiktoken_rs::cl100k_base;

#[derive(Serialize, Debug)]
pub struct PackResult {
    pub content: String,
    pub token_count: usize,
    pub file_count: usize,
    pub skipped_files: Vec<String>,
}

pub fn pack_project(dir: &Path, max_tokens: usize) -> Result<PackResult> {
    let bpe = cl100k_base()?;
    let files = crate::walker::get_project_walker(dir)?;
    
    // Pre-map outlines for the entire project. This resolves the relative path bug
    // in the template code and optimizes the walk to occur only once.
    let all_outlines = crate::mapper::map_project(dir)?;
    
    let mut packed_content = String::new();
    let mut file_count = 0;
    let mut skipped_files = Vec::new();
    
    // 1. Generate Folder tree structure
    packed_content.push_str("# Project Directory Structure\n```\n");
    for f in &files {
        let rel_path = f.strip_prefix(dir).unwrap_or(f).to_string_lossy().to_string();
        packed_content.push_str(&format!("{}\n", rel_path));
    }
    packed_content.push_str("```\n\n");
    
    // 2. Pack file contents
    for f in files {
        let rel_path = f.strip_prefix(dir).unwrap_or(&f).to_string_lossy().to_string();
        let content = std::fs::read_to_string(&f)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let file_representation = if lines.len() < 200 {
            format!("## File: {}\n```\n{}\n```\n\n", rel_path, content)
        } else {
            // Outlines representation using mapper
            let matching_outline = all_outlines.iter().find(|o| o.file == rel_path);
            
            if let Some(out) = matching_outline {
                format!("## File: {} (Symbol Outline only - file is >= 200 lines)\n```\n{}\n```\n\n", 
                    rel_path, crate::mapper::format_markdown(std::slice::from_ref(out)))
            } else {
                format!("## File: {} (Skipped content - file is >= 200 lines)\n\n", rel_path)
            }
        };
        
        let tokens = bpe.encode_with_special_tokens(&file_representation);
        let current_tokens = bpe.encode_with_special_tokens(&packed_content);
        
        if current_tokens.len() + tokens.len() < max_tokens {
            packed_content.push_str(&file_representation);
            file_count += 1;
        } else {
            skipped_files.push(rel_path);
        }
    }
    
    let final_tokens = bpe.encode_with_special_tokens(&packed_content).len();
    Ok(PackResult {
        content: packed_content,
        token_count: final_tokens,
        file_count,
        skipped_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_pack_project_basic() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let file1 = path.join("small.rs");
        fs::write(&file1, "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();

        let file2 = path.join("large.rs");
        let mut large_content = String::new();
        for i in 0..250 {
            large_content.push_str(&format!("// line {}\n", i));
        }
        // Let's also add a struct/function inside large.rs so symbols are detected
        large_content.push_str("pub struct MyLargeStruct {}\n");
        fs::write(&file2, &large_content).unwrap();

        let result = pack_project(path, 10000).unwrap();
        assert_eq!(result.file_count, 2);
        assert!(result.skipped_files.is_empty());
        assert!(result.content.contains("# Project Directory Structure"));
        assert!(result.content.contains("small.rs"));
        assert!(result.content.contains("large.rs"));
        assert!(result.content.contains("fn main()"));
        // Large file should show symbol outline because it's >= 200 lines
        assert!(result.content.contains("large.rs (Symbol Outline only - file is >= 200 lines)"));
        assert!(result.content.contains("MyLargeStruct"));
    }

    #[test]
    fn test_pack_project_token_limit() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let file1 = path.join("file1.rs");
        fs::write(&file1, "fn first() {}\n").unwrap();

        let file2 = path.join("file2.rs");
        fs::write(&file2, "fn second() {}\n").unwrap();

        let result = pack_project(path, 35).unwrap();
        assert!(result.file_count < 2);
        assert!(!result.skipped_files.is_empty());
    }
}
