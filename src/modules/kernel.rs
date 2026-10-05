use crate::module::{Module, ModuleOutput};
use crate::utsname::uname_ref;

pub struct Kernel;

impl Module for Kernel {
    fn name(&self) -> &'static str {
        "Kernel"
    }
    fn id(&self) -> &'static str {
        "kernel"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let u = uname_ref()?;
        Some(ModuleOutput::new("Kernel", &u.release))
    }
}
