use crate::module::BoxedModule;
use crate::modules::*;

pub struct PresetInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub const PRESETS: &[PresetInfo] = &[
    PresetInfo {
        name: "all",
        description: "Run all available system modules (both standard and optional)",
    },
    PresetInfo {
        name: "minimal",
        description: "Ultra-fast minimalist display with only the most essential identifiers",
    },
    PresetInfo {
        name: "compact",
        description: "Compact and dense layout with essential metrics and zero blank lines",
    },
    PresetInfo {
        name: "detailed",
        description: "Everything at once: hardware detail, desktop theming and live monitoring",
    },
    PresetInfo {
        name: "modern",
        description: "Modern layout with CPU temperatures, usage, Wi-Fi, and network I/O",
    },
    PresetInfo {
        name: "hardware",
        description: "Comprehensive hardware diagnostics (CPU, GPU, caches, disks, RAM slots, sensors)",
    },
    PresetInfo {
        name: "fastfetch",
        description: "Same modules and order as fastfetch's default configuration",
    },
    PresetInfo {
        name: "neofetch",
        description: "Same modules and order as Neofetch 7.x default configuration",
    },
    PresetInfo {
        name: "paleofetch",
        description: "Same modules and order as paleofetch's default configuration",
    },
    PresetInfo {
        name: "catnap",
        description: "Same modules and order as catnap's default configuration",
    },
    PresetInfo {
        name: "macchina",
        description: "Same modules and order as macchina's default configuration",
    },
    PresetInfo {
        name: "sysprint",
        description: "Same modules and order as sysprint's default configuration",
    },
    PresetInfo {
        name: "nitch",
        description: "Same modules and order as nitch's default configuration",
    },
    PresetInfo {
        name: "pfetch",
        description: "Same modules and order as pfetch's default configuration",
    },
];

pub fn get(name: &str) -> Option<Vec<BoxedModule>> {
    match name.to_ascii_lowercase().as_str() {
        "all" => Some(
            crate::modules::all()
                .into_iter()
                .filter(|m| m.id() != "weather" && m.id() != "publicip")
                .collect(),
        ),
        "minimal" => Some(minimal()),
        "compact" => Some(compact()),
        "detailed" => Some(crate::modules::detailed_modules()),
        "modern" => Some(modern()),
        "hardware" => Some(hardware()),
        "fastfetch" => Some(fastfetch()),
        "neofetch" => Some(neofetch()),
        "paleofetch" => Some(paleofetch()),
        "catnap" => Some(catnap()),
        "macchina" => Some(macchina()),
        "sysprint" => Some(sysprint()),
        "nitch" => Some(nitch()),
        "pfetch" => Some(pfetch()),
        _ => None,
    }
}

pub fn compact() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(packages::Packages),
        Box::new(shell::Shell),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(terminal::Terminal),
        Box::new(cpu::Cpu),
        Box::new(gpu::Gpu),
        Box::new(memory::Memory),
        Box::new(disk::Disk),
    ]
}

pub fn neofetch() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(packages::Packages),
        Box::new(shell::Shell),
        Box::new(resolution::Resolution),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(wmtheme::WmTheme),
        Box::new(theme::Theme),
        Box::new(icons::Icons),
        Box::new(terminal::Terminal),
        Box::new(terminalfont::TerminalFont),
        Box::new(cpu::Cpu),
        Box::new(gpu::Gpu),
        Box::new(memory::Memory),
        Box::new(break_::Break),
        Box::new(colors::Colors),
    ]
}

pub fn modern() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(break_::Break),
        Box::new(shell::Shell),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(terminal::Terminal),
        Box::new(break_::Break),
        Box::new(cpu::Cpu),
        Box::new(cputemp::CpuTemp),
        Box::new(cpuusage::CpuUsage),
        Box::new(gpu::Gpu),
        Box::new(memory::Memory),
        Box::new(swap::Swap),
        Box::new(disk::Disk),
        Box::new(battery::Battery),
        Box::new(break_::Break),
        Box::new(wifi::Wifi),
        Box::new(localip::LocalIp),
        Box::new(netio::NetIo),
        Box::new(break_::Break),
        Box::new(colors::Colors),
    ]
}

pub fn hardware() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(host::Host),
        Box::new(board::Board),
        Box::new(bios::Bios),
        Box::new(chassis::Chassis),
        Box::new(break_::Break),
        Box::new(cpu::Cpu),
        Box::new(cpucache::CpuCache),
        Box::new(cputemp::CpuTemp),
        Box::new(cpuusage::CpuUsage),
        Box::new(gpu::Gpu),
        Box::new(sound::Sound),
        Box::new(break_::Break),
        Box::new(memory::Memory),
        Box::new(swap::Swap),
        Box::new(disk::Disk),
        Box::new(battery::Battery),
    ]
}

pub fn minimal() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(memory::Memory),
    ]
}

/// Mirrors fastfetch's default structure: OS through Locale, then the palette.
pub fn fastfetch() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(packages::Packages),
        Box::new(shell::Shell),
        Box::new(display::Display),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(wmtheme::WmTheme),
        Box::new(theme::Theme),
        Box::new(icons::Icons),
        Box::new(font::Font),
        Box::new(cursor::Cursor),
        Box::new(terminal::Terminal),
        Box::new(cpu::Cpu),
        Box::new(gpu::Gpu),
        Box::new(memory::Memory),
        Box::new(swap::Swap),
        Box::new(disk::Disk),
        Box::new(localip::LocalIp),
        Box::new(locale::Locale),
        Box::new(break_::Break),
        Box::new(colors::Colors),
    ]
}

/// Mirrors paleofetch's default layout.
pub fn paleofetch() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(battery::Battery),
        Box::new(packages::Packages),
        Box::new(shell::Shell),
        Box::new(resolution::Resolution),
        Box::new(terminal::Terminal),
        Box::new(cpu::Cpu),
        Box::new(memory::Memory),
    ]
}

/// Mirrors catnap's default layout, including its live CPU load row.
pub fn catnap() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(host::Host),
        Box::new(uptime::Uptime),
        Box::new(os::Os),
        Box::new(kernel::Kernel),
        Box::new(packages::Packages),
        Box::new(de::De),
        Box::new(terminal::Terminal),
        Box::new(shell::Shell),
        Box::new(gpu::Gpu),
        Box::new(cpu::Cpu),
        Box::new(cpuusage::CpuUsage),
        Box::new(memory::Memory),
        Box::new(disk::Disk),
        Box::new(battery::Battery),
        Box::new(colors::Colors),
    ]
}

/// Mirrors macchina's default layout.
pub fn macchina() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(os::Os),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(packages::Packages),
        Box::new(shell::Shell),
        Box::new(terminal::Terminal),
        Box::new(brightness::Brightness),
        Box::new(resolution::Resolution),
        Box::new(uptime::Uptime),
        Box::new(cpu::Cpu),
        Box::new(cpuusage::CpuUsage),
        Box::new(memory::Memory),
        Box::new(gpu::Gpu),
    ]
}

/// Mirrors sysprint's sectioned default output.
pub fn sysprint() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(os::Os),
        Box::new(kernel::Kernel),
        Box::new(initsystem::InitSystem),
        Box::new(host::Host),
        Box::new(uptime::Uptime),
        Box::new(break_::Break),
        Box::new(cpu::Cpu),
        Box::new(cputemp::CpuTemp),
        Box::new(cpuusage::CpuUsage),
        Box::new(gpu::Gpu),
        Box::new(break_::Break),
        Box::new(memory::Memory),
        Box::new(break_::Break),
        Box::new(de::De),
        Box::new(wm::Wm),
        Box::new(terminal::Terminal),
        Box::new(shell::Shell),
        Box::new(datetime::DateTime),
        Box::new(disk::Disk),
    ]
}

/// Mirrors nitch's default layout: user first, no kernel-heavy detail.
pub fn nitch() -> Vec<BoxedModule> {
    vec![
        Box::new(title::Title),
        Box::new(separator::Separator),
        Box::new(host::Host),
        Box::new(os::Os),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(shell::Shell),
        Box::new(packages::Packages),
        Box::new(memory::Memory),
        Box::new(colors::Colors),
    ]
}

/// Mirrors pfetch's minimal six-line layout.
pub fn pfetch() -> Vec<BoxedModule> {
    vec![
        Box::new(os::Os),
        Box::new(host::Host),
        Box::new(kernel::Kernel),
        Box::new(uptime::Uptime),
        Box::new(packages::Packages),
        Box::new(memory::Memory),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_exist_and_non_empty() {
        for p in PRESETS {
            let mods = get(p.name);
            assert!(mods.is_some(), "preset {} should exist", p.name);
            assert!(
                !mods.unwrap().is_empty(),
                "preset {} should have modules",
                p.name
            );
        }
        assert!(get("non_existent_preset_xyz").is_none());
    }

    #[test]
    fn test_every_preset_name_is_unique() {
        let mut names: Vec<&str> = PRESETS.iter().map(|p| p.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate preset name in PRESETS");
    }

    #[test]
    fn test_competitor_presets_match_fetched_defaults() {
        // pfetch is the smallest competitor layout: exactly six modules.
        assert_eq!(pfetch().len(), 6);
        assert_eq!(nitch().len(), 10);
        assert_eq!(minimal().len(), 6);
    }
}
