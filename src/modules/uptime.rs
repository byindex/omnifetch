use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;

pub struct Uptime;

impl Module for Uptime {
    fn name(&self) -> &'static str {
        "Uptime"
    }
    fn id(&self) -> &'static str {
        "uptime"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let total_secs = {
            let mut info = std::mem::MaybeUninit::<libc::sysinfo>::uninit();
            if unsafe { libc::sysinfo(info.as_mut_ptr()) } == 0 {
                unsafe { info.assume_init().uptime as u64 }
            } else {
                sys::read_num("/proc/uptime")? as u64
            }
        };
        let human = sys::human_duration(std::time::Duration::from_secs(total_secs));

        let tmpl = config::get().format("uptime").unwrap_or("{uptime}");
        if tmpl == "{uptime}" {
            return Some(ModuleOutput::new("Uptime", human));
        }

        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        let mut ctx = template::Context::new("uptime");
        ctx.set_str("uptime", &human);
        ctx.set_num("days", days as f64);
        ctx.set_num("d", days as f64);
        ctx.set_num("hours", hours as f64);
        ctx.set_num("h", hours as f64);
        ctx.set_num("mins", mins as f64);
        ctx.set_num("m", mins as f64);
        ctx.set_num("secs", secs as f64);
        ctx.set_num("s", secs as f64);
        ctx.set_num("total_secs", total_secs as f64);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Uptime", formatted))
    }
}
