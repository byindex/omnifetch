pub mod cache;
pub mod cmd;
pub mod completion;
pub mod config;
pub mod image;
pub mod json;
pub mod logo;
pub mod module;
pub mod modules;
pub mod presets;
pub mod render;
pub mod style;
pub mod sys;
pub mod template;
pub mod toml;
pub mod tui_gen_config;
pub mod utsname;

pub use module::{BoxedModule, Field, Module, ModuleOutput};
pub use modules::{default_modules, fast_modules};

pub struct ProgressBar;

impl ProgressBar {
    pub fn render_colored(
        used: u64,
        total: u64,
        width: usize,
        fill: &str,
        empty: &str,
        color: bool,
    ) -> String {
        if total == 0 {
            return String::new();
        }
        let width = width.clamp(1, 500);
        let filled = if used > 0 {
            (((used as f64 / total as f64) * width as f64).round() as usize)
                .max(1)
                .min(width)
        } else {
            0
        };
        let empty_count = width.saturating_sub(filled);
        let pct = (used as f64 / total as f64) * 100.0;

        if !color {
            return format!("[{}{}]", fill.repeat(filled), empty.repeat(empty_count));
        }

        let fill_color = if pct < 60.0 {
            "\x1b[32m"
        } else if pct < 85.0 {
            "\x1b[33m"
        } else {
            "\x1b[31m"
        };
        let empty_color = "\x1b[90m";
        let reset = "\x1b[0m";

        format!(
            "[{fill_color}{}{empty_color}{}{reset}]",
            fill.repeat(filled),
            empty.repeat(empty_count)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_zero_total() {
        assert_eq!(ProgressBar::render_colored(0, 0, 10, "=", "-", false), "");
    }

    #[test]
    fn test_progress_bar_basic_uncolored() {
        let bar = ProgressBar::render_colored(50, 100, 10, "#", ".", false);
        assert_eq!(bar, "[#####.....]");

        let full = ProgressBar::render_colored(100, 100, 10, "#", ".", false);
        assert_eq!(full, "[##########]");

        let empty = ProgressBar::render_colored(0, 100, 10, "#", ".", false);
        assert_eq!(empty, "[..........]");
    }

    #[test]
    fn test_progress_bar_clamping_and_coloring() {
        let bar = ProgressBar::render_colored(50, 100, 1000, "#", ".", false);
        // clamped to 500 max width
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert_eq!(bar.chars().count(), 502);

        let colored = ProgressBar::render_colored(90, 100, 10, "#", ".", true);
        assert!(colored.contains("\x1b[31m")); // Red color for >= 85%
    }
}
