use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct TerminalSize;

impl Module for TerminalSize {
    fn name(&self) -> &'static str {
        "Terminal Size"
    }
    fn id(&self) -> &'static str {
        "terminalsize"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let size = sys::terminal_size()?;
        let display = if size.xpixel > 0 && size.ypixel > 0 {
            format!(
                "{} columns x {} rows ({}x{} px)",
                size.cols, size.rows, size.xpixel, size.ypixel
            )
        } else {
            format!("{} columns x {} rows", size.cols, size.rows)
        };
        Some(ModuleOutput::new("Terminal Size", display))
    }
}
