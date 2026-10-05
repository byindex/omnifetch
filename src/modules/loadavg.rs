use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Loadavg;

impl Module for Loadavg {
    fn name(&self) -> &'static str {
        "Load"
    }
    fn id(&self) -> &'static str {
        "loadavg"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let s = sys::read_trim("/proc/loadavg")?;
        let mut it = s.split_whitespace();
        let l = [it.next()?, it.next()?, it.next()?];
        Some(ModuleOutput::new(
            "Load",
            format!("{}/{}/{} procs/{}", l[0], l[1], l[2], it.next()?),
        ))
    }
}
