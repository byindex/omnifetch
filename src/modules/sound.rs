use std::fs;
use std::path::Path;

use crate::cmd;
use crate::module::{Module, ModuleOutput};

pub struct Sound;

impl Module for Sound {
    fn name(&self) -> &'static str {
        "Sound"
    }
    fn id(&self) -> &'static str {
        "sound"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let server = detect_audio_server();
        let card = detect_audio_card();
        let volume = detect_volume();

        let base = match (server, card) {
            (Some(s), Some(c)) => format!("{s} [{c}]"),
            (Some(s), None) => s,
            (None, Some(c)) => c,
            (None, None) => return None,
        };

        let val = match volume {
            Some(v) => format!("{base} ({v})"),
            None => base,
        };

        Some(ModuleOutput::new("Sound", val))
    }
}

fn detect_audio_server() -> Option<String> {
    let uid = unsafe { libc::getuid() };
    let pw_socket = format!("/run/user/{uid}/pipewire-0");
    if Path::new(&pw_socket).exists() {
        return Some("PipeWire".into());
    }
    let pulse_socket = format!("/run/user/{uid}/pulse/native");
    if Path::new(&pulse_socket).exists() {
        return Some("PulseAudio".into());
    }
    None
}

fn detect_audio_card() -> Option<String> {
    let content = fs::read_to_string("/proc/asound/cards").ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        // Format: " 0 [PCH            ]: HDA-Intel - HDA Intel PCH"
        if let Some((_, rest)) = trimmed.split_once("]: ") {
            let name = rest.split(" - ").nth(1).unwrap_or(rest).trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

pub fn parse_wpctl_volume(output: &str) -> Option<String> {
    let trimmed = output.trim();
    let vol_part = trimmed.strip_prefix("Volume:")?.trim();
    let is_muted = vol_part.contains("[MUTED]");
    let num_str = vol_part.replace("[MUTED]", "").trim().to_string();
    let vol_f = num_str.parse::<f64>().ok()?;
    let pct = (vol_f * 100.0).round() as u32;
    if is_muted {
        Some(format!("{pct}% [MUTED]"))
    } else {
        Some(format!("{pct}%"))
    }
}

fn detect_volume_wireplumber() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = std::path::Path::new(&home).join(".local/state/wireplumber/default-routes");
    let content = fs::read_to_string(path).ok()?;

    let mut target_route = None;
    for line in content.lines() {
        if line.contains(":profile:output:")
            && let Some((_, rest)) = line.split_once("=[\"")
            && let Some((route, _)) = rest.split_once("\"]")
        {
            target_route = Some(route.to_string());
            break;
        }
    }

    let mut candidate = None;
    for line in content.lines() {
        if !line.contains(":output:") {
            continue;
        }
        let is_target = target_route
            .as_ref()
            .map(|t| line.contains(t))
            .unwrap_or(false);
        if let Some((_, json_part)) = line.split_once('=')
            && let Some(vol_str) = json_part.split("\"channelVolumes\":[").nth(1)
            && let Some(nums) = vol_str.split(']').next()
        {
            let first_val = nums.split(',').next()?.trim().parse::<f64>().ok()?;
            let is_muted = json_part.contains("\"mute\":true");
            let pct = (first_val * 100.0).round() as u32;
            let s = if is_muted {
                format!("{pct}% [MUTED]")
            } else {
                format!("{pct}%")
            };
            if is_target {
                return Some(s);
            }
            if candidate.is_none() || !is_muted {
                candidate = Some(s);
            }
        }
    }
    candidate
}

fn detect_volume() -> Option<String> {
    if let Some(v) = detect_volume_wireplumber() {
        return Some(v);
    }

    if let Some(out) = cmd::run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"])
        && let Some(v) = parse_wpctl_volume(&out)
    {
        return Some(v);
    }

    if let Some(vol_out) = cmd::run("pactl", &["get-sink-volume", "@DEFAULT_SINK@"])
        && let Some(pct) = vol_out.split_whitespace().find(|w| w.ends_with('%'))
    {
        let is_muted = cmd::run("pactl", &["get-sink-mute", "@DEFAULT_SINK@"])
            .map(|m| m.contains("yes"))
            .unwrap_or(false);
        if is_muted {
            return Some(format!("{pct} [MUTED]"));
        } else {
            return Some(pct.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wpctl_volume() {
        assert_eq!(
            parse_wpctl_volume("Volume: 0.00 [MUTED]").as_deref(),
            Some("0% [MUTED]")
        );
        assert_eq!(parse_wpctl_volume("Volume: 0.65").as_deref(), Some("65%"));
        assert_eq!(parse_wpctl_volume("Volume: 1.00").as_deref(), Some("100%"));
    }

    #[test]
    fn wpctl_volume_rounds_and_tolerates_padding() {
        // Rounding, not truncation: wpctl gives two decimals.
        assert_eq!(parse_wpctl_volume("Volume: 0.005").as_deref(), Some("1%"));
        assert_eq!(parse_wpctl_volume("Volume: 0.004").as_deref(), Some("0%"));
        assert_eq!(parse_wpctl_volume("Volume: 0.335").as_deref(), Some("34%"));
        assert_eq!(
            parse_wpctl_volume("  Volume: 0.50  \n").as_deref(),
            Some("50%")
        );
    }

    #[test]
    fn wpctl_volume_rejects_unexpected_input() {
        assert!(parse_wpctl_volume("").is_none());
        assert!(parse_wpctl_volume("Volume:").is_none());
        assert!(parse_wpctl_volume("Device @ DEFAULT_SINK:").is_none());
        assert!(parse_wpctl_volume("Volume: not-a-number").is_none());
        assert!(parse_wpctl_volume("Volume: [MUTED]").is_none());
    }
}
