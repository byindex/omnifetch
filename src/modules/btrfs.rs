use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct Btrfs;

impl Module for Btrfs {
    fn name(&self) -> &'static str {
        "Btrfs"
    }

    fn id(&self) -> &'static str {
        "btrfs"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let entries = fs::read_dir("/sys/fs/btrfs").ok()?;
        let mut volumes = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip features or non-uuid directories
            if name == "features" || !entry.path().is_dir() {
                continue;
            }

            let path = entry.path();
            let label = sys::read_trim(path.join("label")).unwrap_or_default();
            let display_name = if !label.is_empty() { label } else { name };

            let bytes =
                sys::read_num(path.join("allocation/data/total_bytes")).unwrap_or(0.0) as u64;
            if bytes > 0 {
                let size_str = sys::human_bytes(bytes);
                volumes.push(format!("{display_name} ({size_str} allocated)"));
            } else {
                volumes.push(display_name);
            }
        }

        if volumes.is_empty() {
            return None;
        }

        Some(ModuleOutput::multi("Btrfs", volumes))
    }
}
