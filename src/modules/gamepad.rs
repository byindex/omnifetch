use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::collections::HashSet;

pub struct Gamepad;

impl Module for Gamepad {
    fn name(&self) -> &'static str {
        "Gamepad"
    }
    fn id(&self) -> &'static str {
        "gamepad"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut pads = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Some(content) = sys::proc_input_devices() {
                let mut current_name: Option<String> = None;
                let mut is_js = false;
                let mut seen = HashSet::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        if is_js
                            && let Some(name) = current_name.take()
                            && is_real_gamepad(&name)
                            && seen.insert(name.clone())
                        {
                            pads.push(name);
                        }
                        current_name = None;
                        is_js = false;
                        continue;
                    }

                    if let Some(rest) = line.strip_prefix("N: Name=") {
                        let trimmed = rest.trim().trim_matches('"').trim();
                        current_name = Some(trimmed.to_string());
                    } else if let Some(rest) = line.strip_prefix("H: Handlers=")
                        && rest.split_whitespace().any(|h| h.starts_with("js"))
                    {
                        is_js = true;
                    }
                }

                if is_js
                    && let Some(name) = current_name
                    && is_real_gamepad(&name)
                    && seen.insert(name.clone())
                {
                    pads.push(name);
                }
            }
        }

        if pads.is_empty() {
            None
        } else {
            Some(ModuleOutput::new("Gamepad", pads.join(", ")))
        }
    }
}

fn is_real_gamepad(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.contains("accelerometer")
        || lower.contains("sensor")
        || lower.contains("lis3lv02d")
        || lower.contains("hdmi")
    {
        return false;
    }
    lower.contains("gamepad")
        || lower.contains("controller")
        || lower.contains("joystick")
        || lower.contains("xbox")
        || lower.contains("playstation")
        || lower.contains("dualsense")
        || lower.contains("dualshock")
        || lower.contains("nintendo")
}
