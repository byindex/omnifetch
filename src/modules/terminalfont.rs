use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::PathBuf;

pub struct TerminalFont;

impl Module for TerminalFont {
    fn name(&self) -> &'static str {
        "Terminal Font"
    }
    fn id(&self) -> &'static str {
        "terminalfont"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let font = detect_terminal_font()?;
        Some(ModuleOutput::new("Terminal Font", font))
    }
}

fn detect_terminal_font() -> Option<String> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;

    // Rio: ~/.config/rio/config.toml
    let rio = home.join(".config/rio/config.toml");
    if let Ok(text) = fs::read_to_string(rio) {
        let mut family = None;
        let mut size = None;
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "family" => family = Some(v.trim().trim_matches('"').to_string()),
                    "size" => size = Some(v.trim().to_string()),
                    _ => {}
                }
            }
        }
        match (family, size) {
            (Some(f), Some(s)) => return Some(format!("{f} {s}")),
            (Some(f), None) => return Some(f),
            (None, Some(s)) => return Some(format!("Default {s}")),
            _ => {}
        }
    }

    // Kitty: ~/.config/kitty/kitty.conf
    let kitty = home.join(".config/kitty/kitty.conf");
    if let Ok(text) = fs::read_to_string(kitty) {
        let mut family = None;
        let mut size = None;
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("font_family") => family = Some(parts.collect::<Vec<_>>().join(" ")),
                Some("font_size") => size = parts.next().map(str::to_string),
                _ => {}
            }
        }
        if let Some(f) = family {
            return Some(format!(
                "{f}{}",
                size.map(|s| format!(" {s}")).unwrap_or_default()
            ));
        }
    }

    // Alacritty: ~/.config/alacritty/alacritty.toml
    let alacritty = home.join(".config/alacritty/alacritty.toml");
    if let Ok(text) = fs::read_to_string(alacritty) {
        for line in text.lines() {
            if line.contains("family =")
                && let Some((_, v)) = line.split_once('=')
            {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }

    None
}
