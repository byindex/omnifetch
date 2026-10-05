use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct CpuCache;

impl Module for CpuCache {
    fn name(&self) -> &'static str {
        "Cache"
    }
    fn id(&self) -> &'static str {
        "cpucache"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let cache_key = "cpucache";
        if let Some(cached) = sys::cache_get(cache_key) {
            let lines: Vec<String> = cached.lines().map(String::from).collect();
            if !lines.is_empty() {
                return Some(ModuleOutput::multi("Cache", lines));
            }
        }
        let mut values: Vec<String> = Vec::new();
        for i in 0..10 {
            let base = format!("/sys/devices/system/cpu/cpu0/cache/index{i}");
            if !std::path::Path::new(&base).exists() {
                break;
            }
            let level = sys::read_num(format!("{base}/level")).unwrap_or(0.0) as u32;
            let size = sys::read_num(format!("{base}/size")).unwrap_or(0.0);
            let kind = sys::read_trim(format!("{base}/type")).unwrap_or_default();
            let label = match (kind.as_str(), level) {
                ("Instruction", 1) => "L1 Instruction".to_string(),
                ("Data", 1) => "L1 Data".to_string(),
                ("Unified", 1) => "L1".to_string(),
                ("Unified", 2) => "L2".to_string(),
                ("Unified", 3) => "L3".to_string(),
                (_, l) => format!("L{l} {kind}"),
            };
            values.push(format!("{label}: {} KiB", size as u64));
        }
        if !values.is_empty() {
            sys::cache_set(cache_key, &values.join("\n"));
            Some(ModuleOutput::multi("Cache", values))
        } else {
            None
        }
    }
}
