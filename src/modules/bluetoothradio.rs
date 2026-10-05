use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;

pub struct BluetoothRadio;

impl Module for BluetoothRadio {
    fn name(&self) -> &'static str {
        "BluetoothRadio"
    }

    fn id(&self) -> &'static str {
        "bluetoothradio"
    }

    fn run(&self) -> Option<ModuleOutput> {
        let entries = fs::read_dir("/sys/class/bluetooth").ok()?;
        let mut radios = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Look for hci0, hci1, etc.
            if !name.starts_with("hci") || name.contains(':') {
                continue;
            }
            let path = entry.path();
            let address = sys::read_trim(path.join("address")).unwrap_or_default();

            // Try to read driver / device vendor
            let mut info = name.clone();
            if !address.is_empty() {
                info = format!("{info} ({address})");
            }
            radios.push(info);
        }

        if radios.is_empty() {
            return None;
        }

        Some(ModuleOutput::new("BluetoothRadio", radios.join(", ")))
    }
}
