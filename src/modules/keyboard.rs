use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::collections::HashSet;

pub struct Keyboard;

impl Module for Keyboard {
    fn name(&self) -> &'static str {
        "Keyboard"
    }
    fn id(&self) -> &'static str {
        "keyboard"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut keyboards = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Some(content) = sys::proc_input_devices() {
                let mut current_name: Option<String> = None;
                let mut is_kbd = false;
                let mut seen = HashSet::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        if is_kbd
                            && let Some(name) = current_name.take()
                            && is_valid_keyboard(&name)
                        {
                            let clean = clean_keyboard_name(&name);
                            if seen.insert(clean.clone()) {
                                keyboards.push(clean);
                            }
                        }
                        current_name = None;
                        is_kbd = false;
                        continue;
                    }

                    if let Some(rest) = line.strip_prefix("N: Name=") {
                        let trimmed = rest.trim().trim_matches('"');
                        current_name = Some(trimmed.to_string());
                    } else if let Some(rest) = line.strip_prefix("H: Handlers=")
                        && rest.split_whitespace().any(|h| h == "kbd")
                    {
                        is_kbd = true;
                    }
                }

                if is_kbd
                    && let Some(name) = current_name
                    && is_valid_keyboard(&name)
                {
                    let clean = clean_keyboard_name(&name);
                    if seen.insert(clean.clone()) {
                        keyboards.push(clean);
                    }
                }
            }
        }

        if keyboards.is_empty() {
            None
        } else {
            Some(ModuleOutput::new("Keyboard", keyboards.join(", ")))
        }
    }
}

fn is_valid_keyboard(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.contains("button")
        || lower.contains("switch")
        || lower.contains("video bus")
        || lower.contains("hotkey")
        || lower.contains("control")
        || lower.contains("speaker")
        || lower.contains("mic")
        || lower.contains("headphone")
        || lower.contains("headset")
        || lower.contains("earbuds")
        || lower.contains("earphone")
        || lower.contains("airpods")
        || lower.contains("audio")
        || lower.contains("avrcp")
        || lower.contains("wireless")
        || lower.contains("mouse")
    {
        return false;
    }
    true
}

fn clean_keyboard_name(raw: &str) -> String {
    let s = raw.trim().trim_matches('"').trim();
    // A name doubled by udev, such as "Usb KeyBoard Usb KeyBoard", collapses to one half.
    let words: Vec<&str> = s.split_whitespace().collect();
    let s_clean = if words.len() >= 2 && words.len().is_multiple_of(2) {
        let mid = words.len() / 2;
        let (first, second) = words.split_at(mid);
        let first_lower: Vec<String> = first.iter().map(|w| w.to_lowercase()).collect();
        let second_lower: Vec<String> = second.iter().map(|w| w.to_lowercase()).collect();
        if first_lower == second_lower {
            first.join(" ")
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };

    if s_clean.eq_ignore_ascii_case("usb keyboard") {
        return "USB Keyboard".to_string();
    }
    if s_clean.eq_ignore_ascii_case("at translated set 2 keyboard") {
        return "AT Translated Set 2 Keyboard (Built-in)".to_string();
    }

    s_clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_keyboard_name() {
        assert_eq!(
            clean_keyboard_name("Usb KeyBoard Usb KeyBoard"),
            "USB Keyboard"
        );
        assert_eq!(
            clean_keyboard_name("AT Translated Set 2 keyboard"),
            "AT Translated Set 2 Keyboard (Built-in)"
        );
        assert_eq!(clean_keyboard_name("Keychron K2"), "Keychron K2");
    }

    #[test]
    fn test_is_valid_keyboard() {
        assert!(is_valid_keyboard("AT Translated Set 2 keyboard"));
        assert!(is_valid_keyboard("Usb KeyBoard Usb KeyBoard"));
        assert!(!is_valid_keyboard("Proove Tender (AVRCP)"));
        assert!(!is_valid_keyboard("Sony WH-1000XM4 (AVRCP)"));
        assert!(!is_valid_keyboard("Power Button"));
    }
}
