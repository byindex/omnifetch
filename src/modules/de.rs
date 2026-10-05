use crate::module::{Module, ModuleOutput};

pub struct De;

impl Module for De {
    fn name(&self) -> &'static str {
        "DE"
    }
    fn id(&self) -> &'static str {
        "de"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let v = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .ok()?
            .replace(':', ", ");
        Some(ModuleOutput::new("DE", v))
    }
}
