use crate::module::{Module, ModuleOutput};
use crate::modules::gpu;
use std::fs;

pub struct NetAdapter;

impl Module for NetAdapter {
    fn name(&self) -> &'static str {
        "NetAdapter"
    }
    fn id(&self) -> &'static str {
        "netadapter"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut adapters = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = fs::read_dir("/sys/class/net") {
                let mut iface_devices = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name == "lo"
                        || name.starts_with("docker")
                        || name.starts_with("veth")
                        || name.starts_with("br-")
                        || name.starts_with("virbr")
                    {
                        continue;
                    }
                    let dev_path = entry.path().join("device");
                    if !dev_path.exists() {
                        continue;
                    }
                    let vendor_str =
                        fs::read_to_string(dev_path.join("vendor")).unwrap_or_default();
                    let device_str =
                        fs::read_to_string(dev_path.join("device")).unwrap_or_default();
                    let v =
                        u16::from_str_radix(vendor_str.trim().trim_start_matches("0x"), 16).ok();
                    let d =
                        u16::from_str_radix(device_str.trim().trim_start_matches("0x"), 16).ok();

                    let driver = fs::read_link(dev_path.join("driver"))
                        .ok()
                        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                        .unwrap_or_default();

                    if let (Some(v), Some(d)) = (v, d) {
                        iface_devices.push((name, v, d, driver));
                    }
                }

                let pci_pairs: Vec<(u16, u16)> =
                    iface_devices.iter().map(|(_, v, d, _)| (*v, *d)).collect();
                let names = gpu::pci_names(&pci_pairs);

                for (idx, (iface, v, d, driver)) in iface_devices.into_iter().enumerate() {
                    let desc = names
                        .get(idx)
                        .cloned()
                        .flatten()
                        .unwrap_or_else(|| format!("{v:04x}:{d:04x}"));
                    let clean_desc = desc
                        .replace("Corporation ", "")
                        .replace("Co., Ltd. ", "")
                        .replace("Semiconductor ", "");
                    let driver_suffix = if !driver.is_empty() {
                        format!(" [{driver}]")
                    } else {
                        String::new()
                    };
                    adapters.push(format!("{iface}: {clean_desc}{driver_suffix}"));
                }
            }
        }

        if adapters.is_empty() {
            None
        } else if adapters.len() == 1 {
            Some(ModuleOutput::new("NetAdapter", &adapters[0]))
        } else {
            Some(ModuleOutput::multi("NetAdapter", adapters))
        }
    }
}
