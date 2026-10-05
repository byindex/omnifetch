use std::fs;

use crate::module::{Module, ModuleOutput};

pub struct Vulkan;

impl Module for Vulkan {
    fn name(&self) -> &'static str {
        "Vulkan"
    }
    fn id(&self) -> &'static str {
        "vulkan"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut api_versions = Vec::new();
        for dir in ["/usr/share/vulkan/icd.d", "/etc/vulkan/icd.d"] {
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for e in entries.flatten() {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path)
                    && let Some((_, rest)) = content.split_once("\"api_version\":")
                    && let Some(val) = rest.split('"').nth(1)
                    && !api_versions.contains(&val.to_string())
                {
                    api_versions.push(val.to_string());
                }
            }
        }

        if api_versions.is_empty() {
            return None;
        }

        api_versions.sort();
        Some(ModuleOutput::new("Vulkan", api_versions.join(", ")))
    }
}
