use std::fs;

use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Display;

impl Module for Display {
    fn name(&self) -> &'static str {
        "Display"
    }
    fn id(&self) -> &'static str {
        "display"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut displays = Vec::new();
        let entries = fs::read_dir("/sys/class/drm").ok()?;
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            // Match connector dirs like card1-LVDS-1, card0-DP-1
            let Some((_, conn)) = name.split_once('-') else {
                continue;
            };
            if conn.is_empty() || conn.starts_with("card") {
                continue;
            }
            if sys::read_trim(path.join("status")).as_deref() != Some("connected") {
                continue;
            }
            let mode = sys::read_trim(path.join("modes"))
                .and_then(|m| m.lines().next().map(str::to_string))
                .unwrap_or_else(|| "connected".into());

            let rate = if let Ok(edid_bytes) = fs::read(path.join("edid")) {
                parse_edid_refresh_rate(&edid_bytes, &mode)
            } else {
                None
            };

            let mode_str = if let Some(hz) = rate {
                format!("{mode} @ {hz}Hz")
            } else {
                mode
            };

            displays.push(format!("{conn} ({mode_str})"));
        }

        if displays.is_empty() {
            return None;
        }

        Some(ModuleOutput::multi("Display", displays))
    }
}

pub fn parse_edid_refresh_rate(edid: &[u8], target_mode: &str) -> Option<u64> {
    if edid.len() < 128 {
        return None;
    }
    // Check standard EDID header: 00 FF FF FF FF FF FF 00
    if edid[..8] != [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00] {
        return None;
    }

    let mut first_valid_hz = None;

    // 4 Detailed Timing Descriptors at offsets 54, 72, 90, 108 (each 18 bytes)
    for &offset in &[54, 72, 90, 108] {
        if offset + 18 > edid.len() {
            break;
        }
        let block = &edid[offset..offset + 18];
        let pixel_clock_10k = u16::from_le_bytes([block[0], block[1]]) as u64;
        if pixel_clock_10k == 0 {
            // Display descriptor, not a timing descriptor
            continue;
        }
        let pixel_clock_hz = pixel_clock_10k * 10_000;

        let h_active = (block[2] as u16 | (((block[4] as u16) & 0xf0) << 4)) as u64;
        let h_blank = (block[3] as u16 | (((block[4] as u16) & 0x0f) << 8)) as u64;
        let h_total = h_active + h_blank;

        let v_active = (block[5] as u16 | (((block[7] as u16) & 0xf0) << 4)) as u64;
        let v_blank = (block[6] as u16 | (((block[7] as u16) & 0x0f) << 8)) as u64;
        let v_total = v_active + v_blank;

        if h_total > 0 && v_total > 0 {
            let hz = ((pixel_clock_hz as f64) / ((h_total * v_total) as f64)).round() as u64;
            if hz > 0 {
                let expected_prefix = format!("{h_active}x{v_active}");
                if target_mode.starts_with(&expected_prefix) {
                    return Some(hz);
                }
                if first_valid_hz.is_none() {
                    first_valid_hz = Some(hz);
                }
            }
        }
    }

    first_valid_hz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_edid_refresh_rate() {
        let mut edid = vec![0u8; 128];
        // Standard header
        edid[..8].copy_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);

        // DTD at offset 54: 1920x1080 @ 60Hz
        // Pixel clock: 148.50 MHz = 14850 * 10 kHz = 0x3A02
        let offset = 54;
        edid[offset..offset + 2].copy_from_slice(&14850u16.to_le_bytes());
        // h_active = 1920 (0x780), h_blank = 280 (0x118) -> h_total = 2200
        edid[offset + 2] = 0x80; // lower 8 bits of 1920
        edid[offset + 3] = 0x18; // lower 8 bits of 280
        edid[offset + 4] = (0x07 << 4) | 0x01; // upper 4 bits: 0x7 for h_active, 0x1 for h_blank

        // v_active = 1080 (0x438), v_blank = 45 (0x02D) -> v_total = 1125
        edid[offset + 5] = 0x38; // lower 8 bits of 1080
        edid[offset + 6] = 0x2D; // lower 8 bits of 45
        edid[offset + 7] = (0x04 << 4) | 0x00; // upper 4 bits: 0x4 for v_active, 0x0 for v_blank
        // Total = 2200 * 1125 = 2,475,000. 148,500,000 / 2,475,000 = 60.00 Hz!

        let hz = parse_edid_refresh_rate(&edid, "1920x1080");
        assert_eq!(hz, Some(60));
    }
}
