use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct InitSystem;

impl Module for InitSystem {
    fn name(&self) -> &'static str {
        "Init"
    }
    fn id(&self) -> &'static str {
        "initsystem"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let comm = sys::read_trim("/proc/1/comm")?;
        Some(ModuleOutput::new("Init", comm))
    }
}
