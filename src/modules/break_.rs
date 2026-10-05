use crate::module::{Module, ModuleOutput};

pub struct Break;

impl Module for Break {
    fn name(&self) -> &'static str {
        "Break"
    }
    fn id(&self) -> &'static str {
        "break"
    }
    fn run(&self) -> Option<ModuleOutput> {
        Some(ModuleOutput::new("Break", ""))
    }
}
