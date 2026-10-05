use std::fs;

use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Resolution;

impl Module for Resolution {
    fn name(&self) -> &'static str {
        "Resolution"
    }
    fn id(&self) -> &'static str {
        "resolution"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if let Some(out) = from_drm() {
            return Some(out);
        }
        // Not every setup exposes DRM connectors.
        from_fb()
    }
}

/// Reads every connected connector's mode from sysfs, which works on Wayland where `xrandr` does not.
fn from_drm() -> Option<ModuleOutput> {
    let entries = fs::read_dir("/sys/class/drm").ok()?;
    let mut resolutions = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        // Connector dirs look like card0-DP-1. Plain cardN entries are something else.
        let Some((_, connector)) = name.split_once('-') else {
            continue;
        };
        if connector.is_empty() || connector.starts_with("card") {
            continue;
        }
        if sys::read_trim(path.join("status")).as_deref() != Some("connected") {
            continue;
        }
        let Some(mode) =
            sys::read_trim(path.join("modes")).and_then(|m| m.lines().next().map(str::to_string))
        else {
            continue;
        };
        resolutions.push(format!("{connector} ({mode})"));
    }

    if resolutions.is_empty() {
        None
    } else {
        Some(ModuleOutput::multi("Resolution", resolutions))
    }
}

/// Last resort: the framebuffer's virtual size, e.g. `1366,768`.
fn from_fb() -> Option<ModuleOutput> {
    let mut out = Vec::new();
    for entry in fs::read_dir("/sys/class/graphics").ok()?.flatten() {
        let path = entry.path();
        if let Some(size) = sys::read_trim(path.join("virtual_size")) {
            let res = size.replace(',', "x");
            out.push(format!("{} ({res})", entry.file_name().to_string_lossy()));
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(ModuleOutput::multi("Resolution", out))
    }
}
