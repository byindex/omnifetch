use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct PhysicalDisk;

impl Module for PhysicalDisk {
    fn name(&self) -> &'static str {
        "PhysicalDisk"
    }

    fn id(&self) -> &'static str {
        "physicaldisk"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let entries = fs::read_dir("/sys/block").ok()?;
        let mut disks = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip virtual/pseudo block devices
            if name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("zram")
                || name.starts_with("dm-")
                || name.starts_with("sr")
                || name.starts_with("fd")
            {
                continue;
            }

            let path = entry.path();
            let size_sectors = sys::read_num(path.join("size")).unwrap_or(0.0) as u64;
            if size_sectors == 0 {
                continue;
            }

            let size_bytes = size_sectors * 512;
            let size_str = sys::human_bytes(size_bytes);

            let model = sys::read_trim(path.join("device/model"))
                .or_else(|| sys::read_trim(path.join("model")))
                .unwrap_or_else(|| name.clone());

            let is_rotational = sys::read_trim(path.join("queue/rotational"))
                .map(|s| s == "1")
                .unwrap_or(false);

            let disk_type = if name.starts_with("nvme") {
                "NVMe SSD"
            } else if is_rotational {
                "HDD"
            } else {
                "SSD"
            };

            disks.push(format!("{model} ({size_str}) [{disk_type}]"));
        }

        if disks.is_empty() {
            return None;
        }

        Some(ModuleOutput::multi("PhysicalDisk", disks))
    }
}
