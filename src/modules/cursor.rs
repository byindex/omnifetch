use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::template;
use std::fs;
use std::path::PathBuf;

pub struct Cursor;

impl Module for Cursor {
    fn name(&self) -> &'static str {
        "Cursor"
    }
    fn id(&self) -> &'static str {
        "cursor"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (theme, size) = detect_cursor()?;

        let tmpl = config::get().format("cursor").unwrap_or(if size.is_some() {
            "{name} ({size}px)"
        } else {
            "{name}"
        });

        let mut ctx = template::Context::new("cursor");
        ctx.set_str("name", theme.clone());
        ctx.set_str("n", theme);
        if let Some(s) = size {
            ctx.set_str("size", s.to_string());
            ctx.set_num("s", s as f64);
        } else {
            ctx.set_str("size", "");
            ctx.set_str("s", "");
        }

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Cursor", formatted))
    }
}

fn detect_cursor() -> Option<(String, Option<u32>)> {
    if let Ok(theme) = std::env::var("XCURSOR_THEME")
        && !theme.trim().is_empty()
    {
        let size = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s| s.parse().ok());
        return Some((theme.trim().to_string(), size));
    }

    let home = std::env::var("HOME").ok().map(PathBuf::from)?;

    // KDE: ~/.config/kcminputrc
    let kcm = home.join(".config/kcminputrc");
    if let Ok(text) = fs::read_to_string(kcm) {
        let mut theme = None;
        let mut size = None;
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "cursorTheme" => theme = Some(v.trim().to_string()),
                    "cursorSize" => size = v.trim().parse().ok(),
                    _ => {}
                }
            }
        }
        if let Some(t) = theme {
            return Some((t, size));
        }
    }

    // GTK: ~/.config/gtk-3.0/settings.ini
    let gtk = home.join(".config/gtk-3.0/settings.ini");
    if let Ok(text) = fs::read_to_string(gtk) {
        let mut theme = None;
        let mut size = None;
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "gtk-cursor-theme-name" => theme = Some(v.trim().to_string()),
                    "gtk-cursor-theme-size" => size = v.trim().parse().ok(),
                    _ => {}
                }
            }
        }
        if let Some(t) = theme {
            return Some((t, size));
        }
    }

    // Last resort: the icon theme's own default cursor.
    let default_theme = home.join(".icons/default/index.theme");
    if let Ok(text) = fs::read_to_string(default_theme) {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("Inherits=") {
                let name = rest.trim().to_string();
                if !name.is_empty() {
                    return Some((name, None));
                }
            }
        }
    }

    None
}
