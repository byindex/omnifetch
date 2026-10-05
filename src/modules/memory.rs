use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;

pub struct Memory;

impl Module for Memory {
    fn name(&self) -> &'static str {
        "Memory"
    }
    fn id(&self) -> &'static str {
        "memory"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let m = sys::meminfo();
        if m.total_kb == 0 {
            return None;
        }
        let used = m.total_kb.saturating_sub(m.available_kb);
        let free = m.available_kb;
        let pct = used as f64 / m.total_kb as f64 * 100.0;

        let tmpl = config::get()
            .format("memory")
            .unwrap_or("{used} / {total} {bar} {pct}%");

        let used_bytes = used * 1024;
        let total_bytes = m.total_kb * 1024;
        let free_bytes = free * 1024;

        let mut ctx = template::Context::new("memory").with_metric(used, m.total_kb, pct);
        ctx.set_bytes("used", used_bytes);
        ctx.set_bytes("u", used_bytes);
        ctx.set_bytes("total", total_bytes);
        ctx.set_bytes("t", total_bytes);
        ctx.set_bytes("free", free_bytes);
        ctx.set_bytes("f", free_bytes);
        ctx.set_bytes("available", free_bytes);
        ctx.set_bytes("a", free_bytes);
        ctx.set_num("pct", pct);
        ctx.set_num("p", pct);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Memory", formatted))
    }
}
