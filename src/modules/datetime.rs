use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct DateTime;

impl Module for DateTime {
    fn name(&self) -> &'static str {
        "Date/Time"
    }
    fn id(&self) -> &'static str {
        "datetime"
    }
    fn run(&self) -> Option<ModuleOutput> {
        sys::strftime_local("%a %d %b %Y %r %Z").map(|d| ModuleOutput::new("Date/Time", d))
    }
}
