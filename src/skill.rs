use crate::config::Config;
use crate::db::{Database, Skill};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct SkillManager {
    config: Config,
    db: Database,
}

impl SkillManager {
    pub fn new(config: Config, db: Database) -> Self {
        Self { config, db }
    }

    pub fn list(&self) -> Result<Vec<Skill>> {
        self.db.list_skills()
    }

    pub fn show(&self, name: &str) -> Result<Option<Skill>> {
        self.db.get_skill(name)
    }

    pub fn install(&self, name: &str) -> Result<()> {
        // Check if already installed
        if let Some(skill) = self.db.get_skill(name)? {
            return Err(anyhow!(
                "Skill '{}' is already installed (v{}). Use 'onpkg skill remove {}' first.",
                name, skill.version, name
            ));
        }

        // Check if it's a local file
        let path = Path::new(name);
        if path.exists() && path.is_file() {
            return self.install_from_path(name, path);
        }

        // Check ~/.onpkg/skills/ directory
        let local_path = self.config.skills_dir().join(format!("{}.md", name));
        if local_path.exists() {
            return self.install_from_path(name, &local_path);
        }

        // Search common skill directories
        if let Some(path) = find_in_common_dirs(name) {
            return self.install_from_path(name, &path);
        }

        // Check built-in skills
        if let Some(content) = BuiltinSkills::get(name) {
            let dest = self.config.skills_dir().join(format!("{}.md", name));
            fs::create_dir_all(&self.config.skills_dir())?;
            fs::write(&dest, &content)?;

            let desc = parse_skill_description(&content);

            let skill = Skill {
                name: name.to_string(),
                description: desc,
                version: "1.0.0".to_string(),
                source: "builtin".to_string(),
                path: dest.to_string_lossy().to_string(),
            };
            self.db.insert_skill(&skill)?;
            return Ok(());
        }

        Err(anyhow!(
            "Skill '{}' not found. Try 'onpkg skill add {} <path>' or 'onpkg registry search --type skill {}'",
            name, name, name
        ))
    }

    pub fn install_from_path(&self, name: &str, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("Path {:?} does not exist", path));
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;

        let dest = self.config.skills_dir().join(format!("{}.md", name));
        fs::create_dir_all(&self.config.skills_dir())?;
        fs::write(&dest, &content)?;

        // Extract description from frontmatter or first heading
        let description = parse_skill_description(&content);

        let skill = Skill {
            name: name.to_string(),
            description,
            version: "1.0.0".to_string(),
            source: path.to_string_lossy().to_string(),
            path: dest.to_string_lossy().to_string(),
        };
        self.db.insert_skill(&skill)?;
        Ok(())
    }

    pub fn add_from_path(&self, name: &str, source: &str) -> Result<()> {
        let source_path = Path::new(source);
        if source_path.exists() {
            return self.install_from_path(name, source_path);
        }

        // Check if source is a skill directory containing SKILL.md
        let dir_skill = source_path.join("SKILL.md");
        if source_path.is_dir() && dir_skill.exists() {
            return self.install_from_path(name, &dir_skill);
        }

        // Assume it's a URL - download it
        Err(anyhow!("Remote skill installation from URL not yet implemented"))
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let path = self.config.skills_dir().join(format!("{}.md", name));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        self.db.delete_skill(name)?;
        Ok(())
    }

    pub fn skill_path(&self, name: &str) -> Option<String> {
        self.db.get_skill(name).ok()?.map(|s| s.path)
    }
}

/// Built-in skills that ship with onpkg
pub struct BuiltinSkills;

impl BuiltinSkills {
    pub fn get(name: &str) -> Option<String> {
        match name {
            "onpkg" => Some(include_str!("../SKILL.md").to_string()),
            "frontend-design" => Some(include_str!("../builtin-skills/frontend-design/SKILL.md").to_string()),
            "ui-ux-pro-max" => Some(include_str!("../builtin-skills/ui-ux-pro-max/SKILL.md").to_string()),
            "react" => Some(include_str!("../builtin-skills/react.md").to_string()),
            "tailwind" | "tailwindcss" => Some(include_str!("../builtin-skills/tailwind.md").to_string()),
            "next" | "nextjs" => Some(include_str!("../builtin-skills/next.md").to_string()),
            "hono" => Some(include_str!("../builtin-skills/hono.md").to_string()),
            "fastapi" => Some(include_str!("../builtin-skills/fastapi.md").to_string()),
            "prisma" => Some(include_str!("../builtin-skills/prisma.md").to_string()),
            "express" => Some(include_str!("../builtin-skills/express.md").to_string()),
            "flutter" => Some(include_str!("../builtin-skills/flutter.md").to_string()),
            "mongodb" | "mongoose" => Some(include_str!("../builtin-skills/mongodb.md").to_string()),
            "postgres" | "postgresql" => Some(include_str!("../builtin-skills/postgres.md").to_string()),
            "rust" => Some(include_str!("../builtin-skills/rust.md").to_string()),
            "vite" => Some(include_str!("../builtin-skills/vite.md").to_string()),
            _ => None,
        }
    }
}

/// Parse skill description from frontmatter or first heading
fn parse_skill_description(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Try YAML frontmatter between --- markers
    if lines.len() > 2 && lines[0].trim() == "---" {
        for i in 1..lines.len() {
            if lines[i].trim() == "---" {
                break;
            }
            if let Some(rest) = lines[i].strip_prefix("description:") {
                let desc = rest.trim().trim_matches('"').to_string();
                if !desc.is_empty() {
                    return desc;
                }
            }
        }
    }

    // Fallback to first # heading not inside a code block
    let mut in_code_block = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block && trimmed.starts_with("# ") {
            return trimmed.trim_start_matches("# ").to_string();
        }
    }

    "skill".to_string()
}

/// Find a skill in common agent skill directories
pub fn find_in_common_dirs(name: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let candidates = vec![
        // Crush/Claude skills
        PathBuf::from(&home).join(".claude/skills").join(name).join("SKILL.md"),
        PathBuf::from(&home).join(".claude/skills").join(name),
        // BMAD agent skills
        PathBuf::from(&home).join(".agents/skills").join(name).join("SKILL.md"),
        PathBuf::from(&home).join(".agents/skills").join(name),
        PathBuf::from(&home).join(".agents/skills/ui-ux-pro-max/.claude/skills").join(name).join("SKILL.md"),
    ];

    for p in candidates {
        if p.is_file() {
            return Some(p);
        }
        // Check if name.md exists in dir
        let md = p.join(format!("{}.md", name));
        if md.is_file() {
            return Some(md);
        }
    }
    None
}
