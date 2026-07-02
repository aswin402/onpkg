use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_MCP_MODE: AtomicBool = AtomicBool::new(false);

/// Terminal UI helpers using indicatif for progress and colored output
pub struct TUI;

impl TUI {
    pub fn new() -> Self {
        Self
    }

    pub fn set_mcp_mode(val: bool) {
        IS_MCP_MODE.store(val, Ordering::Relaxed);
    }

    fn is_mcp_mode() -> bool {
        IS_MCP_MODE.load(Ordering::Relaxed)
    }

    fn print(text: &str) {
        if Self::is_mcp_mode() {
            eprintln!("{}", text);
        } else {
            println!("{}", text);
        }
    }

    /// Create a spinner for waiting operations
    pub fn spinner(message: &str) -> indicatif::ProgressBar {
        let pb = ProgressBar::new_spinner();
        if Self::is_mcp_mode() {
            // In MCP mode, disable progress bar/spinner drawing
            pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
        }
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    /// Create a progress bar
    pub fn progress_bar(len: u64) -> ProgressBar {
        let pb = ProgressBar::new(len);
        if Self::is_mcp_mode() {
            pb.set_draw_target(indicatif::ProgressDrawTarget::stderr());
        }
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:50.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("━●─"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    /// Print a success message
    pub fn success(message: &str, detail: Option<&str>) {
        let detail_str = detail
            .map(|d| format!(" \x1b[38;2;100;116;139m{}\x1b[0m", d))
            .unwrap_or_default();
        Self::print(&format!(
            "\x1b[1m\x1b[38;2;0;229;160m✓\x1b[0m {} {}",
            message, detail_str
        ));
    }

    /// Print an info message
    pub fn info(message: &str) {
        Self::print(&format!("\x1b[38;2;96;165;250mℹ\x1b[0m {}", message));
    }

    /// Print a warning message
    pub fn warn(message: &str) {
        Self::print(&format!("\x1b[38;2;245;166;35m⚠\x1b[0m {}", message));
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("\x1b[1m\x1b[38;2;255;107;107m✗\x1b[0m {}", message);
    }

    /// Print a label with primary color
    pub fn label(text: &str, message: &str) {
        Self::print(&format!("\x1b[38;2;0;212;224m{}\x1b[0m  {}", text, message));
    }

    /// Print the onpkg logo
    pub fn logo() {
        if Self::is_mcp_mode() {
            return;
        }
        let cyan = "\x1b[38;2;0;212;224m";
        let bold = "\x1b[1m";
        let muted = "\x1b[38;2;100;116;139m";
        let reset = "\x1b[0m";
        println!();
        println!("{bold}{cyan}  ╔═╗╔╗╔╔═╗╦╔═╔═╗{reset}");
        println!("{bold}{cyan}  ║ ║║║║╠═╝╠╩╗║ ╦{reset}");
        println!("{bold}{cyan}  ╚═╝╝╚╝╩  ╩ ╩╚═╝{reset}");
        println!(
            "{muted}  onpkg v{} · online package & template manager{reset}",
            env!("CARGO_PKG_VERSION")
        );
        println!();
    }
}
