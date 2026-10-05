use crate::module::{Module, ModuleOutput};
use std::fs;

pub struct Tpm;

impl Module for Tpm {
    fn name(&self) -> &'static str {
        "TPM"
    }
    fn id(&self) -> &'static str {
        "tpm"
    }
    fn run(&self) -> Option<ModuleOutput> {
        #[cfg(target_os = "linux")]
        {
            let tpm_dir = std::path::Path::new("/sys/class/tpm/tpm0");
            if !tpm_dir.exists() {
                return None;
            }

            let version_major = fs::read_to_string(tpm_dir.join("tpm_version_major"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let desc = fs::read_to_string(tpm_dir.join("device/description"))
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let ver_str = match version_major.as_str() {
                "2" => "2.0",
                "1" => "1.2",
                other if !other.is_empty() => other,
                _ => "Present",
            };

            let out = if !desc.is_empty() {
                format!("{ver_str} ({desc})")
            } else {
                ver_str.to_string()
            };

            return Some(ModuleOutput::new("TPM", out));
        }

        #[allow(unreachable_code)]
        None
    }
}
