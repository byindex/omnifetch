use crate::cmd;
use crate::module::{Module, ModuleOutput};

pub struct OpenGl;

impl Module for OpenGl {
    fn name(&self) -> &'static str {
        "OpenGL"
    }
    fn id(&self) -> &'static str {
        "opengl"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if let Some(v) = detect_fast_opengl() {
            return Some(ModuleOutput::new("OpenGL", v));
        }

        let out = cmd::run("glxinfo", &["-B"])?;
        for line in out.lines() {
            let trimmed = line.trim();
            if let Some((_, ver)) = trimmed.split_once("OpenGL version string:") {
                let v = ver.trim().to_string();
                if !v.is_empty() {
                    return Some(ModuleOutput::new("OpenGL", v));
                }
            }
        }
        None
    }
}

fn detect_fast_opengl() -> Option<String> {
    // 1. Check NVIDIA driver
    if let Ok(content) = std::fs::read_to_string("/proc/driver/nvidia/version") {
        for line in content.lines() {
            if let Some(pos) = line.find("Kernel Module") {
                let ver = line[pos + 13..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim();
                if !ver.is_empty() {
                    return Some(format!("4.6.0 NVIDIA {ver}"));
                }
            }
        }
    }

    // 2. Check Mesa version via /usr/lib/dri/ first (small directory, fast!)
    let mut mesa_ver = None;
    for dri_dir in [
        "/usr/lib/dri",
        "/usr/lib64/dri",
        "/usr/lib/x86_64-linux-gnu/dri",
    ] {
        let Ok(entries) = std::fs::read_dir(dri_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if let Ok(target) = std::fs::read_link(entry.path()) {
                let ts = target.to_string_lossy();
                if let Some(pos) = ts.find("libgallium-") {
                    let rest = &ts[pos + 11..];
                    if let Some(v) = rest.strip_suffix(".so") {
                        mesa_ver = Some(v.to_string());
                        break;
                    }
                }
            }
        }
        if mesa_ver.is_some() {
            break;
        }
    }

    if mesa_ver.is_none() {
        for lib_dir in ["/usr/lib", "/usr/lib64", "/usr/lib/x86_64-linux-gnu"] {
            let Ok(entries) = std::fs::read_dir(lib_dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if let Some(rest) = s.strip_prefix("libgallium-")
                    && let Some(v) = rest.strip_suffix(".so")
                {
                    mesa_ver = Some(v.to_string());
                    break;
                }
            }
            if mesa_ver.is_some() {
                break;
            }
        }
    }

    let mesa_ver = mesa_ver?;

    // Check GPU PCI ID to determine GL max version (4.2 for Ivy Bridge, 4.6 for all modern)
    let is_ivy_bridge = if let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") {
        entries.flatten().any(|e| {
            let path = e.path();
            let vendor = std::fs::read_to_string(path.join("vendor")).unwrap_or_default();
            let device = std::fs::read_to_string(path.join("device")).unwrap_or_default();
            vendor.trim() == "0x8086"
                && matches!(
                    device.trim(),
                    "0x0152" | "0x0156" | "0x0162" | "0x0166" | "0x015a"
                )
        })
    } else {
        false
    };

    let gl_ver = if is_ivy_bridge { "4.2" } else { "4.6" };
    Some(format!("{gl_ver} (Compatibility Profile) Mesa {mesa_ver}"))
}
