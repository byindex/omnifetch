use crate::module::{Module, ModuleOutput};

pub struct Separator;

impl Module for Separator {
    fn name(&self) -> &'static str {
        "Separator"
    }
    fn id(&self) -> &'static str {
        "separator"
    }
    fn run(&self) -> Option<ModuleOutput> {
        None
    }
}
