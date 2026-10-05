use std::path::Path;

use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Host;

impl Module for Host {
    fn name(&self) -> &'static str {
        "Host"
    }
    fn id(&self) -> &'static str {
        "host"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let vendor = sys::dmi("sys_vendor").unwrap_or_default();
        let product = sys::dmi("product_name").unwrap_or_default();

        let base_host = if let Some(vm) = detect_vm(&vendor, &product) {
            Some(vm)
        } else {
            match (vendor.is_empty(), product.is_empty()) {
                (true, true) => None,
                (true, false) => Some(product),
                (false, true) => Some(vendor),
                (false, false) => {
                    if product.starts_with(&vendor) {
                        Some(product)
                    } else {
                        Some(format!("{vendor} {product}"))
                    }
                }
            }
        };

        let container = detect_container();

        let host_str = match (container, base_host) {
            (Some(c), Some(b)) => format!("{c} on {b}"),
            (Some(c), None) => c,
            (None, Some(b)) => b,
            (None, None) => return None,
        };

        Some(ModuleOutput::new("Host", host_str))
    }
}

pub fn detect_container() -> Option<String> {
    if Path::new("/.dockerenv").exists() {
        return Some("Docker Container".into());
    }
    if Path::new("/run/.containerenv").exists() {
        return Some("Podman Container".into());
    }
    if Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
        return Some("WSL (Windows Subsystem for Linux)".into());
    }
    if let Ok(ver) = std::fs::read_to_string("/proc/version")
        && (ver.contains("microsoft") || ver.contains("WSL"))
    {
        return Some("WSL (Windows Subsystem for Linux)".into());
    }
    if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
        if cgroup.contains("docker") {
            return Some("Docker Container".into());
        }
        if cgroup.contains("lxc") {
            return Some("LXC Container".into());
        }
        if cgroup.contains("podman") {
            return Some("Podman Container".into());
        }
        if cgroup.contains("kubepods") {
            return Some("Kubernetes Pod".into());
        }
    }
    None
}

pub fn detect_vm(vendor: &str, product: &str) -> Option<String> {
    let combined = format!("{vendor} {product}").to_ascii_lowercase();
    if combined.contains("qemu") || combined.contains("kvm") || combined.contains("bochs") {
        Some("KVM / QEMU Virtual Machine".into())
    } else if combined.contains("virtualbox") || combined.contains("innotek") {
        Some("Oracle VirtualBox".into())
    } else if combined.contains("vmware") {
        Some("VMware Virtual Machine".into())
    } else if combined.contains("hyper-v")
        || (vendor.to_ascii_lowercase().contains("microsoft") && combined.contains("virtual"))
    {
        Some("Microsoft Hyper-V".into())
    } else if combined.contains("xen")
        || std::fs::read_to_string("/sys/hypervisor/type")
            .map(|s| s.trim() == "xen")
            .unwrap_or(false)
    {
        Some("Xen Virtual Machine".into())
    } else if combined.contains("parallels") {
        Some("Parallels Virtual Machine".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_vm_virtualbox() {
        assert_eq!(
            detect_vm("innotek GmbH", "VirtualBox").as_deref(),
            Some("Oracle VirtualBox")
        );
    }

    #[test]
    fn test_detect_vm_qemu() {
        assert_eq!(
            detect_vm("QEMU", "Standard PC (Q35 + ICH9, 2009)").as_deref(),
            Some("KVM / QEMU Virtual Machine")
        );
    }

    #[test]
    fn test_detect_vm_real_hardware() {
        assert_eq!(detect_vm("Dell Inc.", "Latitude E6430"), None);
    }
}
