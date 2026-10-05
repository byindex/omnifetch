use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct Monitor;

impl Module for Monitor {
    fn name(&self) -> &'static str {
        "Monitor"
    }

    fn id(&self) -> &'static str {
        "monitor"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let mut monitors = Vec::new();
        let entries = fs::read_dir("/sys/class/drm").ok()?;
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
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
                .unwrap_or_else(|| "1920x1080".into());

            // Try to extract monitor name from EDID if readable
            let mut monitor_name = conn.to_string();
            if let Ok(edid) = fs::read(path.join("edid"))
                && edid.len() >= 128
            {
                // Check ASCII string descriptors at offsets 54, 72, 90, 108
                for &desc_offset in &[54, 72, 90, 108] {
                    if desc_offset + 18 <= edid.len()
                        && edid[desc_offset] == 0
                        && edid[desc_offset + 1] == 0
                        && edid[desc_offset + 2] == 0
                        && edid[desc_offset + 3] == 0xfc
                    {
                        let name_bytes = &edid[desc_offset + 5..desc_offset + 18];
                        let s = String::from_utf8_lossy(name_bytes).trim().to_string();
                        if !s.is_empty() {
                            monitor_name = s;
                            break;
                        }
                    }
                }
            }

            monitors.push(format!("{monitor_name} ({mode})"));
        }

        if monitors.is_empty() {
            return None;
        }

        Some(ModuleOutput::multi("Monitor", monitors))
    }
}
