use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::template;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Wifi;

impl Module for Wifi {
    fn name(&self) -> &'static str {
        "Wi-Fi"
    }
    fn id(&self) -> &'static str {
        "wifi"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (iface, quality_pct, dbm) = read_wireless_stats()?;
        let ssid = get_ssid(&iface).unwrap_or_else(|| iface.clone());

        let tmpl = config::get()
            .format("wifi")
            .unwrap_or("{ssid} ({signal}% [{dbm} dBm])");

        let mut ctx =
            template::Context::new("wifi").with_metric(quality_pct as u64, 100, quality_pct as f64);
        ctx.set_str("ssid", ssid.clone());
        ctx.set_str("interface", iface.clone());
        ctx.set_str("i", iface);
        ctx.set_num("signal", quality_pct as f64);
        ctx.set_num("pct", quality_pct as f64);
        ctx.set_num("p", quality_pct as f64);
        ctx.set_num("dbm", dbm as f64);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Wi-Fi", formatted))
    }
}

fn read_wireless_stats() -> Option<(String, u32, i32)> {
    let content = fs::read_to_string("/proc/net/wireless").ok()?;
    // Two header lines, then one row per interface. Unparsable rows are skipped.
    for line in content.lines().skip(2) {
        let mut it = line.split_whitespace();
        let (Some(iface), Some(_status), Some(link), Some(level)) =
            (it.next(), it.next(), it.next(), it.next())
        else {
            continue;
        };

        let link = link.trim_end_matches('.').parse::<f64>().unwrap_or(0.0);
        let level = level.trim_end_matches('.').parse::<i32>().unwrap_or(0);
        // Link quality is normally out of 70
        let pct = ((link / 70.0) * 100.0).round().clamp(0.0, 100.0) as u32;

        return Some((iface.trim_end_matches(':').to_string(), pct, level));
    }
    None
}

fn get_ssid(iface: &str) -> Option<String> {
    let cache_file = wifi_cache_path();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    // Check cache (valid for 10 seconds)
    if let Ok(content) = fs::read_to_string(&cache_file) {
        let mut parts = content.splitn(2, ' ');
        if let (Some(ts_str), Some(cached_ssid)) = (parts.next(), parts.next())
            && let Ok(ts) = ts_str.parse::<u64>()
            && now.saturating_sub(ts) < 10
        {
            let s = cached_ssid.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    if let Some(ssid) = get_ssid_ioctl(iface) {
        return Some(ssid);
    }

    let ssid = get_ssid_nmcli()
        .or_else(|| {
            let out = std::process::Command::new("wpa_cli")
                .args(["-i", iface, "status"])
                .output()
                .ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .find(|l| l.starts_with("ssid="))
                .and_then(|l| l.split_once('='))
                .map(|(_, s)| s.trim().to_string())
        })
        .or_else(|| {
            // Last resort: ask iw directly
            let out = std::process::Command::new("iw")
                .arg("dev")
                .arg(iface)
                .arg("link")
                .output()
                .ok()?;
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .find(|l| l.contains("SSID:"))
                .and_then(|l| l.split_once("SSID:"))
                .map(|(_, s)| s.trim().to_string())
        })?;

    let _ = fs::write(cache_file, format!("{now} {ssid}"));
    Some(ssid)
}

fn get_ssid_nmcli() -> Option<String> {
    let out = std::process::Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "active,ssid",
            "dev",
            "wifi",
            "list",
            "--rescan",
            "no",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.starts_with("yes:") {
            let ssid = line.strip_prefix("yes:")?.trim();
            if !ssid.is_empty() {
                return Some(ssid.to_string());
            }
        }
    }
    None
}

fn wifi_cache_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let shm = Path::new("/dev/shm");
    if shm.is_dir() {
        shm.join(format!(".omnifetch_wifi_{uid}"))
    } else {
        std::env::temp_dir().join(format!(".omnifetch_wifi_{uid}"))
    }
}

#[repr(C)]
struct IwPoint {
    pointer: *mut libc::c_char,
    length: u16,
    flags: u16,
}

#[repr(C)]
struct IwReq {
    ifrn_name: [u8; 16],
    point: IwPoint,
    _pad: [u8; 4],
}

const SIOCGIWESSID: libc::c_ulong = 0x8B1B;

fn get_ssid_ioctl(iface: &str) -> Option<String> {
    if iface.len() >= 16 {
        return None;
    }
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if fd < 0 {
        return None;
    }

    let mut buf = [0u8; 33];
    let mut req: IwReq = unsafe { std::mem::zeroed() };
    let bytes = iface.as_bytes();
    req.ifrn_name[..bytes.len()].copy_from_slice(bytes);
    req.point.pointer = buf.as_mut_ptr() as *mut libc::c_char;
    req.point.length = 32;
    req.point.flags = 1;

    let res = unsafe { libc::ioctl(fd, SIOCGIWESSID, &mut req) };
    unsafe { libc::close(fd) };

    if res == 0 {
        let len = (req.point.length as usize).min(32);
        let s = std::str::from_utf8(&buf[..len]).ok()?.trim();
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    None
}
