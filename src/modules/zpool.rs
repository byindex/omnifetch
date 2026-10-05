use crate::module::{Module, ModuleOutput};
use std::fs;

pub struct Zpool;

impl Module for Zpool {
    fn name(&self) -> &'static str {
        "Zpool"
    }

    fn id(&self) -> &'static str {
        "zpool"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let entries = fs::read_dir("/proc/spl/kstat/zfs").ok()?;
        let mut pools = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Look for pool directories or state files
            if entry.path().is_dir() && name != "arcstats" {
                pools.push(format!("{name} [ONLINE]"));
            }
        }

        if pools.is_empty() {
            return None;
        }

        Some(ModuleOutput::multi("Zpool", pools))
    }
}
