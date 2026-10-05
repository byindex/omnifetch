use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Brightness;

impl Module for Brightness {
    fn name(&self) -> &'static str {
        "Brightness"
    }
    fn id(&self) -> &'static str {
        "brightness"
    }
    fn run(&self) -> Option<ModuleOutput> {
        for e in std::fs::read_dir("/sys/class/backlight").ok()?.flatten() {
            let dir = e.path();
            let cur = sys::read_num(dir.join("brightness"))?;
            let max = sys::read_num(dir.join("max_brightness")).unwrap_or(100.0);
            if max <= 0.0 {
                continue;
            }
            return Some(ModuleOutput::new(
                "Brightness",
                format!("{:.0}%", cur / max * 100.0),
            ));
        }
        None
    }
}
