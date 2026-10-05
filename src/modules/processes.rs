use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Processes;

impl Module for Processes {
    fn name(&self) -> &'static str {
        "Processes"
    }
    fn id(&self) -> &'static str {
        "processes"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (procs, threads) = sys::process_and_thread_count();
        Some(ModuleOutput::new(
            "Processes",
            format!("{procs} ({threads} threads)"),
        ))
    }
}
