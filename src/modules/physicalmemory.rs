use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct PhysicalMemory;

impl Module for PhysicalMemory {
    fn name(&self) -> &'static str {
        "PhysicalMemory"
    }

    fn id(&self) -> &'static str {
        "physicalmemory"
    }

    fn run(&self) -> Option<ModuleOutput> {
        // Read DMI entries 17-*
        let mut slots = Vec::new();
        if let Ok(entries) = fs::read_dir("/sys/firmware/dmi/entries") {
            for e in entries.flatten() {
                let fname = e.file_name();
                if fname.as_encoded_bytes().starts_with(b"17-")
                    && let Ok(s) = fname.into_string()
                {
                    slots.push(s);
                }
            }
        }
        slots.sort();

        if slots.is_empty() {
            return None;
        }

        let mut modules = Vec::new();
        for slot in &slots {
            let path = format!("/sys/firmware/dmi/entries/{slot}/raw");
            if let Ok(bytes) = fs::read(&path)
                && bytes.len() >= 0x16
            {
                let size_raw = u16::from_le_bytes([bytes[0x0c], bytes[0x0d]]);
                if size_raw != 0 && size_raw != 0xffff {
                    let size_mb = if (size_raw & 0x8000) == 0 {
                        size_raw as u64
                    } else {
                        ((size_raw & 0x7fff) as u64) / 1024
                    };
                    let type_id = bytes[0x12];
                    let mem_type = match type_id {
                        0x12 => "DDR",
                        0x13 => "DDR2",
                        0x18 => "DDR3",
                        0x1a => "DDR4",
                        0x22 => "DDR5",
                        0x1e => "LPDDR4",
                        0x23 => "LPDDR5",
                        _ => "RAM",
                    };
                    let speed = if bytes.len() >= 0x17 {
                        let sp = u16::from_le_bytes([bytes[0x15], bytes[0x16]]);
                        if sp > 0 {
                            format!(" {sp} MT/s")
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    modules.push(format!("{size_mb} MiB {mem_type}{speed}"));
                }
            }
        }

        if !modules.is_empty() {
            return Some(ModuleOutput::multi("PhysicalMemory", modules));
        }

        // Without permission to read the DIMM sizes, divide MemTotal by the slot count.
        let total_kb = sys::meminfo().total_kb;
        if total_kb > 0 {
            let slot_count = slots.len() as u64;
            let total_gb = (total_kb as f64 / 1024.0 / 1024.0).ceil() as u64;
            let per_slot_gb = (total_gb / slot_count.max(1)).max(1);
            let mut list = Vec::with_capacity(slot_count as usize);
            for i in 1..=slot_count {
                list.push(format!("{per_slot_gb} GiB [Slot {i}]"));
            }
            return Some(ModuleOutput::multi("PhysicalMemory", list));
        }

        None
    }
}
