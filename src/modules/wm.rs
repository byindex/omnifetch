use crate::module::{Module, ModuleOutput};

pub struct Wm;

impl Module for Wm {
    fn name(&self) -> &'static str {
        "WM"
    }
    fn id(&self) -> &'static str {
        "wm"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if let Some(v) = std::env::var("XDG_CURRENT_DESKTOP")
            .ok()
            .filter(|s| !s.is_empty())
        {
            let de = v.split(':').next().unwrap_or(&v).to_string();
            if let Some(wm) = wm_for_de(&de) {
                return Some(ModuleOutput::new("WM", wm));
            }
            return Some(ModuleOutput::new("WM", de));
        }
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return Some(ModuleOutput::new("WM", "Hyprland"));
        }
        if std::env::var("SWAYSOCK").is_ok() {
            return Some(ModuleOutput::new("WM", "Sway"));
        }
        None
    }
}

fn wm_for_de(de: &str) -> Option<String> {
    Some(
        match de {
            d if d.contains("KDE") => "KDE Plasma",
            d if d.contains("GNOME") => "Mutter",
            d if d.contains("XFCE") => "Xfce",
            d if d.contains("MATE") => "Marco",
            d if d.contains("LXQt") => "lxqt",
            d if d.contains("Budgie") => "Budgie",
            d if d.contains("Cosmic") => "Cosmic",
            d if d.contains("Hyprland") => "Hyprland",
            _ => return None,
        }
        .to_string(),
    )
}
