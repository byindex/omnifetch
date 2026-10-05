use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CpuUsage;

impl Module for CpuUsage {
    fn name(&self) -> &'static str {
        "CPU Usage"
    }
    fn id(&self) -> &'static str {
        "cpuusage"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (total, idle) = read_cpu_ticks()?;
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_millis() as u64;

        let pct = calculate_pct(total, idle, now_ms);

        let tmpl = config::get().format("cpuusage").unwrap_or("{pct}% {bar}");

        let mut ctx = template::Context::new("cpuusage").with_metric(pct.round() as u64, 100, pct);
        ctx.set_num("pct", pct);
        ctx.set_num("p", pct);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("CPU Usage", formatted))
    }
}

fn stat_file_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let shm = Path::new("/dev/shm");
    if shm.is_dir() {
        shm.join(format!(".omnifetch_cpustat_{uid}"))
    } else {
        std::env::temp_dir().join(format!(".omnifetch_cpustat_{uid}"))
    }
}

fn read_cpu_ticks() -> Option<(u64, u64)> {
    let mut buf = [0u8; 256];
    let n = sys::read_bytes_into(std::path::Path::new("/proc/stat"), &mut buf)?;
    let text = std::str::from_utf8(&buf[..n]).ok()?;
    let line = text.lines().next()?;
    if !line.starts_with("cpu ") {
        return None;
    }
    let mut nums = line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse::<u64>().ok());

    let user = nums.next()?;
    let nice = nums.next()?;
    let system = nums.next()?;
    let idle = nums.next()?;
    let iowait = nums.next().unwrap_or(0);
    let irq = nums.next().unwrap_or(0);
    let softirq = nums.next().unwrap_or(0);
    let steal = nums.next().unwrap_or(0);

    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    let idle_all = idle + iowait;

    Some((total, idle_all))
}

fn calculate_pct(total: u64, idle: u64, now_ms: u64) -> f64 {
    let path = stat_file_path();
    let mut pct = if total > 0 {
        ((total.saturating_sub(idle)) as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    if let Ok(prev_str) = fs::read_to_string(&path) {
        let mut it = prev_str
            .split_whitespace()
            .filter_map(|s| s.parse::<u64>().ok());
        if let (Some(prev_total), Some(prev_idle), Some(prev_time)) =
            (it.next(), it.next(), it.next())
            && now_ms > prev_time
            && (now_ms - prev_time) < 60_000
            && total > prev_total
        {
            let d_total = total - prev_total;
            let d_idle = idle.saturating_sub(prev_idle);
            if d_total > 0 {
                pct = ((d_total.saturating_sub(d_idle)) as f64 / d_total as f64) * 100.0;
            }
        }
    }

    let _ = fs::write(path, format!("{total} {idle} {now_ms}"));
    pct.clamp(0.0, 100.0)
}
