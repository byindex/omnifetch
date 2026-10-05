use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Bios;

impl Module for Bios {
    fn name(&self) -> &'static str {
        "BIOS"
    }
    fn id(&self) -> &'static str {
        "bios"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let vendor = sys::dmi("bios_vendor")?;
        let version = sys::dmi("bios_version").unwrap_or_default();
        let date = sys::dmi("bios_date").unwrap_or_default();
        let v = [vendor, version, date]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        Some(ModuleOutput::new("BIOS", v))
    }
}
