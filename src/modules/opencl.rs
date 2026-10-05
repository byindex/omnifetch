use crate::module::{Module, ModuleOutput};
use std::fs;

pub struct OpenCl;

impl Module for OpenCl {
    fn name(&self) -> &'static str {
        "OpenCL"
    }

    fn id(&self) -> &'static str {
        "opencl"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let entries = fs::read_dir("/etc/OpenCL/vendors").ok()?;
        let mut platforms = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("icd") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    let s = line.trim();
                    if s.is_empty() || s.starts_with('#') {
                        continue;
                    }
                    let name = if s.contains("intel") || s.contains("igdrcl") {
                        "Intel Graphics OpenCL"
                    } else if s.contains("nvidia") || s.contains("cuda") {
                        "NVIDIA CUDA OpenCL"
                    } else if s.contains("amd") || s.contains("rocm") {
                        "AMD ROCm OpenCL"
                    } else if s.contains("pocl") {
                        "Portable Computing Language (PoCL)"
                    } else if s.contains("mesa") || s.contains("rusticl") {
                        "Mesa Rusticl OpenCL"
                    } else {
                        s
                    };
                    if !platforms.contains(&name.to_string()) {
                        platforms.push(name.to_string());
                    }
                }
            }
        }

        if platforms.is_empty() {
            return None;
        }

        Some(ModuleOutput::new("OpenCL", platforms.join(", ")))
    }
}
