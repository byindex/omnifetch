use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct Top;

impl Module for Top {
    fn name(&self) -> &'static str {
        "Top"
    }

    fn id(&self) -> &'static str {
        "top"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let mut top_name = String::new();
        let mut max_rss_bytes = 0u64;
        let page_size = 4096u64;

        let dir = fs::read_dir("/proc").ok()?;
        // Both the path and the `statm` line live in stack buffers. Once per process.
        let mut statm_path = [0u8; 32];
        let mut comm_path = [0u8; 32];
        let mut statm_buf = [0u8; 128];
        let mut comm_buf = [0u8; 128];
        let prefix = b"/proc/";
        let statm_suffix = b"/statm";
        let comm_suffix = b"/comm";
        statm_path[..prefix.len()].copy_from_slice(prefix);
        comm_path[..prefix.len()].copy_from_slice(prefix);

        let entries = dir.flatten();
        for entry in entries {
            let file_name = entry.file_name();
            let Some(name_str) = file_name.to_str() else {
                continue;
            };
            if !name_str.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let pid_len = name_str.len();
            let base = prefix.len() + pid_len;
            if base + statm_suffix.len() + 1 > statm_path.len() {
                continue;
            }

            statm_path[prefix.len()..base].copy_from_slice(name_str.as_bytes());
            statm_path[base..base + statm_suffix.len()].copy_from_slice(statm_suffix);
            statm_path[base + statm_suffix.len()] = 0;

            // Read /proc/PID/statm: "size resident shared text lib data dt"
            let Some(statm) =
                sys::read_small(&statm_path[..base + statm_suffix.len() + 1], &mut statm_buf)
            else {
                continue;
            };
            let mut parts = statm.split_ascii_whitespace();
            let _size = parts.next();
            let Some(resident_pages_str) = parts.next() else {
                continue;
            };
            let Ok(resident_pages) = resident_pages_str.parse::<u64>() else {
                continue;
            };
            let rss_bytes = resident_pages * page_size;
            if rss_bytes <= max_rss_bytes {
                continue;
            }

            // Only the current leader needs its name read back.
            comm_path[prefix.len()..base].copy_from_slice(name_str.as_bytes());
            comm_path[base..base + comm_suffix.len()].copy_from_slice(comm_suffix);
            comm_path[base + comm_suffix.len()] = 0;
            let Some(comm) =
                sys::read_small(&comm_path[..base + comm_suffix.len() + 1], &mut comm_buf)
            else {
                continue;
            };
            let trimmed = comm.trim();
            if !trimmed.is_empty() {
                max_rss_bytes = rss_bytes;
                top_name = trimmed.to_string();
            }
        }

        if max_rss_bytes == 0 || top_name.is_empty() {
            return None;
        }

        let rss_str = sys::human_bytes(max_rss_bytes);
        Some(ModuleOutput::new("Top", format!("{top_name} ({rss_str})")))
    }
}
