use crate::module::{Module, ModuleOutput};
use std::fs;
use std::path::Path;

pub struct BootMgr;

impl Module for BootMgr {
    fn name(&self) -> &'static str {
        "Bootloader"
    }
    fn id(&self) -> &'static str {
        "bootmgr"
    }
    fn run(&self) -> Option<ModuleOutput> {
        #[cfg(target_os = "linux")]
        {
            if let Some(loader) = detect_bootloader() {
                return Some(ModuleOutput::new("Bootloader", loader));
            }
        }

        None
    }
}

#[cfg(target_os = "linux")]
fn detect_bootloader() -> Option<String> {
    // Only efivar *names* are read: reading contents can block in the kernel for ~1s, so the guess uses which exist.
    let efivars = Path::new("/sys/firmware/efi/efivars");
    if efivars.is_dir() {
        if let Ok(entries) = fs::read_dir(efivars) {
            for entry in entries.flatten() {
                let s = entry.file_name();
                if s.as_encoded_bytes().starts_with(b"LoaderInfo-") {
                    return Some("systemd-boot".to_string());
                }
            }
        }

        if let Ok(entries) = fs::read_dir(efivars) {
            for entry in entries.flatten() {
                let s = entry.file_name();
                if s.as_encoded_bytes().starts_with(b"LoaderImageIdentifier-") {
                    return Some("systemd-boot".to_string());
                }
            }
        }
    }

    // Check config files in /boot
    if Path::new("/boot/grub/grub.cfg").exists() || Path::new("/boot/grub2/grub.cfg").exists() {
        return Some("GRUB".to_string());
    }
    if Path::new("/boot/limine.cfg").exists() || Path::new("/boot/limine/limine.conf").exists() {
        return Some("Limine".to_string());
    }
    if Path::new("/boot/refind_linux.conf").exists() {
        return Some("rEFInd".to_string());
    }

    if Path::new("/sys/firmware/efi").is_dir() {
        return Some("UEFI Direct".to_string());
    }

    None
}
