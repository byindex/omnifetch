use crate::module::{Module, ModuleOutput};
use std::fs;

pub struct Camera;

impl Module for Camera {
    fn name(&self) -> &'static str {
        "Camera"
    }
    fn id(&self) -> &'static str {
        "camera"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut cameras = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = fs::read_dir("/sys/class/video4linux") {
                let mut seen = std::collections::HashSet::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name_file = path.join("name");
                    if let Ok(raw_name) = fs::read_to_string(&name_file) {
                        let trimmed = raw_name.trim().trim_end_matches(':').trim();
                        if trimmed.is_empty()
                            || trimmed.contains("Metadata")
                            || trimmed.contains("metadata")
                        {
                            continue;
                        }
                        // Clean up underscores
                        let cleaned = trimmed.replace('_', " ");
                        if seen.insert(cleaned.clone()) {
                            cameras.push(cleaned);
                        }
                    }
                }
            }
        }

        if cameras.is_empty() {
            None
        } else {
            Some(ModuleOutput::new("Camera", cameras.join(", ")))
        }
    }
}
