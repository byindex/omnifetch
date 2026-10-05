use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Chassis;

impl Module for Chassis {
    fn name(&self) -> &'static str {
        "Chassis"
    }
    fn id(&self) -> &'static str {
        "chassis"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let raw = sys::dmi("chassis_type")?;
        let t: u32 = raw.parse().ok()?;
        // SMBIOS chassis type, spec 3.1.4.10.
        let name = match t {
            1 | 2 | 3 | 16 | 17 | 23 | 28 | 29 | 30 => "Desktop",
            4..=7 | 14 | 15 | 25 => "Laptop",
            8..=11 => "Portable",
            12 => "Docking Station",
            13 | 34 => "All in One",
            18 => "Blade",
            19..=22 => "Rack Mount",
            24 => "Sealed-case PC",
            26 | 27 => "Compact PCI",
            31 => "Advanced TCA",
            32 | 33 => "Blade Enclosure",
            _ => "Other",
        };
        Some(ModuleOutput::new("Chassis", name))
    }
}
