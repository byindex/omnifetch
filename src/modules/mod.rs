pub mod audioserver;
pub mod battery;
pub mod bios;
pub mod bluetooth;
pub mod bluetoothradio;
pub mod board;
pub mod bootmgr;
pub mod break_;
pub mod brightness;
pub mod btrfs;
pub mod camera;
pub mod chassis;
pub mod codec;
pub mod colors;
pub mod command;
pub mod containers;
pub mod cpu;
pub mod cpucache;
pub mod cputemp;
pub mod cpuusage;
pub mod cursor;
pub mod custom;
pub mod datetime;
pub mod de;
pub mod devenv;
pub mod disk;
pub mod diskio;
pub mod display;
pub mod displayserver;
pub mod dns;
pub mod editor;
pub mod font;
pub mod gamepad;
pub mod git;
pub mod gpu;
pub mod gpudriver;
pub mod host;
pub mod icons;
pub mod initsystem;
pub mod kernel;
pub mod keyboard;
pub mod lm;
pub mod loadavg;
pub mod locale;
pub mod localip;
pub mod media;
pub mod memory;
pub mod monitor;
pub mod mouse;
pub mod mpris;
pub mod netadapter;
pub mod netio;
pub mod opencl;
pub mod opengl;
pub mod os;
pub mod packages;
pub mod physicaldisk;
pub mod physicalmemory;
pub mod player;
pub mod poweradapter;
pub mod powerprofile;
pub mod processes;
pub mod publicip;
pub mod quote;
pub mod resolution;
pub mod security;
pub mod separator;
pub mod shell;
pub mod sound;
pub mod swap;
pub mod terminal;
pub mod terminalfont;
pub mod terminalsize;
pub mod terminaltheme;
pub mod theme;
pub mod title;
pub mod top;
pub mod touchpad;
pub mod tpm;
pub mod uptime;
pub mod users;
pub mod version;
pub mod vulkan;
pub mod wallpaper;
pub mod weather;
pub mod wifi;
pub mod wm;
pub mod wmtheme;
pub mod zpool;

use crate::module::BoxedModule;

pub const ALL_MODULE_IDS: &[&str] = &[
    "title",
    "separator",
    "break",
    "os",
    "kernel",
    "bootmgr",
    "initsystem",
    "security",
    "host",
    "board",
    "bios",
    "tpm",
    "chassis",
    "uptime",
    "loadavg",
    "processes",
    "packages",
    "datetime",
    "locale",
    "users",
    "version",
    "cpu",
    "cpucache",
    "cputemp",
    "cpuusage",
    "powerprofile",
    "gpu",
    "gpudriver",
    "display",
    "displayserver",
    "sound",
    "audioserver",
    "memory",
    "swap",
    "disk",
    "battery",
    "vulkan",
    "opengl",
    "shell",
    "terminal",
    "terminalfont",
    "terminalsize",
    "wm",
    "wmtheme",
    "de",
    "theme",
    "icons",
    "font",
    "cursor",
    "wallpaper",
    "editor",
    "devenv",
    "git",
    "netadapter",
    "localip",
    "dns",
    "wifi",
    "netio",
    "resolution",
    "brightness",
    "camera",
    "keyboard",
    "mouse",
    "touchpad",
    "gamepad",
    "media",
    "player",
    "bluetooth",
    "bluetoothradio",
    "physicaldisk",
    "physicalmemory",
    "diskio",
    "poweradapter",
    "lm",
    "opencl",
    "codec",
    "btrfs",
    "zpool",
    "terminaltheme",
    "top",
    "custom",
    "command",
    "monitor",
    "containers",
    "quote",
    "publicip",
    "weather",
    "colors",
];

pub fn create(name: &str) -> Option<BoxedModule> {
    let lower = name.trim().to_ascii_lowercase();
    match lower.as_str() {
        "title" => Some(Box::new(title::Title)),
        "separator" => Some(Box::new(separator::Separator)),
        "break" => Some(Box::new(break_::Break)),
        "os" => Some(Box::new(os::Os)),
        "kernel" => Some(Box::new(kernel::Kernel)),
        "bootmgr" => Some(Box::new(bootmgr::BootMgr)),
        "initsystem" => Some(Box::new(initsystem::InitSystem)),
        "security" => Some(Box::new(security::Security)),
        "host" => Some(Box::new(host::Host)),
        "board" => Some(Box::new(board::Board)),
        "bios" => Some(Box::new(bios::Bios)),
        "tpm" => Some(Box::new(tpm::Tpm)),
        "chassis" => Some(Box::new(chassis::Chassis)),
        "uptime" => Some(Box::new(uptime::Uptime)),
        "loadavg" => Some(Box::new(loadavg::Loadavg)),
        "processes" => Some(Box::new(processes::Processes)),
        "packages" => Some(Box::new(packages::Packages)),
        "datetime" | "date" | "time" => Some(Box::new(datetime::DateTime)),
        "locale" => Some(Box::new(locale::Locale)),
        "users" | "user" => Some(Box::new(users::Users)),
        "version" => Some(Box::new(version::Version)),
        "cpu" => Some(Box::new(cpu::Cpu)),
        "cpucache" => Some(Box::new(cpucache::CpuCache)),
        "cputemp" => Some(Box::new(cputemp::CpuTemp)),
        "cpuusage" => Some(Box::new(cpuusage::CpuUsage)),
        "powerprofile" => Some(Box::new(powerprofile::PowerProfile)),
        "gpu" => Some(Box::new(gpu::Gpu)),
        "gpudriver" => Some(Box::new(gpudriver::GpuDriver)),
        "display" => Some(Box::new(display::Display)),
        "displayserver" => Some(Box::new(displayserver::DisplayServer)),
        "sound" => Some(Box::new(sound::Sound)),
        "audioserver" => Some(Box::new(audioserver::AudioServer)),
        "memory" | "ram" => Some(Box::new(memory::Memory)),
        "swap" => Some(Box::new(swap::Swap)),
        "disk" => Some(Box::new(disk::Disk)),
        "battery" => Some(Box::new(battery::Battery)),
        "vulkan" => Some(Box::new(vulkan::Vulkan)),
        "opengl" => Some(Box::new(opengl::OpenGl)),
        "shell" => Some(Box::new(shell::Shell)),
        "terminal" => Some(Box::new(terminal::Terminal)),
        "terminalfont" => Some(Box::new(terminalfont::TerminalFont)),
        "terminalsize" => Some(Box::new(terminalsize::TerminalSize)),
        "wm" => Some(Box::new(wm::Wm)),
        "wmtheme" => Some(Box::new(wmtheme::WmTheme)),
        "de" => Some(Box::new(de::De)),
        "theme" => Some(Box::new(theme::Theme)),
        "icons" => Some(Box::new(icons::Icons)),
        "font" => Some(Box::new(font::Font)),
        "cursor" => Some(Box::new(cursor::Cursor)),
        "wallpaper" => Some(Box::new(wallpaper::Wallpaper)),
        "editor" => Some(Box::new(editor::Editor)),
        "devenv" => Some(Box::new(devenv::DevEnv)),
        "git" => Some(Box::new(git::Git)),
        "netadapter" => Some(Box::new(netadapter::NetAdapter)),
        "localip" => Some(Box::new(localip::LocalIp)),
        "dns" => Some(Box::new(dns::Dns)),
        "wifi" => Some(Box::new(wifi::Wifi)),
        "netio" => Some(Box::new(netio::NetIo)),
        "resolution" => Some(Box::new(resolution::Resolution)),
        "brightness" => Some(Box::new(brightness::Brightness)),
        "camera" => Some(Box::new(camera::Camera)),
        "keyboard" => Some(Box::new(keyboard::Keyboard)),
        "mouse" => Some(Box::new(mouse::Mouse)),
        "touchpad" => Some(Box::new(touchpad::Touchpad)),
        "gamepad" => Some(Box::new(gamepad::Gamepad)),
        "media" => Some(Box::new(media::Media)),
        "player" => Some(Box::new(player::Player)),
        "bluetooth" => Some(Box::new(bluetooth::Bluetooth)),
        "bluetoothradio" => Some(Box::new(bluetoothradio::BluetoothRadio)),
        "physicaldisk" => Some(Box::new(physicaldisk::PhysicalDisk)),
        "physicalmemory" => Some(Box::new(physicalmemory::PhysicalMemory)),
        "diskio" => Some(Box::new(diskio::DiskIo)),
        "poweradapter" => Some(Box::new(poweradapter::PowerAdapter)),
        "lm" => Some(Box::new(lm::Lm)),
        "opencl" => Some(Box::new(opencl::OpenCl)),
        "codec" => Some(Box::new(codec::Codec)),
        "btrfs" => Some(Box::new(btrfs::Btrfs)),
        "zpool" => Some(Box::new(zpool::Zpool)),
        "terminaltheme" => Some(Box::new(terminaltheme::TerminalTheme)),
        "top" => Some(Box::new(top::Top)),
        "custom" => Some(Box::new(custom::Custom)),
        "command" => Some(Box::new(command::Command)),
        "monitor" => Some(Box::new(monitor::Monitor)),
        "containers" => Some(Box::new(containers::Containers)),
        "quote" => Some(Box::new(quote::Quote)),
        "publicip" => Some(Box::new(publicip::PublicIp)),
        "weather" => Some(Box::new(weather::Weather)),
        "colors" => Some(Box::new(colors::Colors)),
        _ => None,
    }
}

pub fn create_module(name: &str) -> Option<BoxedModule> {
    if let Some(m) = create(name) {
        return Some(m);
    }
    let norm = name
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "");
    create(&norm)
}

// Matrix coordinates require direct indexing across both dimensions for dynamic programming.
#[allow(clippy::needless_range_loop)]
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut dp = vec![vec![0; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

pub fn find_closest_module(name: &str) -> Option<&'static str> {
    let lower = name
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '_', ' '], "");
    let mut best: Option<&'static str> = None;
    let mut best_dist = usize::MAX;

    for &candidate in ALL_MODULE_IDS {
        let dist = levenshtein(&lower, candidate);
        if dist < best_dist {
            best_dist = dist;
            best = Some(candidate);
        }
    }

    let threshold = (lower.len() / 2).clamp(2, 3);
    if best_dist <= threshold { best } else { None }
}

pub fn all() -> Vec<BoxedModule> {
    vec![
        // 1. Header
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(break_::Break),
        // 2. System / OS
        Box::new(os::Os),
        Box::new(kernel::Kernel),
        Box::new(bootmgr::BootMgr),
        Box::new(initsystem::InitSystem),
        Box::new(security::Security),
        Box::new(host::Host),
        Box::new(board::Board),
        Box::new(bios::Bios),
        Box::new(tpm::Tpm),
        Box::new(chassis::Chassis),
        Box::new(uptime::Uptime),
        Box::new(loadavg::Loadavg),
        Box::new(processes::Processes),
        Box::new(packages::Packages),
        Box::new(datetime::DateTime),
        Box::new(locale::Locale),
        Box::new(users::Users),
        Box::new(version::Version),
        Box::new(break_::Break),
        // 3. Desktop & Visual
        Box::new(shell::Shell),
        Box::new(terminal::Terminal),
        Box::new(terminalfont::TerminalFont),
        Box::new(terminalsize::TerminalSize),
        Box::new(terminaltheme::TerminalTheme),
        Box::new(wm::Wm),
        Box::new(wmtheme::WmTheme),
        Box::new(de::De),
        Box::new(theme::Theme),
        Box::new(icons::Icons),
        Box::new(font::Font),
        Box::new(cursor::Cursor),
        Box::new(wallpaper::Wallpaper),
        Box::new(editor::Editor),
        Box::new(devenv::DevEnv),
        Box::new(git::Git),
        Box::new(display::Display),
        Box::new(displayserver::DisplayServer),
        Box::new(resolution::Resolution),
        Box::new(break_::Break),
        // 4. Hardware & Performance
        Box::new(cpu::Cpu),
        Box::new(cpucache::CpuCache),
        Box::new(cputemp::CpuTemp),
        Box::new(cpuusage::CpuUsage),
        Box::new(powerprofile::PowerProfile),
        Box::new(poweradapter::PowerAdapter),
        Box::new(gpu::Gpu),
        Box::new(gpudriver::GpuDriver),
        Box::new(sound::Sound),
        Box::new(audioserver::AudioServer),
        Box::new(memory::Memory),
        Box::new(swap::Swap),
        Box::new(disk::Disk),
        Box::new(battery::Battery),
        Box::new(vulkan::Vulkan),
        Box::new(opengl::OpenGl),
        Box::new(break_::Break),
        // 5. Network & Connectivity
        Box::new(netadapter::NetAdapter),
        Box::new(localip::LocalIp),
        Box::new(dns::Dns),
        Box::new(wifi::Wifi),
        Box::new(netio::NetIo),
        Box::new(publicip::PublicIp),
        Box::new(weather::Weather),
        Box::new(break_::Break),
        // 6. Devices, Peripherals & Extra
        Box::new(brightness::Brightness),
        Box::new(camera::Camera),
        Box::new(keyboard::Keyboard),
        Box::new(mouse::Mouse),
        Box::new(touchpad::Touchpad),
        Box::new(gamepad::Gamepad),
        Box::new(media::Media),
        Box::new(player::Player),
        Box::new(bluetooth::Bluetooth),
        Box::new(bluetoothradio::BluetoothRadio),
        Box::new(physicaldisk::PhysicalDisk),
        Box::new(physicalmemory::PhysicalMemory),
        Box::new(diskio::DiskIo),
        Box::new(lm::Lm),
        Box::new(opencl::OpenCl),
        Box::new(codec::Codec),
        Box::new(btrfs::Btrfs),
        Box::new(zpool::Zpool),
        Box::new(top::Top),
        Box::new(custom::Custom),
        Box::new(command::Command),
        Box::new(monitor::Monitor),
        Box::new(containers::Containers),
        Box::new(break_::Break),
        Box::new(quote::Quote),
        Box::new(break_::Break),
        Box::new(colors::Colors),
    ]
}

pub fn default_modules() -> Vec<BoxedModule> {
    vec![
        // 1. Header
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(break_::Break),
        // 2. System
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(packages::Packages),
        Box::new(break_::Break),
        // 3. Desktop
        Box::new(shell::Shell),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(terminal::Terminal),
        Box::new(break_::Break),
        // 4. Hardware
        Box::new(cpu::Cpu),
        Box::new(gpu::Gpu),
        Box::new(memory::Memory),
        Box::new(swap::Swap),
        Box::new(disk::Disk),
        Box::new(break_::Break),
        // 5. Colors
        Box::new(colors::Colors),
    ]
}

/// Hardware detail, theming and live monitoring in one dump. Selected with `-p detailed`.
pub fn detailed_modules() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(break_::Break),
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(board::Board),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(break_::Break),
        Box::new(packages::Packages),
        Box::new(shell::Shell),
        Box::new(display::Display),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(theme::Theme),
        Box::new(icons::Icons),
        Box::new(font::Font),
        Box::new(cursor::Cursor),
        Box::new(terminal::Terminal),
        Box::new(terminalsize::TerminalSize),
        Box::new(break_::Break),
        Box::new(cpu::Cpu),
        Box::new(cpucache::CpuCache),
        Box::new(cputemp::CpuTemp),
        Box::new(gpu::Gpu),
        Box::new(sound::Sound),
        Box::new(memory::Memory),
        Box::new(swap::Swap),
        Box::new(disk::Disk),
        Box::new(battery::Battery),
        Box::new(break_::Break),
        Box::new(wifi::Wifi),
        Box::new(localip::LocalIp),
        Box::new(dns::Dns),
        Box::new(netio::NetIo),
        Box::new(locale::Locale),
        Box::new(loadavg::Loadavg),
        Box::new(processes::Processes),
        Box::new(datetime::DateTime),
        Box::new(break_::Break),
        Box::new(colors::Colors),
    ]
}

pub fn fast_modules() -> Vec<BoxedModule> {
    vec![
        // 1. Header
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(break_::Break),
        // 2. System
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(board::Board),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(packages::Packages),
        Box::new(break_::Break),
        // 3. Desktop
        Box::new(shell::Shell),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(theme::Theme),
        Box::new(icons::Icons),
        Box::new(font::Font),
        Box::new(cursor::Cursor),
        Box::new(terminal::Terminal),
        Box::new(terminalsize::TerminalSize),
        Box::new(break_::Break),
        // 4. Hardware
        Box::new(cpu::Cpu),
        Box::new(gpu::Gpu),
        Box::new(memory::Memory),
        Box::new(disk::Disk),
        Box::new(break_::Break),
        // 5. Colors
        Box::new(colors::Colors),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_module_exact_and_aliases() {
        assert!(create_module("bluetooth").is_some());
        assert!(create_module("cpu").is_some());
        assert!(create_module("ram").is_some());
        assert!(create_module("cpu-temp").is_some());
        assert!(create_module("CPU Temp").is_some());
        assert!(create_module("date").is_some());
        assert!(create_module("bluetooh").is_none());
    }

    #[test]
    fn test_find_closest_module_suggestions() {
        assert_eq!(find_closest_module("bluetooh"), Some("bluetooth"));
        assert_eq!(find_closest_module("bluetoth"), Some("bluetooth"));
        assert_eq!(find_closest_module("wif"), Some("wifi"));
        assert_eq!(find_closest_module("memmory"), Some("memory"));
        assert_eq!(find_closest_module("proccesses"), Some("processes"));
        assert_eq!(find_closest_module("totallyunknownxyz123"), None);
    }

    #[test]
    fn test_all_modules_count() {
        let all_mods = all();
        let unique_ids: std::collections::HashSet<_> = all_mods.iter().map(|m| m.id()).collect();
        for &id in ALL_MODULE_IDS {
            assert!(unique_ids.contains(id), "all() should include {id}");
        }
    }
}
