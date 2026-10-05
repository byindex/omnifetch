use crate::module::{Module, ModuleOutput};
use std::collections::HashSet;
use std::fs;

pub struct GpuDriver;

impl Module for GpuDriver {
    fn name(&self) -> &'static str {
        "GPU Driver"
    }
    fn id(&self) -> &'static str {
        "gpudriver"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut drivers = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
                let mut seen = HashSet::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    let class_str = fs::read_to_string(path.join("class")).unwrap_or_default();
                    let class_clean = class_str.trim().trim_start_matches("0x");
                    if class_clean.starts_with("03") {
                        // Display controller (VGA 0300, 3D 0302, Display 0380)
                        if let Ok(driver_target) = fs::read_link(path.join("driver"))
                            && let Some(name) = driver_target.file_name()
                        {
                            let driver_name = name.to_string_lossy().to_string();
                            if seen.insert(driver_name.clone()) {
                                let formatted = format_driver_info(&driver_name);
                                drivers.push(formatted);
                            }
                        }
                    }
                }
            }
        }

        if drivers.is_empty() {
            None
        } else {
            Some(ModuleOutput::new("GPU Driver", drivers.join(", ")))
        }
    }
}

#[cfg(target_os = "linux")]
fn format_driver_info(driver: &str) -> String {
    if driver == "nvidia" {
        if let Ok(content) = fs::read_to_string("/proc/driver/nvidia/version") {
            for line in content.lines() {
                if let Some(pos) = line.find("Kernel Module") {
                    let ver = line[pos + 13..]
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !ver.is_empty() {
                        return format!("NVIDIA {ver}");
                    }
                }
            }
        }
        return "NVIDIA (proprietary)".to_string();
    }
    format!("{driver} (in-tree)")
}
