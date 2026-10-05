use crate::cmd;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use std::fs;
use std::path::Path;

pub struct Bluetooth;

impl Module for Bluetooth {
    fn name(&self) -> &'static str {
        "Bluetooth"
    }
    fn id(&self) -> &'static str {
        "bluetooth"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (has_adapter, is_enabled, connected_devices) = detect_bluetooth()?;

        if !has_adapter {
            return None;
        }

        if !is_enabled {
            return Some(ModuleOutput::new("Bluetooth", "Off"));
        }

        if connected_devices.is_empty() {
            Some(ModuleOutput::new("Bluetooth", "On (no devices)"))
        } else {
            Some(ModuleOutput::new("Bluetooth", connected_devices.join(", ")))
        }
    }
}

fn detect_bluetooth() -> Option<(bool, bool, Vec<String>)> {
    let bt_dir = Path::new("/sys/class/bluetooth");
    if !bt_dir.is_dir() {
        return None;
    }

    let mut has_adapter = false;
    let mut is_enabled = true;

    if let Ok(entries) = fs::read_dir(bt_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("hci") {
                has_adapter = true;
                let rfkill_state = e.path().join("rfkill/state");
                if let Some(state) = sys::read_trim(rfkill_state)
                    && state == "0"
                {
                    is_enabled = false;
                }
                break;
            }
        }
    }

    if !has_adapter {
        return None;
    }

    if !is_enabled {
        return Some((true, false, Vec::new()));
    }

    // sysfs/procfs need no subprocess. `busctl` is a fork plus a PATH walk.
    let mut devices = Vec::new();
    detect_via_sysfs_power(&mut devices);
    if devices.is_empty() {
        detect_via_proc_input(&mut devices);
    }

    if devices.is_empty() {
        // BlueZ sees devices the other two miss, but only if there is a service to ask.
        let bluez_present = crate::modules::mpris::has_service(
            crate::modules::mpris::Bus::System,
            "org.bluez",
            std::time::Duration::from_millis(30),
        );
        if bluez_present {
            // A successful busctl answer stands as the answer, empty list included.
            match detect_via_busctl() {
                Some(found) => devices = found,
                None => devices = detect_via_bluetoothctl(),
            }
        }
    }

    Some((has_adapter, is_enabled, devices))
}

fn detect_via_busctl() -> Option<Vec<String>> {
    let out = cmd::run(
        "busctl",
        &[
            "call",
            "--json=short",
            "--no-pager",
            "org.bluez",
            "/",
            "org.freedesktop.DBus.ObjectManager",
            "GetManagedObjects",
        ],
    )?;

    let val: crate::json::JsonValue = crate::json::parse(&out).ok()?;
    parse_busctl_objects(&val)
}

fn parse_busctl_objects(val: &crate::json::JsonValue) -> Option<Vec<String>> {
    let data_arr = val.get("data")?.as_array()?;
    let objects = data_arr.first()?.as_object()?;

    let mut devices = Vec::new();

    for (_path, ifaces_val) in objects {
        if let Some(dev) = ifaces_val.get("org.bluez.Device1") {
            let connected = dev
                .get("Connected")
                .and_then(|v| v.get("data"))
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
            if connected {
                let name = dev
                    .get("Alias")
                    .and_then(|v| v.get("data"))
                    .and_then(|s| s.as_str())
                    .or_else(|| {
                        dev.get("Name")
                            .and_then(|v| v.get("data"))
                            .and_then(|s| s.as_str())
                    })
                    .unwrap_or("Bluetooth Device");

                let battery_pct = ifaces_val
                    .get("org.bluez.Battery1")
                    .and_then(|b| b.get("Percentage"))
                    .and_then(|v| v.get("data"))
                    .and_then(|n| n.as_u64());

                let dev_str = if let Some(pct) = battery_pct {
                    format!("{name} ({pct}%)")
                } else {
                    name.to_string()
                };

                if !devices.iter().any(|d: &String| d.starts_with(name)) {
                    devices.push(dev_str);
                }
            }
        }
    }

    Some(devices)
}

/// Answers every device from one `bluetoothctl` process fed on stdin, not one fork per device.
fn detect_via_bluetoothctl() -> Vec<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let Some(list) = cmd::run("bluetoothctl", &["devices", "Connected"]) else {
        return Vec::new();
    };

    let macs = parse_device_list(&list);
    if macs.is_empty() {
        return Vec::new();
    }

    let mut script = String::new();
    for mac in &macs {
        script.push_str("info ");
        script.push_str(mac);
        script.push('\n');
    }

    let child = Command::new("bluetoothctl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else {
        return Vec::new();
    };
    if let Some(stdin) = child.stdin.as_mut()
        && stdin.write_all(script.as_bytes()).is_err()
    {
        return Vec::new();
    }
    drop(child.stdin.take());
    let Ok(output) = child.wait_with_output() else {
        return Vec::new();
    };
    let info = String::from_utf8_lossy(&output.stdout);

    parse_bluetoothctl_info(&info, &macs)
}

/// MAC addresses from `bluetoothctl devices Connected`.
fn parse_device_list(list: &str) -> Vec<String> {
    list.lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("Device ")?;
            rest.split_whitespace().next().map(str::to_string)
        })
        .collect()
}

/// Vendor part of a MAC (`78:0E:FC`), used to label a device that reports no name.
fn vendor_prefix(mac: &str) -> Option<String> {
    let cut = mac.match_indices(':').nth(2).map(|(i, _)| i)?;
    Some(mac[..cut].to_string())
}

/// Device labels from batched `bluetoothctl info`, keeping only headers named in `macs`.
fn parse_bluetoothctl_info(info: &str, macs: &[String]) -> Vec<String> {
    let mut devices: Vec<String> = Vec::new();
    let mut current_mac: Option<&str> = None;
    let mut current_name: Option<String> = None;
    let mut current_batt: Option<String> = None;

    fn flush(
        devices: &mut Vec<String>,
        mac: Option<&str>,
        name: &mut Option<String>,
        batt: &mut Option<String>,
    ) {
        if let Some(mac) = mac {
            let label = name
                .clone()
                .or_else(|| vendor_prefix(mac))
                .unwrap_or_else(|| mac.to_string());
            let entry = match batt {
                Some(p) => format!("{label} ({p}%)"),
                None => label,
            };
            if !devices.contains(&entry) {
                devices.push(entry);
            }
        }
        *name = None;
        *batt = None;
    }

    for line in info.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Device ") {
            flush(
                &mut devices,
                current_mac,
                &mut current_name,
                &mut current_batt,
            );
            current_mac = macs
                .iter()
                .find(|m| rest.starts_with(m.as_str()))
                .map(String::as_str);
        } else if let Some(v) = line.strip_prefix("Name: ") {
            current_name = Some(v.trim().trim_matches('"').to_string());
        } else if let Some(v) = line.strip_prefix("Battery Percentage:") {
            // `87% (0x57)`: the number before the bracket is the percentage.
            let head = v.split_once('(').map_or(v, |(head, _)| head);
            let digits = head.trim().trim_end_matches('%').trim();
            if !digits.is_empty() {
                current_batt = Some(digits.to_string());
            }
        }
    }
    flush(
        &mut devices,
        current_mac,
        &mut current_name,
        &mut current_batt,
    );

    devices
}

fn detect_via_sysfs_power(devices: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let is_bt_battery =
                name.starts_with("hid-") || name.contains("bluetooth") || name.contains("bluez");

            if is_bt_battery || path_contains_bluetooth(&path) {
                let model = sys::read_trim(path.join("model_name"))
                    .or_else(|| sys::read_trim(path.join("manufacturer")));
                let capacity =
                    sys::read_trim(path.join("capacity")).and_then(|c| c.parse::<u32>().ok());

                if let Some(m) = model {
                    let dev = if let Some(cap) = capacity {
                        format!("{m} ({cap}%)")
                    } else {
                        m
                    };
                    if !devices.iter().any(|d| d.starts_with(&dev)) {
                        devices.push(dev);
                    }
                }
            }
        }
    }
}

fn detect_via_proc_input(devices: &mut Vec<String>) {
    if let Ok(content) = fs::read_to_string("/proc/bus/input/devices") {
        let mut current_name: Option<String> = None;
        let mut is_bt_input = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                if is_bt_input && let Some(name) = current_name.take() {
                    let clean = name.replace("(AVRCP)", "").trim().to_string();
                    if !clean.is_empty() && !devices.iter().any(|d| d.starts_with(&clean)) {
                        devices.push(clean);
                    }
                }
                current_name = None;
                is_bt_input = false;
                continue;
            }

            if let Some(rest) = line.strip_prefix("I: Bus=") {
                if rest.starts_with("0005") || rest.starts_with("5 ") {
                    is_bt_input = true;
                }
            } else if let Some(rest) = line.strip_prefix("N: Name=") {
                let trimmed = rest.trim().trim_matches('"');
                current_name = Some(trimmed.to_string());
            }
        }

        if is_bt_input && let Some(name) = current_name {
            let clean = name.replace("(AVRCP)", "").trim().to_string();
            if !clean.is_empty() && !devices.iter().any(|d| d.starts_with(&clean)) {
                devices.push(clean);
            }
        }
    }
}

fn path_contains_bluetooth(path: &Path) -> bool {
    let Ok(target) = fs::read_link(path) else {
        return false;
    };
    let s = target.to_string_lossy();
    s.contains("/bluetooth/") || s.contains("/hci")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "Waiting to connect to bluetoothd...\n\
Device 78:0E:FC:1A:2B:3C Tender Pro\n\
Device AA:BB:CC:DD:EE:FF Second Thing\n";

    const INFO: &str = "Waiting to connect to bluetoothd...\n\
Device 78:0E:FC:1A:2B:3C (random)\n\
\tName: Tender Pro\n\
\tAlias: Tender Pro\n\
\tBattery Percentage: 87% (0x57)\n\
Device AA:BB:CC:DD:EE:FF (random)\n\
\tName: Second Thing\n";

    #[test]
    fn device_list_yields_macs_and_skips_noise() {
        assert_eq!(
            parse_device_list(LIST),
            vec![
                "78:0E:FC:1A:2B:3C".to_string(),
                "AA:BB:CC:DD:EE:FF".to_string()
            ]
        );
        assert!(parse_device_list("Waiting to connect to bluetoothd...\n").is_empty());
        assert!(parse_device_list("").is_empty());
    }

    #[test]
    fn busctl_reply_with_nothing_connected_yields_empty_not_none() {
        // None would mean "busctl failed" and send the caller off to bluetoothctl.
        let json_str = r#"{
            "data": [{
                "/org/bluez/hci0": {
                    "org.bluez.Adapter1": { "Powered": { "type": "b", "data": true } }
                },
                "/org/bluez/hci0/dev_11_22": {
                    "org.bluez.Device1": {
                        "Alias": { "type": "s", "data": "Headset" },
                        "Connected": { "type": "b", "data": false }
                    }
                }
            }]
        }"#;
        let json = crate::json::parse(json_str).unwrap();
        assert_eq!(parse_busctl_objects(&json), Some(Vec::new()));
    }

    #[test]
    fn info_pairs_names_and_battery_with_devices() {
        let macs = parse_device_list(LIST);
        assert_eq!(
            parse_bluetoothctl_info(INFO, &macs),
            vec!["Tender Pro (87%)".to_string(), "Second Thing".to_string()]
        );
    }

    #[test]
    fn info_ignores_devices_outside_the_requested_list() {
        let macs = vec!["78:0E:FC:1A:2B:3C".to_string()];
        let out = parse_bluetoothctl_info(INFO, &macs);
        assert_eq!(out, vec!["Tender Pro (87%)".to_string()]);
    }

    #[test]
    fn info_falls_back_to_vendor_prefix_when_unnamed() {
        let macs = vec!["78:0E:FC:1A:2B:3C".to_string()];
        let info = "Device 78:0E:FC:1A:2B:3C (random)\n\tAlias: x\n";
        assert_eq!(
            parse_bluetoothctl_info(info, &macs),
            vec!["78:0E:FC".to_string()]
        );
    }

    #[test]
    fn info_drops_duplicate_entries() {
        let macs = vec!["78:0E:FC:1A:2B:3C".to_string()];
        let block = "Device 78:0E:FC:1A:2B:3C (random)\n\tName: Tender Pro\n";
        let doubled = format!("{block}{block}");
        assert_eq!(
            parse_bluetoothctl_info(&doubled, &macs),
            vec!["Tender Pro".to_string()]
        );
    }

    #[test]
    fn test_parse_busctl_objects() {
        let sample_json = r#"{
            "type": "a{oa{sa{sv}}}",
            "data": [{
                "/org/bluez/hci0": {
                    "org.bluez.Adapter1": {
                        "Powered": {"type": "b", "data": true}
                    }
                },
                "/org/bluez/hci0/dev_B0_35_EE_1F_BC_06": {
                    "org.bluez.Device1": {
                        "Alias": {"type": "s", "data": "Proove Tender"},
                        "Connected": {"type": "b", "data": true}
                    },
                    "org.bluez.Battery1": {
                        "Percentage": {"type": "y", "data": 90}
                    }
                },
                "/org/bluez/hci0/dev_41_42_DF_E9_E8_00": {
                    "org.bluez.Device1": {
                        "Name": {"type": "s", "data": "F9"},
                        "Connected": {"type": "b", "data": false}
                    }
                }
            }]
        }"#;

        let val: crate::json::JsonValue = crate::json::parse(sample_json).unwrap();
        let devices = parse_busctl_objects(&val).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], "Proove Tender (90%)");
    }
}
