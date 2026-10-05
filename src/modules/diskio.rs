use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct DiskIo;

impl Module for DiskIo {
    fn name(&self) -> &'static str {
        "DiskIO"
    }

    fn id(&self) -> &'static str {
        "diskio"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let content = fs::read_to_string("/proc/diskstats").ok()?;
        let mut total_read_bytes = 0u64;
        let mut total_write_bytes = 0u64;

        for line in content.lines() {
            let mut parts = line.split_whitespace();
            let _major = parts.next()?;
            let _minor = parts.next()?;
            let dev = parts.next()?;

            // Only count primary physical block devices (sd[a-z], nvme[0-9]n[0-9], mmcblk[0-9])
            let is_target = (dev.starts_with("sd") && dev.len() == 3)
                || (dev.starts_with("nvme") && dev.contains('n') && !dev.contains('p'))
                || (dev.starts_with("vd") && dev.len() == 3);

            if !is_target {
                continue;
            }

            let sectors_read = parts
                .nth(2)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let sectors_written = parts
                .nth(3)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            total_read_bytes += sectors_read * 512;
            total_write_bytes += sectors_written * 512;
        }

        if total_read_bytes == 0 && total_write_bytes == 0 {
            return None;
        }

        let read_str = sys::human_bytes(total_read_bytes);
        let write_str = sys::human_bytes(total_write_bytes);

        Some(ModuleOutput::new(
            "DiskIO",
            format!("{read_str} (Read) / {write_str} (Write)"),
        ))
    }
}
