use crate::module::{Module, ModuleOutput};

pub struct Editor;

impl Module for Editor {
    fn name(&self) -> &'static str {
        "Editor"
    }
    fn id(&self) -> &'static str {
        "editor"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let e = std::env::var("VISUAL")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("EDITOR").ok().filter(|s| !s.is_empty()))?;
        let bin = e.split_whitespace().next().unwrap_or(&e);
        let name = bin.rsplit('/').next()?.to_string();
        Some(ModuleOutput::new("Editor", name))
    }
}
