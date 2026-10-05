use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Board;

impl Module for Board {
    fn name(&self) -> &'static str {
        "Board"
    }
    fn id(&self) -> &'static str {
        "board"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let vendor = sys::dmi("board_vendor");
        let name = sys::dmi("board_name");
        let v = match (vendor, name) {
            (None, None) => return None,
            (Some(v), None) | (None, Some(v)) => v,
            (Some(v), Some(n)) => {
                if n.contains(&v) {
                    n
                } else {
                    format!("{v} {n}")
                }
            }
        };
        Some(ModuleOutput::new("Board", v))
    }
}
