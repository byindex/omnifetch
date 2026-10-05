use std::fs;
use std::path::PathBuf;

use crate::module::{Module, ModuleOutput};

pub struct Font;

impl Module for Font {
    fn name(&self) -> &'static str {
        "Font"
    }
    fn id(&self) -> &'static str {
        "font"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let f = detect_gtk_setting("gtk-font-name").or_else(detect_terminal_font)?;
        Some(ModuleOutput::new("Font", f))
    }
}

fn home_config(rel: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let p = PathBuf::from(home).join(".config").join(rel);
    p.exists().then_some(p)
}

fn detect_gtk_setting(key: &str) -> Option<String> {
    for rel in ["gtk-3.0/settings.ini", "gtk-4.0/settings.ini"] {
        if let Some(path) = home_config(rel)
            && let Ok(content) = fs::read_to_string(path)
        {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=')
                    && k.trim() == key
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

fn detect_terminal_font() -> Option<String> {
    // Check Rio config
    if let Some(path) = home_config("rio/config.toml")
        && let Ok(content) = fs::read_to_string(path)
    {
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=')
                && k.trim() == "family"
            {
                let val = v.trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }
    }
    None
}
