use std::fs;
use std::path::PathBuf;

use crate::module::{Module, ModuleOutput};

pub struct Theme;

impl Module for Theme {
    fn name(&self) -> &'static str {
        "Theme"
    }
    fn id(&self) -> &'static str {
        "theme"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let t = detect_theme()?;
        Some(ModuleOutput::new("Theme", t))
    }
}

fn home_config(rel: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let p = PathBuf::from(home).join(".config").join(rel);
    p.exists().then_some(p)
}

fn detect_theme() -> Option<String> {
    for rel in ["gtk-3.0/settings.ini", "gtk-4.0/settings.ini"] {
        if let Some(path) = home_config(rel)
            && let Ok(content) = fs::read_to_string(path)
        {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=')
                    && k.trim() == "gtk-theme-name"
                {
                    let val = v.trim().trim_matches('"').to_string();
                    if !val.is_empty() {
                        return Some(val);
                    }
                }
            }
        }
    }
    None
}
