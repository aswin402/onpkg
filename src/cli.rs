use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "onpkg",
    version = "0.1.0",
    about = "Online Package & Template Manager — scaffold projects, manage skills, cache packages",
    long_about = "onpkg is an online package and template manager for developers and AI agents.\n\nCommands:\n  template    Scaffold projects from prebuilt templates\n  skill       Install and manage AI agent skills\n  pkg         Install and manage packages\n  registry    Interact with the online registry\n  init        Initialize a new project\n  doctor      Run diagnostics\n  update      Update onpkg itself",
    disable_version_flag = true
)]
pub struct Args {
    /// Print version information
    #[arg(short = 'V', long = "version")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new project with onpkg support
    Init {
        /// Project name
        name: Option<String>,
        /// Template to use (optional)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Template commands — scaffold projects from prebuilt templates
    Template {
        #[command(subcommand)]
        subcmd: TemplateSubcommand,
    },

    /// Skill commands — install and manage AI agent skills
    Skill {
        #[command(subcommand)]
        subcmd: SkillSubcommand,
    },

    /// Package commands — install and manage packages
    #[command(name = "pkg")]
    Pkg {
        #[command(subcommand)]
        subcmd: PkgSubcommand,
    },

    /// Registry commands — interact with the online registry
    Registry {
        #[command(subcommand)]
        subcmd: RegistrySubcommand,
    },

    /// Run diagnostics to check onpkg health
    Doctor,

    /// Update onpkg itself to the latest version
    Update,

    /// Stack commands — scaffold and install complete stacks online
    Stack {
        #[command(subcommand)]
        subcmd: StackSubcommand,
    },

    /// AI commands — generate skills or templates using AI (Gemini)
    Ai {
        #[command(subcommand)]
        subcmd: AiSubcommand,
    },
}

// ── Template ──────────────────────────────────────────────────────────────

#[derive(Subcommand, Clone, Debug)]
pub enum TemplateSubcommand {
    /// List all available templates
    List {
        /// Filter by category: website, app, frontend, backend
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Show details about a template
    Show {
        /// Template name
        name: String,
    },
    /// Scaffold a template into the current directory
    Use {
        /// Template name
        name: String,
        /// Target directory (defaults to current dir or project name)
        #[arg(short, long)]
        dir: Option<String>,
        /// Variables for template substitution (key=value)
        #[arg(short, long)]
        var: Vec<String>,
    },
    /// Add a custom template from a local path or git URL
    Add {
        /// Name for the template
        name: String,
        /// Path to template directory or git URL
        source: String,
    },
    /// Remove a custom template
    Remove {
        /// Template name
        name: String,
    },
    /// Publish a template to the registry
    Publish {
        /// Template name
        name: String,
    },
}

// ── Skill ─────────────────────────────────────────────────────────────────

#[derive(Subcommand, Clone, Debug)]
pub enum SkillSubcommand {
    /// List installed skills
    List,
    /// Install a skill from the registry or local file
    Install {
        /// Skill name or path to SKILL.md
        name: String,
    },
    /// Show info about an installed skill
    Show {
        /// Skill name
        name: String,
    },
    /// Remove an installed skill
    Remove {
        /// Skill name
        name: String,
    },
    /// Add a custom skill from a local path or URL
    Add {
        /// Name for the skill
        name: String,
        /// Path to SKILL.md or URL
        source: String,
    },
    /// Publish a skill to the registry
    Publish {
        /// Skill name
        name: String,
    },
}

// ── Package (pkg) ─────────────────────────────────────────────────────────

#[derive(Subcommand, Clone, Debug)]
pub enum PkgSubcommand {
    /// Install/cache a package from a registry (npm, PyPI, pub.dev, cargo)
    Install {
        /// Package name
        name: String,
        /// Runtime: npm, pypi, pub, cargo (default: auto-detect)
        #[arg(short, long)]
        runtime: Option<String>,
    },
    /// Add a cached package to the current project
    Add {
        /// Package name
        name: String,
        /// Runtime
        #[arg(short, long)]
        runtime: Option<String>,
    },
    /// List cached packages
    List {
        /// Filter by runtime
        #[arg(short, long)]
        runtime: Option<String>,
    },
    /// Remove a package from cache
    Remove {
        /// Package name
        name: String,
        /// Runtime
        #[arg(short, long)]
        runtime: Option<String>,
    },
}

// ── Registry ──────────────────────────────────────────────────────────────

#[derive(Subcommand, Clone, Debug)]
pub enum RegistrySubcommand {
    /// Search the registry for templates, skills, and packages
    Search {
        /// Search query
        query: String,
        /// Type: template, skill, pkg
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// Show info about a registry item
    Info {
        /// Item name
        name: String,
    },
    /// Configure registry settings
    Config {
        /// Set registry URL
        #[arg(short, long)]
        url: Option<String>,
    },
}

// ── Stack ─────────────────────────────────────────────────────────────────

#[derive(Subcommand, Clone, Debug)]
pub enum StackSubcommand {
    /// List all available stacks
    List {
        /// Filter by category: website, app, frontend, backend, fullstack
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Show details about a stack
    Show {
        /// Stack name
        name: String,
    },
    /// Scaffold a stack and install dependencies online
    Add {
        /// Stack name
        name: String,
        /// Target directory (defaults to current dir or project name)
        #[arg(short, long)]
        dir: Option<String>,
        /// Variables for stack substitution (key=value)
        #[arg(short, long)]
        var: Vec<String>,
        /// Custom package manager (e.g. bun, npm, pnpm, yarn, uv, pip)
        #[arg(short, long)]
        manager: Option<String>,
    },
    /// Scaffold a stack and install dependencies online (synonym for add)
    Use {
        /// Stack name
        name: String,
        /// Target directory (defaults to current dir or project name)
        #[arg(short, long)]
        dir: Option<String>,
        /// Variables for stack substitution (key=value)
        #[arg(short, long)]
        var: Vec<String>,
        /// Custom package manager (e.g. bun, npm, pnpm, yarn, uv, pip)
        #[arg(short, long)]
        manager: Option<String>,
    },
    /// Create a custom stack template
    New {
        /// Stack name
        name: String,
        /// Category
        #[arg(short, long, default_value = "custom")]
        category: String,
    },
}

// ── AI ────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Clone, Debug)]
pub enum AiSubcommand {
    /// Generate a new skill markdown using AI
    Skill {
        /// Technology name (e.g. react, tailwind, docker)
        name: String,
        /// Optional prompt or guidance for the skill
        prompt: Option<String>,
    },
    /// Generate a new template definition TOML using AI
    Template {
        /// Template name (e.g. rust-wasm, nextjs-sqlite)
        name: String,
        /// Description of the stack, architecture, and files needed
        #[arg(short, long)]
        description: String,
    },
}
