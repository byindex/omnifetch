use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::PathBuf;

pub struct Wallpaper;

impl Module for Wallpaper {
    fn name(&self) -> &'static str {
        "Wallpaper"
    }
    fn id(&self) -> &'static str {
        "wallpaper"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let path = detect_wallpaper()?;
        Some(ModuleOutput::new("Wallpaper", path))
    }
}

fn detect_wallpaper() -> Option<String> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;

    // KDE Plasma: plasma-org.kde.plasma.desktop-appletsrc
    let kde_applets = home.join(".config/plasma-org.kde.plasma.desktop-appletsrc");
    if let Ok(text) = fs::read_to_string(kde_applets) {
        for line in text.lines().rev() {
            if let Some(rest) = line.strip_prefix("Image=") {
                let s = rest.trim();
                let clean = s.strip_prefix("file://").unwrap_or(s);
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }

    // Feh: ~/.fehbg
    let fehbg = home.join(".fehbg");
    if let Ok(text) = fs::read_to_string(fehbg) {
        for line in text.lines() {
            if line.contains("feh")
                && let Some(last_arg) = line.split_whitespace().last()
            {
                let clean = last_arg.trim_matches('\'').trim_matches('"');
                if clean.starts_with('/') || clean.starts_with('~') {
                    return Some(clean.to_string());
                }
            }
        }
    }

    // GNOME: gsettings
    if let Ok(out) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-uri"])
        .output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout)
            .trim()
            .trim_matches('\'')
            .to_string();
        let clean = s.strip_prefix("file://").unwrap_or(&s).to_string();
        if !clean.is_empty() {
            return Some(clean);
        }
    }

    None
}
