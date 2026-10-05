use crate::module::{Module, ModuleOutput};

pub struct Terminal;

impl Module for Terminal {
    fn name(&self) -> &'static str {
        "Terminal"
    }
    fn id(&self) -> &'static str {
        "terminal"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if let (Ok(name), Ok(ver)) = (
            std::env::var("TERM_PROGRAM"),
            std::env::var("TERM_PROGRAM_VERSION"),
        ) {
            return Some(ModuleOutput::new("Terminal", format!("{name} {ver}")));
        }
        if std::env::var("WEZTERM_EXECUTABLE").is_ok() {
            return Some(ModuleOutput::new("Terminal", "WezTerm"));
        }
        if std::env::var_os("KITTY_WINDOW_ID").is_some() {
            let ver = std::env::var("TERM_PROGRAM_VERSION").unwrap_or_default();
            let label = if ver.is_empty() {
                "kitty".to_string()
            } else {
                format!("kitty {ver}")
            };
            return Some(ModuleOutput::new("Terminal", label));
        }
        if std::env::var("ALACRITTY_LOG").is_ok() || std::env::var("ALACRITTY_CONFIG").is_ok() {
            return Some(ModuleOutput::new("Terminal", "Alacritty"));
        }
        if std::env::var("VTE_VERSION").is_ok() {
            return Some(ModuleOutput::new("Terminal", "GNOME Terminal"));
        }
        if std::env::var("KONSOLE_VERSION").is_ok() {
            return Some(ModuleOutput::new("Terminal", "Konsole"));
        }
        if let Ok(t) = std::env::var("TERM")
            && !t.is_empty()
        {
            return Some(ModuleOutput::new("Terminal", t));
        }
        None
    }
}
