use crate::config;
use crate::module::{Field, Module, ModuleOutput};

pub struct Colors;

impl Module for Colors {
    fn name(&self) -> &'static str {
        "Colors"
    }
    fn id(&self) -> &'static str {
        "colors"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if !config::get().color {
            return None;
        }

        let symbol = "\u{2588}\u{2588}\u{2588}";
        let mut row1 = String::with_capacity(64);
        for code in 30..=37 {
            row1.push_str(&format!("\x1b[{code}m{symbol}"));
        }
        row1.push_str("\x1b[0m");

        let mut row2 = String::with_capacity(64);
        for code in 90..=97 {
            row2.push_str(&format!("\x1b[{code}m{symbol}"));
        }
        row2.push_str("\x1b[0m");

        Some(ModuleOutput {
            name: "Colors".to_string(),
            fields: vec![Field::new("", row1), Field::cont(row2)],
        })
    }
}
