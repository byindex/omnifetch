use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;

pub struct Swap;

impl Module for Swap {
    fn name(&self) -> &'static str {
        "Swap"
    }
    fn id(&self) -> &'static str {
        "swap"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let m = sys::meminfo();
        if m.swap_total_kb == 0 {
            return None;
        }
        let used = m.swap_total_kb.saturating_sub(m.swap_free_kb);
        let free = m.swap_free_kb;
        let pct = if m.swap_total_kb > 0 {
            used as f64 / m.swap_total_kb as f64 * 100.0
        } else {
            0.0
        };

        let tmpl = config::get()
            .format("swap")
            .unwrap_or("{used} / {total} {bar} {pct}%");

        let used_bytes = used * 1024;
        let total_bytes = m.swap_total_kb * 1024;
        let free_bytes = free * 1024;

        let mut ctx = template::Context::new("swap").with_metric(used, m.swap_total_kb, pct);
        ctx.set_bytes("used", used_bytes);
        ctx.set_bytes("u", used_bytes);
        ctx.set_bytes("total", total_bytes);
        ctx.set_bytes("t", total_bytes);
        ctx.set_bytes("free", free_bytes);
        ctx.set_bytes("f", free_bytes);
        ctx.set_num("pct", pct);
        ctx.set_num("p", pct);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Swap", formatted))
    }
}
