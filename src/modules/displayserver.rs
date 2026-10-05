use crate::module::{Module, ModuleOutput};
use std::env;

pub struct DisplayServer;

impl Module for DisplayServer {
    fn name(&self) -> &'static str {
        "Display Server"
    }
    fn id(&self) -> &'static str {
        "displayserver"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let wayland_disp = env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let x11_disp = env::var("DISPLAY").unwrap_or_default();

        let desc = if !wayland_disp.is_empty() || session_type == "wayland" {
            if !wayland_disp.is_empty() {
                format!("Wayland ({wayland_disp})")
            } else {
                "Wayland".to_string()
            }
        } else if !x11_disp.is_empty() || session_type == "x11" {
            if !x11_disp.is_empty() {
                format!("X11 ({x11_disp})")
            } else {
                "X11".to_string()
            }
        } else if session_type == "tty" {
            "TTY".to_string()
        } else {
            return None;
        };

        Some(ModuleOutput::new("Display Server", desc))
    }
}
