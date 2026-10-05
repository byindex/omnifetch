use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::path::Path;

pub struct Codec;

impl Module for Codec {
    fn name(&self) -> &'static str {
        "Codec"
    }

    fn id(&self) -> &'static str {
        "codec"
    }

    fn run(&self) -> Option<ModuleOutput> {
        if !Path::new("/dev/dri/renderD128").exists() {
            return None;
        }

        // Determine driver to report codec profile
        let driver = std::fs::read_link("/sys/class/drm/card0/device/driver")
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_default();

        let vendor = sys::read_trim("/sys/class/drm/card0/device/vendor").unwrap_or_default();
        let vendor_id = u16::from_str_radix(vendor.trim_start_matches("0x"), 16).unwrap_or(0);

        let codecs = match vendor_id {
            0x8086 => "H.264, HEVC, VP9, AV1 (VA-API / QSV)",
            0x1002 => "H.264, HEVC, AV1 (VA-API / AMF)",
            0x10de => "H.264, HEVC, VP9, AV1 (NVDEC / NVENC)",
            _ => {
                if !driver.is_empty() {
                    return Some(ModuleOutput::new(
                        "Codec",
                        format!("Hardware Accelerated [{driver}]"),
                    ));
                }
                "Hardware Accelerated (VA-API)"
            }
        };

        Some(ModuleOutput::new("Codec", codecs))
    }
}
