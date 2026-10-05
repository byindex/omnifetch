use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct PowerAdapter;

impl Module for PowerAdapter {
    fn name(&self) -> &'static str {
        "PowerAdapter"
    }

    fn id(&self) -> &'static str {
        "poweradapter"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let entries = fs::read_dir("/sys/class/power_supply").ok()?;
        let mut adapters = Vec::new();
        let mut path = [0u8; 96];
        let mut buf = [0u8; 128];

        for entry in entries.flatten() {
            // Attribute paths are rebuilt on the stack. A PathBuf each would be pure overhead.
            let Some(dev) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            // `type` decides whether this is a mains adapter, so it is read first.
            let ptype = read_attr(&dev, "type", &mut path, &mut buf).unwrap_or_default();
            if ptype != "Mains" && ptype != "AC" && ptype != "USB" {
                continue;
            }

            let online = read_attr(&dev, "online", &mut path, &mut buf).is_some_and(|s| s == "1");

            let status = if online { "Connected" } else { "Disconnected" };

            let label = read_attr(&dev, "model_name", &mut path, &mut buf)
                .or_else(|| read_attr(&dev, "manufacturer", &mut path, &mut buf))
                .unwrap_or_else(|| dev.clone());

            let power_w = read_num_attr(&dev, "power_now", &mut path, &mut buf)
                .map(|u| (u / 1_000_000.0).round() as u64)
                .or_else(|| {
                    let v = read_num_attr(&dev, "voltage_now", &mut path, &mut buf)?;
                    let c = read_num_attr(&dev, "current_now", &mut path, &mut buf)?;
                    Some(((v * c) / 1_000_000_000_000.0).round() as u64)
                });

            if let Some(w) = power_w
                && w > 0
            {
                adapters.push(format!("{label}: {w}W [{status}]"));
                continue;
            }

            adapters.push(format!("{label} [{status}]"));
        }

        if adapters.is_empty() {
            return None;
        }

        Some(ModuleOutput::new("PowerAdapter", adapters.join(", ")))
    }
}

/// Builds `/sys/class/power_supply/<dev>/<attr>` on the stack, NUL terminated.
fn attr_path(dev: &str, out: &mut [u8; 96]) -> Option<Vec<u8>> {
    let base = b"/sys/class/power_supply/";
    let end = base.len() + dev.len();
    if end >= out.len() {
        return None;
    }
    out[..base.len()].copy_from_slice(base);
    out[base.len()..end].copy_from_slice(dev.as_bytes());
    Some(out[..end].to_vec())
}

/// Reads a `/sys/class/power_supply/<dev>/<attr>` value as trimmed text.
fn read_attr(dev: &str, attr: &str, path: &mut [u8; 96], buf: &mut [u8; 128]) -> Option<String> {
    let mut p = attr_path(dev, path)?;
    p.push(b'/');
    p.extend_from_slice(attr.as_bytes());
    p.push(0);
    sys::read_small(&p, buf).map(|s| s.trim().to_string())
}

/// Reads a `/sys/class/power_supply/<dev>/<attr>` value as a number.
fn read_num_attr(dev: &str, attr: &str, path: &mut [u8; 96], buf: &mut [u8; 128]) -> Option<f64> {
    let text = read_attr(dev, attr, path, buf)?;
    text.parse::<f64>().ok()
}
