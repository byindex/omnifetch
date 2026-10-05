use crate::cache::StaticCache;
use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;
use std::fs;

pub struct CpuTemp;

impl Module for CpuTemp {
    fn name(&self) -> &'static str {
        "CPU Temp"
    }
    fn id(&self) -> &'static str {
        "cputemp"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let temp_c = read_cpu_temp()?;
        let temp_f = temp_c * 9.0 / 5.0 + 32.0;

        let tmpl = config::get().format("cputemp").unwrap_or("{temp}");

        let mut ctx = template::Context::new("cputemp");
        ctx.set_str("temp", format!("{temp_c:.0}\u{00b0}C"));
        ctx.set_str("t", format!("{temp_c:.0}\u{00b0}C"));
        ctx.set_num("celsius", temp_c);
        ctx.set_num("c", temp_c);
        ctx.set_num("fahrenheit", temp_f);
        ctx.set_num("f", temp_f);

        let color = if temp_c < 60.0 {
            "\x1b[32m"
        } else if temp_c < 80.0 {
            "\x1b[33m"
        } else {
            "\x1b[31m"
        };
        ctx.set_str("color", color);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("CPU Temp", formatted))
    }
}

pub fn detect_cpu_temp_and_path() -> Option<(String, f64)> {
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        let mut fallback = None;
        for e in entries.flatten() {
            let path = e.path();
            let name = sys::read_trim(path.join("name")).unwrap_or_default();
            let is_cpu = matches!(
                name.as_str(),
                "coretemp" | "k10temp" | "zenpower" | "cpu_thermal"
            );

            if is_cpu {
                for i in 1..=4 {
                    let temp_file = path.join(format!("temp{i}_input"));
                    if let Some(millidegrees) = sys::read_num(&temp_file) {
                        let deg = millidegrees / 1000.0;
                        if deg > 0.0 && deg < 150.0 {
                            return Some((temp_file.to_string_lossy().to_string(), deg));
                        }
                    }
                }
            } else if fallback.is_none() && name != "dell_smm" && !name.is_empty() {
                let temp_file = path.join("temp1_input");
                if let Some(millidegrees) = sys::read_num(&temp_file) {
                    let deg = millidegrees / 1000.0;
                    if deg > 0.0 && deg < 150.0 {
                        fallback = Some((temp_file.to_string_lossy().to_string(), deg));
                    }
                }
            }
        }
        if let Some(p) = fallback {
            return Some(p);
        }
    }

    for i in 0..=3 {
        let path = format!("/sys/class/thermal/thermal_zone{i}/temp");
        if let Some(millidegrees) = sys::read_num(&path) {
            let deg = millidegrees / 1000.0;
            if deg > 0.0 && deg < 150.0 {
                return Some((path, deg));
            }
        }
    }

    None
}

pub fn detect_cpu_temp_path() -> Option<String> {
    detect_cpu_temp_and_path().map(|(p, _)| p)
}

fn read_cpu_temp() -> Option<f64> {
    if let Some(p) = StaticCache::get_cputemp_path()
        && let Some(millidegrees) = sys::read_num(&p)
    {
        let deg = millidegrees / 1000.0;
        if deg > 0.0 && deg < 150.0 {
            return Some(deg);
        }
    }

    detect_cpu_temp_and_path().map(|(_, deg)| deg)
}
