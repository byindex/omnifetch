use crate::cmd;
use crate::config;
use crate::module::{Module, ModuleOutput};

pub struct Command;

impl Module for Command {
    fn name(&self) -> &'static str {
        "Command"
    }

    fn id(&self) -> &'static str {
        "command"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let cmd_str = config::get().format("command")?;
        let output = cmd::run("sh", &["-c", cmd_str])?;
        Some(ModuleOutput::new("Command", output))
    }
}
