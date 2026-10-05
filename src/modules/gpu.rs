use crate::cmd;
use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Gpu;

impl Module for Gpu {
    fn name(&self) -> &'static str {
        "GPU"
    }
    fn id(&self) -> &'static str {
        "gpu"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut gpus: Vec<String> = Vec::new();

        let gpus_info = detect_gpus();
        let parsed: Vec<Option<(u16, u16)>> = gpus_info
            .iter()
            .map(|g| {
                Some((
                    u16::from_str_radix(g.vendor_id.trim_start_matches("0x"), 16).ok()?,
                    u16::from_str_radix(g.device_id.trim_start_matches("0x"), 16).ok()?,
                ))
            })
            .collect();

        // Only worth a fork once an NVIDIA card is actually there.
        let mut nvidia_details = Vec::new();
        if parsed
            .iter()
            .flatten()
            .any(|&(vendor, _)| vendor == NVIDIA_VENDOR)
            && let Some(info) = nvidia_smi_details()
        {
            nvidia_details = info;
        }

        let wanted: Vec<(u16, u16)> = parsed.iter().flatten().copied().collect();
        let names = pci_names(&wanted);

        for (i, gpu) in gpus_info.iter().enumerate() {
            let mut name = parsed[i]
                .and_then(|id| wanted.iter().position(|w| *w == id))
                .and_then(|idx| names.get(idx).cloned().flatten())
                .unwrap_or_else(|| {
                    format!(
                        "GPU {}:{}",
                        gpu.vendor_id.trim_start_matches("0x"),
                        gpu.device_id.trim_start_matches("0x")
                    )
                });

            let is_nvidia = parsed[i].map(|(v, _)| v) == Some(NVIDIA_VENDOR);
            if is_nvidia && !nvidia_details.is_empty() {
                if let Some((_n_name, mem)) = nvidia_details.first()
                    && !mem.is_empty()
                {
                    name = format!("{name} ({mem})");
                }
            } else if let Some(vram_bytes) = gpu.vram
                && vram_bytes > 0
            {
                let gib = vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                name = format!("{name} ({gib:.1} GiB)");
            }

            if let Some(ref drv) = gpu.driver {
                name = format!("{name} [{drv}]");
            }

            gpus.push(name);
        }

        let mut seen: Vec<String> = Vec::new();
        for g in gpus {
            if !seen.contains(&g) {
                seen.push(g);
            }
        }

        (!seen.is_empty()).then(|| ModuleOutput::multi("GPU", seen))
    }
}

const NVIDIA_VENDOR: u16 = 0x10de;

struct GpuInfo {
    vendor_id: String,
    device_id: String,
    driver: Option<String>,
    vram: Option<u64>,
}

fn detect_gpus() -> Vec<GpuInfo> {
    let mut v: Vec<GpuInfo> = Vec::new();

    // Read DRM class devices
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for e in entries.flatten() {
            let name = e.file_name();
            let b = name.as_encoded_bytes();
            if !b.starts_with(b"card") || b.contains(&b'-') {
                continue;
            }
            let dev = e.path().join("device");
            let (Some(ven), Some(de)) = (
                sys::read_trim(dev.join("vendor")),
                sys::read_trim(dev.join("device")),
            ) else {
                continue;
            };
            let driver = std::fs::read_link(dev.join("driver"))
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
            let vram =
                sys::read_trim(dev.join("mem_info_vram_total")).and_then(|s| s.parse::<u64>().ok());

            if !v.iter().any(|g| g.vendor_id == ven && g.device_id == de) {
                v.push(GpuInfo {
                    vendor_id: ven,
                    device_id: de,
                    driver,
                    vram,
                });
            }
        }
    }

    // DRM comes up empty on odd hardware. The PCI bus is the second opinion.
    if v.is_empty()
        && let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices")
    {
        for e in entries.flatten() {
            let path = e.path();
            let class_str = sys::read_trim(path.join("class")).unwrap_or_default();
            let is_display = class_str.starts_with("0x0300")
                || class_str.starts_with("0x0301")
                || class_str.starts_with("0x0302")
                || class_str.starts_with("0x0380");
            if !is_display {
                continue;
            }
            let (Some(ven), Some(de)) = (
                sys::read_trim(path.join("vendor")),
                sys::read_trim(path.join("device")),
            ) else {
                continue;
            };
            if !v.iter().any(|g| g.vendor_id == ven && g.device_id == de) {
                let driver = std::fs::read_link(path.join("driver"))
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
                v.push(GpuInfo {
                    vendor_id: ven,
                    device_id: de,
                    driver,
                    vram: None,
                });
            }
        }
    }

    v
}

fn nvidia_smi_details() -> Option<Vec<(String, String)>> {
    let raw = cmd::run(
        "nvidia-smi",
        &[
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ],
    )?;
    let mut v = Vec::new();
    for line in raw.lines() {
        let mut it = line.split(',');
        let name = it.next()?.trim();
        let mem = it.next().and_then(|s| s.trim().parse::<u64>().ok());
        let mem_str = match mem {
            Some(m) if m >= 1024 => format!("{:.1} GiB", m as f64 / 1024.0),
            Some(m) => format!("{m} MiB"),
            None => String::new(),
        };
        v.push((name.to_string(), mem_str));
    }
    (!v.is_empty()).then_some(v)
}

#[inline]
fn fast_hex(s: &[u8], digits: usize) -> Option<u32> {
    if s.len() < digits {
        return None;
    }
    let mut v = 0u32;
    for &c in &s[..digits] {
        let d = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => return None,
        };
        v = v * 16 + d as u32;
    }
    Some(v)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn format_hex4(v: u16) -> [u8; 4] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    [
        HEX[((v >> 12) & 0xf) as usize],
        HEX[((v >> 8) & 0xf) as usize],
        HEX[((v >> 4) & 0xf) as usize],
        HEX[(v & 0xf) as usize],
    ]
}

pub struct PciFile {
    mmap: Option<(*mut libc::c_void, usize)>,
    bytes: Vec<u8>,
}

impl Drop for PciFile {
    fn drop(&mut self) {
        if let Some((ptr, len)) = self.mmap {
            unsafe { libc::munmap(ptr, len) };
        }
    }
}

impl PciFile {
    pub fn load() -> Option<Self> {
        let path = ["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"]
            .into_iter()
            .find(|p| std::path::Path::new(p).exists())?;
        let cpath = std::ffi::CString::new(path).ok()?;
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY) };
        if fd < 0 {
            return None;
        }
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut st) } != 0 || st.st_size <= 0 {
            unsafe { libc::close(fd) };
            return None;
        }
        let len = st.st_size as usize;
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        unsafe { libc::close(fd) };
        if ptr == libc::MAP_FAILED {
            return None;
        }
        Some(PciFile {
            mmap: Some((ptr, len)),
            bytes: Vec::new(),
        })
    }

    fn as_bytes(&self) -> &[u8] {
        if let Some((ptr, len)) = self.mmap {
            unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
        } else {
            &self.bytes
        }
    }

    pub fn resolve(&self, devices: &[(u16, u16)]) -> Vec<Option<String>> {
        let bytes = self.as_bytes();
        let mut results = Vec::with_capacity(devices.len());

        for &(vendor, device) in devices {
            let mut resolved = None;
            let mut vendor_name = None;

            // Search for vendor header: "\n{vendor:04x}  " or at file start "{vendor:04x}  "
            let hex = format_hex4(vendor);
            let pat = [b'\n', hex[0], hex[1], hex[2], hex[3], b' ', b' '];

            let header_pos = if bytes.starts_with(&pat[1..]) {
                Some(0)
            } else {
                find_subslice(bytes, &pat).map(|i| i + 1)
            };

            if let Some(pos) = header_pos {
                // Find end of vendor header line
                let after_id = pos + 6;
                let line_end = bytes[after_id..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .map(|i| after_id + i)
                    .unwrap_or(bytes.len());
                let vname = std::str::from_utf8(&bytes[after_id..line_end])
                    .ok()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                vendor_name = vname;

                // Section starts at line_end + 1
                let mut cur = if line_end < bytes.len() {
                    line_end + 1
                } else {
                    bytes.len()
                };
                while cur < bytes.len() {
                    let next_nl = bytes[cur..]
                        .iter()
                        .position(|&c| c == b'\n')
                        .map(|i| cur + i)
                        .unwrap_or(bytes.len());
                    let line = &bytes[cur..next_nl];
                    cur = if next_nl < bytes.len() {
                        next_nl + 1
                    } else {
                        bytes.len()
                    };

                    if line.is_empty() || line[0] == b'#' {
                        continue;
                    }
                    if line[0] != b'\t' {
                        // End of vendor section
                        break;
                    }
                    // Vendor devices are indent 1: line starts with '\t' but not '\t\t'
                    if line.get(1) == Some(&b'\t') {
                        continue;
                    }
                    if let Some(dev) = fast_hex(&line[1..], 4)
                        && dev as u16 == device
                        && let Some(dname) = line_name(line, 5)
                    {
                        resolved = Some(dname);
                        break;
                    }
                }
            }

            // Vendor lookup failed, so try the display class section directly.
            if resolved.is_none() {
                let class_pat = b"\nC 03  ";
                let class_pos = if bytes.starts_with(&class_pat[1..]) {
                    Some(0)
                } else {
                    find_subslice(bytes, class_pat).map(|i| i + 1)
                };

                if let Some(pos) = class_pos {
                    let line_end = bytes[pos..]
                        .iter()
                        .position(|&c| c == b'\n')
                        .map(|i| pos + i)
                        .unwrap_or(bytes.len());
                    let mut cur = if line_end < bytes.len() {
                        line_end + 1
                    } else {
                        bytes.len()
                    };
                    while cur < bytes.len() {
                        let next_nl = bytes[cur..]
                            .iter()
                            .position(|&c| c == b'\n')
                            .map(|i| cur + i)
                            .unwrap_or(bytes.len());
                        let line = &bytes[cur..next_nl];
                        cur = if next_nl < bytes.len() {
                            next_nl + 1
                        } else {
                            bytes.len()
                        };

                        if line.is_empty() || line[0] == b'#' {
                            continue;
                        }
                        if line[0] != b'\t' {
                            // End of C 03 section
                            break;
                        }
                        if line.get(1) == Some(&b'\t')
                            && let Some(dev) = fast_hex(&line[2..], 4)
                            && dev as u16 == device
                            && let Some(dname) = line_name(line, 6)
                        {
                            resolved = Some(dname);
                            break;
                        }
                    }
                }
            }

            let full = resolved.map(|dname| {
                if let Some(vname) = vendor_name
                    && !vname.is_empty()
                    && !dname.starts_with(&vname)
                {
                    format!("{vname} {dname}")
                } else {
                    dname
                }
            });
            results.push(full);
        }

        results
    }

    #[cfg(test)]
    pub fn resolve_lines<'a, I>(&self, devices: &[(u16, u16)], lines: I) -> Vec<Option<String>>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        let mut joined = Vec::new();
        for l in lines {
            joined.extend_from_slice(l);
            joined.push(b'\n');
        }
        let temp = PciFile {
            mmap: None,
            bytes: joined,
        };
        temp.resolve(devices)
    }
}

/// Resolves each (vendor, device) pair against the system pci.ids, caching results.
pub(crate) fn pci_names(devices: &[(u16, u16)]) -> Vec<Option<String>> {
    let mut results = vec![None; devices.len()];
    let mut missing = Vec::new();
    let mut missing_indices = Vec::new();

    for (i, &(v, d)) in devices.iter().enumerate() {
        let key = format!("pci_{v:04x}_{d:04x}");
        if let Some(cached) = sys::cache_get(&key) {
            results[i] = Some(cached);
        } else {
            missing.push((v, d));
            missing_indices.push(i);
        }
    }

    if missing.is_empty() {
        return results;
    }

    let Some(file) = PciFile::load() else {
        return results;
    };
    let resolved = file.resolve(&missing);
    for (idx, res) in missing_indices.into_iter().zip(resolved) {
        if let Some(ref name) = res {
            let (v, d) = devices[idx];
            let key = format!("pci_{v:04x}_{d:04x}");
            sys::cache_set(&key, name);
        }
        results[idx] = res;
    }
    results
}

fn line_name(line: &[u8], from: usize) -> Option<String> {
    let mut i = from.min(line.len());
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    if i >= line.len() {
        return None;
    }
    let rest = &line[i..];
    let len = rest.iter().position(|&c| c == b'\t').unwrap_or(rest.len());
    if len == 0 {
        return None;
    }
    std::str::from_utf8(&rest[..len])
        .ok()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const FIXTURE: &str = "\
# comment
0001  Vendor Zero
\t0001  Zero Chip
\t\t1af4 1000  Zero Subsystem
0010  Vendor One
\t0002  One Chip A
\t0003  One Chip B
C 01  Mass storage controller
\t00  SCSI storage controller
\t\t0110  One Chip A
C 03  Display controller
\t00  VGA compatible controller
\t\t0003  Class Shadow Name
\t\t0166  Ivy Bridge mobile GT2 [HD Graphics 4000]
ffff  Vendor Last
\tffff  Last Chip
";

    fn resolve(pairs: &[(u16, u16)]) -> Vec<Option<String>> {
        PciFile {
            mmap: None,
            bytes: Vec::new(),
        }
        .resolve_lines(pairs, FIXTURE.as_bytes().split(|&c| c == b'\n'))
        .into_iter()
        .zip(pairs)
        .map(|(r, _)| r)
        .collect()
    }

    #[test]
    pub fn resolves_vendor_section_entry() {
        assert_eq!(
            resolve(&[(0x0001, 0x0001)])[0].as_deref(),
            Some("Vendor Zero Zero Chip")
        );
    }

    #[test]
    pub fn resolves_last_section_in_file() {
        assert_eq!(
            resolve(&[(0xffff, 0xffff)])[0].as_deref(),
            Some("Vendor Last Last Chip")
        );
    }

    #[test]
    pub fn resolves_class_section_fallback() {
        assert_eq!(
            resolve(&[(0xabcd, 0x0166)])[0].as_deref(),
            Some("Ivy Bridge mobile GT2 [HD Graphics 4000]")
        );
    }

    #[test]
    pub fn vendor_entry_wins_over_class_entry() {
        assert_eq!(
            resolve(&[(0x0010, 0x0003)])[0].as_deref(),
            Some("Vendor One One Chip B")
        );
    }

    #[test]
    pub fn subsystem_lines_do_not_match_vendor_devices() {
        assert_eq!(resolve(&[(0x0001, 0x1af4)])[0], None);
    }

    #[test]
    pub fn non_display_class_is_ignored() {
        assert_eq!(resolve(&[(0xabcd, 0x0110)])[0], None);
    }

    #[test]
    pub fn unknown_ids_resolve_to_none() {
        let got = resolve(&[(0x0001, 0x9999), (0x4242, 0x4242)]);
        assert_eq!(got[0], None);
        assert_eq!(got[1], None);
    }

    #[test]
    pub fn multiple_devices_resolve_in_one_pass() {
        let got = resolve(&[(0x0001, 0x0001), (0xabcd, 0x0166), (0x0001, 0x1af4)]);
        assert_eq!(got[0].as_deref(), Some("Vendor Zero Zero Chip"));
        assert_eq!(
            got[1].as_deref(),
            Some("Ivy Bridge mobile GT2 [HD Graphics 4000]")
        );
        assert_eq!(got[2], None);
    }

    #[test]
    pub fn resolves_against_system_pci_ids() {
        if !["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"]
            .iter()
            .any(|p| std::path::Path::new(p).exists())
        {
            return;
        }
        let f = PciFile::load().unwrap();
        let got = f.resolve(&[(0x8086, 0x0166)]);
        let name = got[0].as_deref().unwrap_or_default();
        assert!(
            name.contains("Graphics")
                || name.contains("Ivy Bridge")
                || name.contains("3rd Gen Core"),
            "unexpected name for 8086:0166: {name:?}"
        );
        assert!(
            name.starts_with("Intel Corporation"),
            "missing vendor: {name:?}"
        );

        let many: Vec<(u16, u16)> = [(0x10de, 0x2684), (0x1002, 0x73df), (0x1234, 0x9999)]
            .into_iter()
            .collect();
        let got = f.resolve(&many);
        assert!(got[0].is_some(), "NVIDIA 2684 should resolve");
        assert!(got[1].is_some(), "AMD 73df should resolve");
        assert!(got[2].is_none(), "bogus id should not resolve");
    }
}
