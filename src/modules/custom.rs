use crate::config;
use crate::module::{Module, ModuleOutput};

pub struct Custom;

impl Module for Custom {
    fn name(&self) -> &'static str {
        "Custom"
    }

    fn id(&self) -> &'static str {
        "custom"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let text = config::get().format("custom")?;
        Some(ModuleOutput::new("Custom", text.to_string()))
    }
}
