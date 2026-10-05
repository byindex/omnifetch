use crate::cache::StaticCache;
use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;

pub struct Battery;

impl Module for Battery {
    fn name(&self) -> &'static str {
        "Battery"
    }
    fn id(&self) -> &'static str {
        "battery"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if let Some(cache) = StaticCache::get()
            && cache.has_battery == Some(false)
        {
            return None;
        }

        let mut out = None;
        let p_supp = std::path::Path::new("/sys/class/power_supply");
        let dirs: Vec<std::path::PathBuf> = ["BAT0", "BAT1", "BAT"]
            .iter()
            .map(|name| p_supp.join(name))
            .filter(|p| p.exists())
            .collect();

        let candidates = if !dirs.is_empty() {
            dirs
        } else {
            let entries = std::fs::read_dir(p_supp).ok()?;
            entries.flatten().map(|e| e.path()).collect()
        };

        for dir in candidates {
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name == "AC" || name.starts_with("hid-") {
                continue;
            }
            if sys::read_trim(dir.join("type")).as_deref() != Some("Battery") {
                continue;
            }
            let cap = sys::read_num(dir.join("capacity"))?;
            let status = sys::read_trim(dir.join("status")).unwrap_or_default();
            let charged = sys::read_num(dir.join("energy_now"))
                .zip(sys::read_num(dir.join("energy_full")))
                .or_else(|| {
                    sys::read_num(dir.join("charge_now"))
                        .zip(sys::read_num(dir.join("charge_full")))
                });

            let tmpl = config::get()
                .format("battery")
                .unwrap_or("{bar} {pct}%{status}{wh}");

            let mut ctx = template::Context::new("battery").with_metric(cap as u64, 100, cap);
            ctx.set_num("pct", cap);
            ctx.set_num("p", cap);
            ctx.set_str(
                "status",
                if status == "Charging" {
                    " (charging)"
                } else if status == "Discharging" {
                    " (discharging)"
                } else {
                    ""
                },
            );
            ctx.set_str("s", &status);
            ctx.set_str("raw_status", status.to_ascii_lowercase());
            ctx.set_str("charging", if status == "Charging" { "true" } else { "" });
            ctx.set_str(
                "discharging",
                if status == "Discharging" { "true" } else { "" },
            );
            if let Some((now, full)) = charged
                && full > 0.0
            {
                ctx.set_str(
                    "wh",
                    format!(" [{:.0}/{:.0} Wh]", now / 1000.0, full / 1000.0),
                );
                ctx.set_num("energy_now", now / 1000.0);
                ctx.set_num("energy_full", full / 1000.0);
            } else {
                ctx.set_str("wh", "");
            }

            let formatted = template::render_template(tmpl, &ctx);
            out = Some(ModuleOutput::new("Battery", formatted));
            break;
        }
        out
    }
}
