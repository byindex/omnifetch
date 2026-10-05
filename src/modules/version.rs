use crate::config;
use crate::module::{Module, ModuleOutput};

pub struct Version;

impl Module for Version {
    fn name(&self) -> &'static str {
        "Version"
    }
    fn id(&self) -> &'static str {
        "version"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        Some(ModuleOutput::new(
            "Version",
            format!("omnifetch {} ({os} {arch})", config::VERSION),
        ))
    }
}
