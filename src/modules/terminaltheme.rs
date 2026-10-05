use crate::module::{Module, ModuleOutput};

pub struct TerminalTheme;

impl Module for TerminalTheme {
    fn name(&self) -> &'static str {
        "TerminalTheme"
    }

    fn id(&self) -> &'static str {
        "terminaltheme"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let theme = detect_terminal_theme();
        Some(ModuleOutput::new("TerminalTheme", theme))
    }
}

fn detect_terminal_theme() -> String {
    if let Ok(colorfgbg) = std::env::var("COLORFGBG")
        && let Some((_, bg)) = colorfgbg.split_once(';')
        && let Ok(bg_num) = bg.trim().parse::<u32>()
    {
        if bg_num == 0 || (8..=14).contains(&bg_num) || bg_num < 7 {
            return "Dark".to_string();
        } else {
            return "Light".to_string();
        }
    }

    // Default to Dark on modern developer Linux terminals
    "Dark".to_string()
}
