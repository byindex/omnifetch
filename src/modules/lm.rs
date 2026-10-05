use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::Path;

pub struct Lm;

impl Module for Lm {
    fn name(&self) -> &'static str {
        "LM"
    }

    fn id(&self) -> &'static str {
        "lm"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let lm = detect_lm()?;
        Some(ModuleOutput::new("LM", lm))
    }
}

fn detect_lm() -> Option<String> {
    // 1. Check systemd display-manager.service symlink
    if let Ok(target) = fs::read_link("/etc/systemd/system/display-manager.service") {
        let target_str = target.to_string_lossy().to_lowercase();
        if target_str.contains("sddm") {
            return Some("SDDM".to_string());
        } else if target_str.contains("gdm") {
            return Some("GDM".to_string());
        } else if target_str.contains("lightdm") {
            return Some("LightDM".to_string());
        } else if target_str.contains("ly") {
            return Some("Ly".to_string());
        } else if target_str.contains("greetd") {
            return Some("greetd".to_string());
        } else if target_str.contains("lxdm") {
            return Some("LXDM".to_string());
        } else if target_str.contains("slim") {
            return Some("SLiM".to_string());
        }
    }

    // 2. Check /etc/X11/default-display-manager (Debian / Ubuntu)
    if let Ok(content) = fs::read_to_string("/etc/X11/default-display-manager") {
        let line = content.trim().to_lowercase();
        if let Some(name) = Path::new(&line).file_name().and_then(|f| f.to_str()) {
            return match name {
                "gdm" | "gdm3" => Some("GDM".to_string()),
                "sddm" => Some("SDDM".to_string()),
                "lightdm" => Some("LightDM".to_string()),
                "lxdm" => Some("LXDM".to_string()),
                "slim" => Some("SLiM".to_string()),
                other => Some(other.to_string()),
            };
        }
    }

    None
}
