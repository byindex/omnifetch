use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::PathBuf;

pub struct WmTheme;

impl Module for WmTheme {
    fn name(&self) -> &'static str {
        "WM Theme"
    }
    fn id(&self) -> &'static str {
        "wmtheme"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let theme = detect_wm_theme()?;
        Some(ModuleOutput::new("WM Theme", theme))
    }
}

fn detect_wm_theme() -> Option<String> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;

    // KDE KWin: ~/.config/kwinrc [org.kde.kdecoration2] theme=...
    let kwinrc = home.join(".config/kwinrc");
    if let Ok(text) = fs::read_to_string(kwinrc) {
        let mut in_decor = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_decor = trimmed.contains("kdecoration");
            } else if in_decor
                && let Some((k, v)) = trimmed.split_once('=')
                && k.trim() == "theme"
            {
                let val = v.trim();
                let clean = val.strip_prefix("__aurorae__svg__").unwrap_or(val);
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }

    // XFWM4: ~/.config/xfce4/xfconf/xfce-perchannel-xml/xfwm4.xml
    let xfwm = home.join(".config/xfce4/xfconf/xfce-perchannel-xml/xfwm4.xml");
    if let Ok(text) = fs::read_to_string(xfwm) {
        for line in text.lines() {
            if line.contains("name=\"theme\"")
                && let Some((_, val)) = line.split_once("value=\"")
                && let Some((theme, _)) = val.split_once('"')
            {
                return Some(theme.to_string());
            }
        }
    }

    None
}
