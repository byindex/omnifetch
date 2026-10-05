use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::collections::HashSet;

pub struct Mouse;

impl Module for Mouse {
    fn name(&self) -> &'static str {
        "Mouse"
    }
    fn id(&self) -> &'static str {
        "mouse"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut mice = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Some(content) = sys::proc_input_devices() {
                let mut current_name: Option<String> = None;
                let mut is_mouse = false;
                let mut seen = HashSet::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        if is_mouse
                            && let Some(name) = current_name.take()
                            && is_valid_mouse(&name)
                            && seen.insert(name.clone())
                        {
                            mice.push(clean_mouse_name(&name));
                        }
                        current_name = None;
                        is_mouse = false;
                        continue;
                    }

                    if let Some(rest) = line.strip_prefix("N: Name=") {
                        let trimmed = rest.trim().trim_matches('"').trim();
                        current_name = Some(trimmed.to_string());
                    } else if let Some(rest) = line.strip_prefix("H: Handlers=")
                        && rest.split_whitespace().any(|h| h.starts_with("mouse"))
                    {
                        is_mouse = true;
                    }
                }

                if is_mouse
                    && let Some(name) = current_name
                    && is_valid_mouse(&name)
                    && seen.insert(name.clone())
                {
                    mice.push(clean_mouse_name(&name));
                }
            }
        }

        if mice.is_empty() {
            None
        } else {
            Some(ModuleOutput::new("Mouse", mice.join(", ")))
        }
    }
}

fn is_valid_mouse(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.contains("accelerometer")
        || lower.contains("keyboard")
        || lower.contains("headset")
        || lower.contains("speaker")
        || lower.contains("touchpad")
        || lower.contains("trackpad")
        || lower.contains("point stick")
        || lower.contains("trackpoint")
        || lower.contains("pointing stick")
        || (lower.contains("stick") && lower.contains("point"))
    {
        return false;
    }
    true
}

fn clean_mouse_name(raw: &str) -> String {
    let s = raw.trim().trim_matches('"').trim();
    // A name doubled by udev, such as "Foo Mouse Foo Mouse", collapses to one half.
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() >= 2 && words.len().is_multiple_of(2) {
        let mid = words.len() / 2;
        let (first, second) = words.split_at(mid);
        let first_lower: Vec<String> = first.iter().map(|w| w.to_lowercase()).collect();
        let second_lower: Vec<String> = second.iter().map(|w| w.to_lowercase()).collect();
        if first_lower == second_lower {
            return first.join(" ");
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_mouse() {
        assert!(is_valid_mouse("INSTANT USB GAMING MOUSE"));
        assert!(is_valid_mouse("Logitech G Pro"));
        assert!(!is_valid_mouse("AlpsPS/2 ALPS DualPoint TouchPad"));
        assert!(!is_valid_mouse("AlpsPS/2 ALPS DualPoint Stick"));
        assert!(!is_valid_mouse("Synaptics TouchPad"));
    }
}
