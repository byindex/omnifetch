use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::Path;

pub struct AudioServer;

impl Module for AudioServer {
    fn name(&self) -> &'static str {
        "Audio Server"
    }
    fn id(&self) -> &'static str {
        "audioserver"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let uid = unsafe { libc::getuid() };

        // PipeWire first: a socket under /run/user is unambiguous.
        let pw_socket = format!("/run/user/{uid}/pipewire-0");
        if Path::new(&pw_socket).exists() {
            let ver = find_pkg_version("pipewire");
            let out = match ver {
                Some(v) => format!("PipeWire {v}"),
                None => "PipeWire".to_string(),
            };
            return Some(ModuleOutput::new("Audio Server", out));
        }

        // Then PulseAudio, by the same socket trick.
        let pulse_socket = format!("/run/user/{uid}/pulse/native");
        if Path::new(&pulse_socket).exists() {
            let ver = find_pkg_version("pulseaudio");
            let out = match ver {
                Some(v) => format!("PulseAudio {v}"),
                None => "PulseAudio".to_string(),
            };
            return Some(ModuleOutput::new("Audio Server", out));
        }

        // ALSA last, and only by its absence of a better answer.
        if Path::new("/proc/asound/version").exists() {
            return Some(ModuleOutput::new("Audio Server", "ALSA".to_string()));
        }

        None
    }
}

fn find_pkg_version(pkg_name: &str) -> Option<String> {
    // Check pacman
    if let Ok(entries) = fs::read_dir("/var/lib/pacman/local") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s = name.to_string_lossy();
            if let Some(rest) = s.strip_prefix(pkg_name)
                && let Some(rest) = rest.strip_prefix('-')
                && rest.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                let without_epoch = if let Some((_, r)) = rest.split_once(':') {
                    r
                } else {
                    rest
                };
                let ver = without_epoch
                    .rsplit_once('-')
                    .map(|(v, _)| v)
                    .unwrap_or(without_epoch);
                return Some(ver.to_string());
            }
        }
    }
    None
}
