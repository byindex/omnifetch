use crate::module::{Module, ModuleOutput};

pub struct Locale;

impl Module for Locale {
    fn name(&self) -> &'static str {
        "Locale"
    }
    fn id(&self) -> &'static str {
        "locale"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let l = std::env::var("LC_ALL")
            .ok()
            .or_else(|| std::env::var("LC_CTYPE").ok())
            .or_else(|| std::env::var("LANG").ok())?;
        if l.is_empty() || l == "C" || l == "POSIX" {
            return None;
        }
        Some(ModuleOutput::new(
            "Locale",
            l.replace('_', "-").replace(".UTF-8", ""),
        ))
    }
}
