use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::collections::HashSet;

pub struct Touchpad;

impl Module for Touchpad {
    fn name(&self) -> &'static str {
        "Touchpad"
    }
    fn id(&self) -> &'static str {
        "touchpad"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut devices = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Some(content) = sys::proc_input_devices() {
                let mut current_name: Option<String> = None;
                let mut is_touchpad_device = false;
                let mut seen = HashSet::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        if is_touchpad_device
                            && let Some(name) = current_name.take()
                            && seen.insert(name.clone())
                        {
                            devices.push(format_touchpad_name(&name));
                        }
                        current_name = None;
                        is_touchpad_device = false;
                        continue;
                    }

                    if let Some(rest) = line.strip_prefix("N: Name=") {
                        let trimmed = rest.trim().trim_matches('"').trim();
                        current_name = Some(trimmed.to_string());
                    } else if let Some(rest) = line.strip_prefix("H: Handlers=")
                        && rest
                            .split_whitespace()
                            .any(|h| h.starts_with("mouse") || h.starts_with("event"))
                        && let Some(ref name) = current_name
                        && is_touchpad_or_stick(name)
                    {
                        is_touchpad_device = true;
                    }
                }

                if is_touchpad_device
                    && let Some(name) = current_name
                    && seen.insert(name.clone())
                {
                    devices.push(format_touchpad_name(&name));
                }
            }
        }

        if devices.is_empty() {
            None
        } else if devices.len() == 1 {
            Some(ModuleOutput::new("Touchpad", &devices[0]))
        } else {
            Some(ModuleOutput::multi("Touchpad", devices))
        }
    }
}

fn is_touchpad_or_stick(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("touchpad")
        || lower.contains("trackpad")
        || lower.contains("point stick")
        || lower.contains("trackpoint")
        || lower.contains("pointing stick")
        || (lower.contains("stick") && lower.contains("point"))
}

fn format_touchpad_name(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("stick") || lower.contains("trackpoint") {
        if !lower.contains("trackpoint") {
            format!("{name} (TrackPoint)")
        } else {
            name.to_string()
        }
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_touchpad_or_stick() {
        assert!(is_touchpad_or_stick("AlpsPS/2 ALPS DualPoint TouchPad"));
        assert!(is_touchpad_or_stick("AlpsPS/2 ALPS DualPoint Stick"));
        assert!(is_touchpad_or_stick("Synaptics TouchPad"));
        assert!(!is_touchpad_or_stick("INSTANT USB GAMING MOUSE"));
    }
}
