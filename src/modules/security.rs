use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::Path;

pub struct Security;

impl Module for Security {
    fn name(&self) -> &'static str {
        "Security"
    }
    fn id(&self) -> &'static str {
        "security"
    }
    fn run(&self) -> Option<ModuleOutput> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(lsm_str) = fs::read_to_string("/sys/kernel/security/lsm") {
                let mut modules = Vec::new();
                for raw in lsm_str.trim().split(',') {
                    let item = raw.trim();
                    match item {
                        "apparmor" => {
                            let mode = if Path::new("/sys/kernel/security/apparmor").exists() {
                                "AppArmor (enforcing)"
                            } else {
                                "AppArmor"
                            };
                            modules.push(mode.to_string());
                        }
                        "selinux" => {
                            let mode = if let Ok(s) = fs::read_to_string("/sys/fs/selinux/enforce")
                            {
                                if s.trim() == "1" {
                                    "SELinux (enforcing)"
                                } else {
                                    "SELinux (permissive)"
                                }
                            } else {
                                "SELinux"
                            };
                            modules.push(mode.to_string());
                        }
                        "landlock" => modules.push("Landlock".to_string()),
                        "lockdown" => modules.push("Lockdown".to_string()),
                        "yama" => modules.push("Yama".to_string()),
                        "tomoyo" => modules.push("Tomoyo".to_string()),
                        "bpf" => modules.push("BPF-LSM".to_string()),
                        other if !other.is_empty() && other != "capability" => {
                            let capitalized =
                                format!("{}{}", other[..1].to_uppercase(), &other[1..]);
                            modules.push(capitalized);
                        }
                        _ => {}
                    }
                }
                if !modules.is_empty() {
                    return Some(ModuleOutput::new("Security", modules.join(", ")));
                }
            }
        }

        None
    }
}
