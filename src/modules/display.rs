use std::fs;

use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Display;

impl Module for Display {
    fn name(&self) -> &'static str {
        "Display"
    }
    fn id(&self) -> &'static str {
        "display"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut displays = Vec::new();
        let entries = fs::read_dir("/sys/class/drm").ok()?;
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            // Match connector dirs like card1-LVDS-1, card0-DP-1
            let Some((_, conn)) = name.split_once('-') else {
                continue;
            };
            if conn.is_empty() || conn.starts_with("card") {
                continue;
            }
            if sys::read_trim(path.join("status")).as_deref() != Some("connected") {
                continue;
            }
            let mode = sys::read_trim(path.join("modes"))
                .and_then(|m| m.lines().next().map(str::to_string))
                .unwrap_or_else(|| "connected".into());
            displays.push(format!("{conn} ({mode})"));
        }

        if displays.is_empty() {
            return None;
        }

        Some(ModuleOutput::multi("Display", displays))
    }
}
