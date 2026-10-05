use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogoType {
    Default,
    Small,
    None,
}

impl LogoType {
    fn next(self) -> Self {
        match self {
            LogoType::Default => LogoType::Small,
            LogoType::Small => LogoType::None,
            LogoType::None => LogoType::Default,
        }
    }

    fn desc(self) -> &'static str {
        match self {
            LogoType::Default => "Built-in ASCII art",
            LogoType::Small => "Built-in small ASCII art",
            LogoType::None => "Disable logo",
        }
    }

    fn to_config_str(self) -> &'static str {
        match self {
            LogoType::Default => "auto",
            LogoType::Small => "mini",
            LogoType::None => "none",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogoPos {
    Left,
    Top,
}

impl LogoPos {
    fn next(self) -> Self {
        match self {
            LogoPos::Left => LogoPos::Top,
            LogoPos::Top => LogoPos::Left,
        }
    }

    fn desc(self) -> &'static str {
        match self {
            LogoPos::Left => "Place the logo to the left (side-by-side)",
            LogoPos::Top => "Place the logo on top (above info)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Minimal,
    Full,
}

impl OutputFormat {
    fn next(self) -> Self {
        match self {
            OutputFormat::Minimal => OutputFormat::Full,
            OutputFormat::Full => OutputFormat::Minimal,
        }
    }

    fn desc(self) -> &'static str {
        match self {
            OutputFormat::Minimal => "Minimal TOML config",
            OutputFormat::Full => "Full TOML config with all comments & options",
        }
    }
}

#[derive(Clone)]
pub struct ModuleItem {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub enabled: bool,
    pub is_special: bool,
}

fn initial_modules() -> Vec<ModuleItem> {
    vec![
        // Column 1
        ModuleItem {
            id: "title".into(),
            name: "Title".into(),
            desc: "Print the title, including your username and hostname".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "separator".into(),
            name: "Separator".into(),
            desc: "Print a line of separator characters".into(),
            enabled: true,
            is_special: true,
        },
        ModuleItem {
            id: "os".into(),
            name: "OS".into(),
            desc: "Operating system name and version".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "host".into(),
            name: "Host".into(),
            desc: "Host / motherboard / system model".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "bios".into(),
            name: "BIOS".into(),
            desc: "BIOS / UEFI firmware information".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "bootmgr".into(),
            name: "Bootmgr".into(),
            desc: "Bootloader and firmware type".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "board".into(),
            name: "Board".into(),
            desc: "Motherboard model and manufacturer".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "chassis".into(),
            name: "Chassis".into(),
            desc: "Chassis form factor (Desktop, Laptop, etc.)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "kernel".into(),
            name: "Kernel".into(),
            desc: "Kernel release and version".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "initsystem".into(),
            name: "InitSystem".into(),
            desc: "Init system and service manager (e.g. systemd)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "uptime".into(),
            name: "Uptime".into(),
            desc: "System uptime since last boot".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "loadavg".into(),
            name: "Loadavg".into(),
            desc: "System load averages (1/5/15 min)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "processes".into(),
            name: "Processes".into(),
            desc: "Number of running processes and threads".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "packages".into(),
            name: "Packages".into(),
            desc: "Number of installed package managers and packages".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "shell".into(),
            name: "Shell".into(),
            desc: "Current shell and version".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "editor".into(),
            name: "Editor".into(),
            desc: "Default text editor ($EDITOR)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "display".into(),
            name: "Display".into(),
            desc: "Screen resolution and refresh rate".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "brightness".into(),
            name: "Brightness".into(),
            desc: "Screen backlight brightness percentage".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "monitor".into(),
            name: "Monitor".into(),
            desc: "Connected monitors and displays".into(),
            enabled: false,
            is_special: false,
        },
        // Column 2
        ModuleItem {
            id: "lm".into(),
            name: "LM".into(),
            desc: "Login Manager / Display Manager (SDDM, GDM, LightDM, etc.)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "de".into(),
            name: "DE".into(),
            desc: "Desktop Environment".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "wm".into(),
            name: "WM".into(),
            desc: "Window Manager".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "wmtheme".into(),
            name: "WMTheme".into(),
            desc: "Window Manager theme / window decoration".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "theme".into(),
            name: "Theme".into(),
            desc: "GTK / Qt / Desktop theme".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "icons".into(),
            name: "Icons".into(),
            desc: "Icon theme".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "font".into(),
            name: "Font".into(),
            desc: "System font names and sizes".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "cursor".into(),
            name: "Cursor".into(),
            desc: "Cursor theme and size".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "wallpaper".into(),
            name: "Wallpaper".into(),
            desc: "Current desktop wallpaper image path".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "terminal".into(),
            name: "Terminal".into(),
            desc: "Current terminal emulator".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "terminalfont".into(),
            name: "TerminalFont".into(),
            desc: "Terminal font name and size".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "terminalsize".into(),
            name: "TerminalSize".into(),
            desc: "Terminal dimensions in columns and rows".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "terminaltheme".into(),
            name: "TerminalTheme".into(),
            desc: "Terminal color scheme / background theme".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "cpu".into(),
            name: "CPU".into(),
            desc: "Processor model, cores, and frequency".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "cpucache".into(),
            name: "CPUCache".into(),
            desc: "CPU cache sizes (L1, L2, L3)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "cpuusage".into(),
            name: "CPUUsage".into(),
            desc: "CPU utilization percentage".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "gpu".into(),
            name: "GPU".into(),
            desc: "Graphics processor and driver".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "top".into(),
            name: "Top".into(),
            desc: "Top process by memory usage (RSS)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "codec".into(),
            name: "Codec".into(),
            desc: "Hardware video acceleration codecs (VA-API, NVDEC)".into(),
            enabled: false,
            is_special: false,
        },
        // Column 3
        ModuleItem {
            id: "memory".into(),
            name: "Memory".into(),
            desc: "RAM usage, total, and percentage".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "physicalmemory".into(),
            name: "PhysicalMemory".into(),
            desc: "Physical RAM sticks and slots (DMI Type 17)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "swap".into(),
            name: "Swap".into(),
            desc: "Swap memory usage, total, and percentage".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "disk".into(),
            name: "Disk".into(),
            desc: "Disk usage by mount point".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "btrfs".into(),
            name: "Btrfs".into(),
            desc: "Btrfs filesystems and allocated space".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "zpool".into(),
            name: "Zpool".into(),
            desc: "ZFS storage pools".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "battery".into(),
            name: "Battery".into(),
            desc: "Battery charge percentage and charging status".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "poweradapter".into(),
            name: "PowerAdapter".into(),
            desc: "AC power adapter connection and wattage".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "player".into(),
            name: "Player".into(),
            desc: "Active MPRIS media players".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "media".into(),
            name: "Media".into(),
            desc: "Now playing media track and status".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "publicip".into(),
            name: "PublicIp".into(),
            desc: "Public IPv4 address".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "localip".into(),
            name: "LocalIp".into(),
            desc: "Local network IP addresses and MAC".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "dns".into(),
            name: "DNS".into(),
            desc: "Configured DNS nameservers".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "wifi".into(),
            name: "Wifi".into(),
            desc: "Wi-Fi SSID, signal strength, and protocol".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "datetime".into(),
            name: "DateTime".into(),
            desc: "Current date and time".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "locale".into(),
            name: "Locale".into(),
            desc: "System locale settings".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "vulkan".into(),
            name: "Vulkan".into(),
            desc: "Vulkan API version and extensions".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "opengl".into(),
            name: "OpenGL".into(),
            desc: "OpenGL renderer and version".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "opencl".into(),
            name: "OpenCL".into(),
            desc: "OpenCL platforms and drivers".into(),
            enabled: false,
            is_special: false,
        },
        // Column 4
        ModuleItem {
            id: "users".into(),
            name: "Users".into(),
            desc: "Currently logged in user accounts".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "bluetooth".into(),
            name: "Bluetooth".into(),
            desc: "Bluetooth status and paired devices".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "bluetoothradio".into(),
            name: "BluetoothRadio".into(),
            desc: "Bluetooth controller HCI adapters".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "sound".into(),
            name: "Sound".into(),
            desc: "Audio devices and active volume".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "camera".into(),
            name: "Camera".into(),
            desc: "Connected video webcams".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "gamepad".into(),
            name: "Gamepad".into(),
            desc: "Connected game controllers and joysticks".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "mouse".into(),
            name: "Mouse".into(),
            desc: "Connected computer mice".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "touchpad".into(),
            name: "Touchpad".into(),
            desc: "Connected touchpads and trackpoints".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "keyboard".into(),
            name: "Keyboard".into(),
            desc: "Connected physical keyboards".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "weather".into(),
            name: "Weather".into(),
            desc: "Current weather and temperature".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "netio".into(),
            name: "NetIO".into(),
            desc: "Network traffic (RX / TX throughput)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "diskio".into(),
            name: "DiskIO".into(),
            desc: "Disk read and write throughput".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "physicaldisk".into(),
            name: "PhysicalDisk".into(),
            desc: "Physical storage drives (SSD/HDD/NVMe)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "tpm".into(),
            name: "TPM".into(),
            desc: "Trusted Platform Module (TPM) version".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "version".into(),
            name: "Version".into(),
            desc: "Omnifetch version and target architecture".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "break".into(),
            name: "Break".into(),
            desc: "An empty line / separator break".into(),
            enabled: true,
            is_special: true,
        },
        ModuleItem {
            id: "colors".into(),
            name: "Colors".into(),
            desc: "Terminal 16-color palette test blocks".into(),
            enabled: true,
            is_special: false,
        },
        ModuleItem {
            id: "quote".into(),
            name: "Quote".into(),
            desc: "Random programming / CS quote with probability chances".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "security".into(),
            name: "Security".into(),
            desc: "Kernel security modules (AppArmor, SELinux, Landlock)".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "cputemp".into(),
            name: "CPUTemp".into(),
            desc: "CPU package/core temperatures".into(),
            enabled: false,
            is_special: false,
        },
        ModuleItem {
            id: "devenv".into(),
            name: "DevEnv".into(),
            desc: "Development environments and toolchains".into(),
            enabled: false,
            is_special: false,
        },
    ]
}

struct RawMode {
    orig: libc::termios,
}

impl RawMode {
    fn enter() -> Option<Self> {
        unsafe {
            if libc::isatty(libc::STDIN_FILENO) != 1 {
                return None;
            }
            let mut orig = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut orig) != 0 {
                return None;
            }
            let mut raw = orig;
            raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
            raw.c_oflag &= !(libc::OPOST);
            raw.c_cflag |= libc::CS8;
            raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw) != 0 {
                return None;
            }
            // Enter alternate screen buffer & hide cursor
            print!("\x1b[?1049h\x1b[?25l");
            let _ = io::stdout().flush();
            Some(Self { orig })
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        unsafe {
            print!("\x1b[?25h\x1b[?1049l");
            let _ = io::stdout().flush();
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &self.orig);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Key {
    None,
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Backspace,
    Delete,
    Esc,
    Ctrl(char),
}

fn parse_escape_sequence(bytes: &[u8]) -> Key {
    if bytes.len() < 2 {
        return Key::None;
    }
    match bytes[1] {
        b'[' => {
            if bytes.len() >= 3 {
                match bytes[2] {
                    b'A' => return Key::Up,
                    b'B' => return Key::Down,
                    b'C' => return Key::Right,
                    b'D' => return Key::Left,
                    b'H' => return Key::Home,
                    b'F' => return Key::End,
                    b'1'..=b'9' => {
                        if bytes.get(3) == Some(&b'~') {
                            match bytes[2] {
                                b'1' | b'7' => return Key::Home,
                                b'4' | b'8' => return Key::End,
                                b'3' => return Key::Delete,
                                b'5' => return Key::PageUp,
                                b'6' => return Key::PageDown,
                                _ => return Key::None,
                            }
                        } else if bytes.len() >= 4 {
                            let last = bytes[bytes.len() - 1];
                            match last {
                                // Shift+Up and Shift+Down reorder the row.
                                b'A' => return Key::Char('K'),
                                b'B' => return Key::Char('J'),
                                b'C' => return Key::Right,
                                b'D' => return Key::Left,
                                _ => return Key::None,
                            }
                        }
                    }
                    _ => return Key::None,
                }
            }
        }
        b'O' => {
            if bytes.len() >= 3 {
                match bytes[2] {
                    b'A' => return Key::Up,
                    b'B' => return Key::Down,
                    b'C' => return Key::Right,
                    b'D' => return Key::Left,
                    b'H' => return Key::Home,
                    b'F' => return Key::End,
                    _ => return Key::None,
                }
            }
        }
        _ => return Key::None,
    }
    Key::None
}

fn read_key() -> Key {
    let mut buf = [0u8; 64];
    let n = unsafe {
        libc::read(
            libc::STDIN_FILENO,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
        )
    };
    if n <= 0 {
        return Key::None;
    }
    let n = n as usize;
    let bytes = &buf[..n];

    if bytes[0] == 0x1b {
        if n == 1 {
            // Check if more bytes of an escape sequence are in flight
            unsafe {
                let mut pfd = libc::pollfd {
                    fd: libc::STDIN_FILENO,
                    events: libc::POLLIN,
                    revents: 0,
                };
                if libc::poll(&mut pfd, 1, 50) > 0 {
                    let mut rest = [0u8; 63];
                    let n2 = libc::read(
                        libc::STDIN_FILENO,
                        rest.as_mut_ptr() as *mut libc::c_void,
                        rest.len(),
                    );
                    if n2 > 0 {
                        let mut combined = Vec::with_capacity(1 + n2 as usize);
                        combined.push(0x1b);
                        combined.extend_from_slice(&rest[..n2 as usize]);
                        return parse_escape_sequence(&combined);
                    }
                }
            }
            return Key::Esc;
        } else {
            return parse_escape_sequence(bytes);
        }
    }

    match bytes[0] {
        b'\r' | b'\n' => Key::Enter,
        0x03 => Key::Ctrl('c'),
        0x7f | 0x08 => Key::Backspace,
        b' ' => Key::Char(' '),
        c => {
            if let Ok(s) = std::str::from_utf8(bytes)
                && let Some(ch) = s.chars().next()
            {
                return Key::Char(ch);
            }
            Key::Char(c as char)
        }
    }
}

pub fn run_interactive(target_path: &Path) -> Result<(), String> {
    let _raw = RawMode::enter().ok_or_else(|| "Failed to enter raw terminal mode".to_string())?;

    let mut logo_type = LogoType::Default;
    let mut logo_pos = LogoPos::Left;
    let mut output_format = OutputFormat::Minimal;
    let mut modules = initial_modules();

    let mut cursor_col: usize = 0;
    let mut cursor_row: usize = 0;
    let mut scroll_row: usize = 0;
    let num_cols = 4;

    loop {
        let rows_per_col = modules.len().div_ceil(num_cols).max(1);
        let size = crate::sys::terminal_size();
        let term_cols = size.map_or(80, |s| s.cols as usize).max(60);
        let term_rows = size.map_or(24, |s| s.rows as usize).max(16);

        let visible_rows = (term_rows.saturating_sub(11)).clamp(4, rows_per_col);
        if cursor_row < scroll_row {
            scroll_row = cursor_row;
        } else if cursor_row >= scroll_row + visible_rows {
            scroll_row = cursor_row + 1 - visible_rows;
        }
        if scroll_row + visible_rows > rows_per_col {
            scroll_row = rows_per_col.saturating_sub(visible_rows);
        }

        // Render entire frame
        let mut out = String::with_capacity(4096);
        out.push_str("\x1b[H"); // Cursor home

        // Header
        out.push_str("  \x1b[1momnifetch\x1b[m configuration\x1b[2m    interactive config generator\x1b[m\x1b[K\r\n\x1b[K\r\n");

        // Logo type
        out.push_str("  \x1b[1;4mL\x1b[24mogo type:\x1b[m  ");
        let lt_opts = [
            (LogoType::Default, "default"),
            (LogoType::Small, "small"),
            (LogoType::None, "none"),
        ];
        for (lt, label) in lt_opts {
            if logo_type == lt {
                out.push_str(&format!("\x1b[32m[x] {label}\x1b[m  "));
            } else {
                out.push_str(&format!("[ ] {label}  "));
            }
        }
        out.push_str(&format!("\x1b[2m- {}\x1b[m\x1b[K\r\n", logo_type.desc()));

        // Logo position
        out.push_str("  \x1b[1;mLogo \x1b[4mp\x1b[24mosition:\x1b[m  ");
        let lp_opts = [(LogoPos::Left, "left"), (LogoPos::Top, "top")];
        for (lp, label) in lp_opts {
            if logo_pos == lp {
                out.push_str(&format!("\x1b[32m[x] {label}\x1b[m  "));
            } else {
                out.push_str(&format!("[ ] {label}  "));
            }
        }
        out.push_str(&format!("\x1b[2m- {}\x1b[m\x1b[K\r\n", logo_pos.desc()));

        // Output
        out.push_str("  \x1b[1;4mO\x1b[24mutput:\x1b[m  ");
        let out_opts = [
            (OutputFormat::Minimal, "minimal"),
            (OutputFormat::Full, "full"),
        ];
        for (of, label) in out_opts {
            if output_format == of {
                out.push_str(&format!("\x1b[32m[x] {label}\x1b[m  "));
            } else {
                out.push_str(&format!("[ ] {label}  "));
            }
        }
        out.push_str(&format!(
            "\x1b[2m- {}\x1b[m\x1b[K\r\n\x1b[K\r\n",
            output_format.desc()
        ));

        // Modules count & scroll indicator
        let selected_count = modules.iter().filter(|m| m.enabled).count();
        let scroll_info = if rows_per_col > visible_rows {
            format!(
                " \x1b[2m(rows {}-{} of {}, scroll with ↑/↓)\x1b[m",
                scroll_row + 1,
                (scroll_row + visible_rows).min(rows_per_col),
                rows_per_col
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  \x1b[1mModules:\x1b[m\x1b[32m  [{}/{} selected]\x1b[m{}\x1b[K\r\n\x1b[K\r\n",
            selected_count,
            modules.len(),
            scroll_info
        ));

        // Grid of modules
        let col_width = ((term_cols - 2) / num_cols).clamp(16, 22);
        for row in scroll_row..(scroll_row + visible_rows).min(rows_per_col) {
            out.push_str("  ");
            for col in 0..num_cols {
                let idx = col * rows_per_col + row;
                if idx < modules.len() {
                    let item = &modules[idx];
                    let is_focused = col == cursor_col && row == cursor_row;

                    let mut text = String::new();
                    if item.is_special {
                        if item.enabled {
                            if is_focused {
                                text.push_str(&format!(
                                    "\x1b[7;36m[-]\x1b[m \x1b[36m{}\x1b[m",
                                    item.name
                                ));
                            } else {
                                text.push_str(&format!(
                                    "\x1b[36m[-]\x1b[m \x1b[36m{}\x1b[m",
                                    item.name
                                ));
                            }
                        } else if is_focused {
                            text.push_str(&format!("\x1b[7m[ ]\x1b[m {}", item.name));
                        } else {
                            text.push_str(&format!("[ ] {}", item.name));
                        }
                    } else if item.enabled {
                        if is_focused {
                            text.push_str(&format!(
                                "\x1b[7;32m[x]\x1b[m \x1b[32m{}\x1b[m",
                                item.name
                            ));
                        } else {
                            text.push_str(&format!(
                                "\x1b[32m[x]\x1b[m \x1b[32m{}\x1b[m",
                                item.name
                            ));
                        }
                    } else if is_focused {
                        text.push_str(&format!("\x1b[7m[ ]\x1b[m {}", item.name));
                    } else {
                        text.push_str(&format!("[ ] {}", item.name));
                    }

                    // Pad visible width
                    let visible_len = 4 + item.name.len();
                    let pad = col_width.saturating_sub(visible_len);
                    text.push_str(&" ".repeat(pad));
                    out.push_str(&text);
                } else {
                    out.push_str(&" ".repeat(col_width));
                }
            }
            out.push_str("\x1b[K\r\n");
        }

        out.push_str("\x1b[K\r\n");

        // Module description line
        let cur_idx = cursor_col * rows_per_col + cursor_row;
        let desc = if cur_idx < modules.len() {
            &modules[cur_idx].desc
        } else {
            ""
        };
        out.push_str(&format!("  {}\x1b[K\r\n", desc));

        // Hotkeys help
        out.push_str("  \x1b[36m↑/↓\x1b[m \x1b[2mk/j\x1b[m move  \x1b[36m←/→\x1b[m col  \x1b[36mSpace\x1b[m toggle/del  \x1b[36mf/F\x1b[m all/invert  \x1b[36mK/J\x1b[m reorder  \x1b[36mb/B\x1b[m break/sep  \x1b[36md\x1b[m del\x1b[K\r\n");
        out.push_str("  \x1b[36ml/L\x1b[m logo  \x1b[36mp/P\x1b[m position  \x1b[36mo\x1b[m minimal/full  \x1b[36ms/Enter\x1b[m save  \x1b[36mq/Esc\x1b[m quit  \x1b[36mg/G\x1b[m top/bottom\x1b[K");

        print!("{out}");
        let _ = io::stdout().flush();

        // Handle user input
        match read_key() {
            Key::Up | Key::Char('k') => {
                if cursor_row > 0 {
                    cursor_row -= 1;
                } else if cursor_col > 0 {
                    cursor_col -= 1;
                    cursor_row = rows_per_col.saturating_sub(1);
                    while cursor_col * rows_per_col + cursor_row >= modules.len() && cursor_row > 0
                    {
                        cursor_row -= 1;
                    }
                }
            }
            Key::Down | Key::Char('j') => {
                if cursor_row + 1 < rows_per_col
                    && (cursor_col * rows_per_col + cursor_row + 1) < modules.len()
                {
                    cursor_row += 1;
                } else if cursor_col + 1 < num_cols
                    && ((cursor_col + 1) * rows_per_col) < modules.len()
                {
                    cursor_col += 1;
                    cursor_row = 0;
                }
            }
            Key::Left | Key::Char('h') => {
                if cursor_col > 0 {
                    cursor_col -= 1;
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    if cur_idx >= modules.len() {
                        cursor_row = modules.len().saturating_sub(1) % rows_per_col;
                    }
                }
            }
            Key::Right => {
                if cursor_col + 1 < num_cols {
                    let next_col = cursor_col + 1;
                    let next_idx = next_col * rows_per_col + cursor_row;
                    if next_idx < modules.len() {
                        cursor_col = next_col;
                    } else if next_col * rows_per_col < modules.len() {
                        cursor_col = next_col;
                        cursor_row = modules.len().saturating_sub(1) % rows_per_col;
                    }
                }
            }
            Key::Char(' ') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx < modules.len() {
                    if modules[cur_idx].is_special {
                        modules.remove(cur_idx);
                        let total = modules.len();
                        if total > 0 {
                            let target = cur_idx.min(total - 1);
                            let new_rows = total.div_ceil(num_cols).max(1);
                            cursor_col = (target / new_rows).min(num_cols - 1);
                            cursor_row = target % new_rows;
                        } else {
                            cursor_col = 0;
                            cursor_row = 0;
                        }
                    } else {
                        modules[cur_idx].enabled = !modules[cur_idx].enabled;
                    }
                }
            }
            Key::Char('f') => {
                let all_enabled = modules.iter().all(|m| m.enabled);
                for m in &mut modules {
                    m.enabled = !all_enabled;
                }
            }
            Key::Char('F') => {
                for m in &mut modules {
                    m.enabled = !m.enabled;
                }
            }
            Key::Char('K') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx > 0 && cur_idx < modules.len() {
                    modules.swap(cur_idx, cur_idx - 1);
                    let target = cur_idx - 1;
                    cursor_col = (target / rows_per_col).min(num_cols - 1);
                    cursor_row = target % rows_per_col;
                }
            }
            Key::Char('J') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx + 1 < modules.len() {
                    modules.swap(cur_idx, cur_idx + 1);
                    let target = cur_idx + 1;
                    cursor_col = (target / rows_per_col).min(num_cols - 1);
                    cursor_row = target % rows_per_col;
                }
            }
            Key::Char('b') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                let insert_pos = (cur_idx + 1).min(modules.len());
                modules.insert(
                    insert_pos,
                    ModuleItem {
                        id: "break".into(),
                        name: "Break".into(),
                        desc: "An empty line / separator break".into(),
                        enabled: true,
                        is_special: true,
                    },
                );
                let total = modules.len();
                let new_rows = total.div_ceil(num_cols).max(1);
                cursor_col = (insert_pos / new_rows).min(num_cols - 1);
                cursor_row = insert_pos % new_rows;
            }
            Key::Char('B') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                let insert_pos = (cur_idx + 1).min(modules.len());
                modules.insert(
                    insert_pos,
                    ModuleItem {
                        id: "separator".into(),
                        name: "Separator".into(),
                        desc: "Print a line of separator characters".into(),
                        enabled: true,
                        is_special: true,
                    },
                );
                let total = modules.len();
                let new_rows = total.div_ceil(num_cols).max(1);
                cursor_col = (insert_pos / new_rows).min(num_cols - 1);
                cursor_row = insert_pos % new_rows;
            }
            Key::Char('d') | Key::Char('D') | Key::Delete | Key::Backspace => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx < modules.len() {
                    if modules[cur_idx].is_special {
                        modules.remove(cur_idx);
                        let total = modules.len();
                        if total > 0 {
                            let target = cur_idx.min(total - 1);
                            let new_rows = total.div_ceil(num_cols).max(1);
                            cursor_col = (target / new_rows).min(num_cols - 1);
                            cursor_row = target % new_rows;
                        } else {
                            cursor_col = 0;
                            cursor_row = 0;
                        }
                    } else {
                        modules[cur_idx].enabled = false;
                    }
                }
            }
            Key::Char('l') | Key::Char('L') => {
                logo_type = logo_type.next();
            }
            Key::Char('p') | Key::Char('P') => {
                logo_pos = logo_pos.next();
            }
            Key::Char('o') | Key::Char('O') => {
                output_format = output_format.next();
            }
            Key::Char('g') | Key::Home => {
                cursor_col = 0;
                cursor_row = 0;
            }
            Key::Char('G') | Key::End => {
                let last = modules.len().saturating_sub(1);
                cursor_col = (last / rows_per_col).min(num_cols - 1);
                cursor_row = last % rows_per_col;
            }
            Key::PageUp => {
                cursor_row = cursor_row.saturating_sub(visible_rows);
            }
            Key::PageDown => {
                let max_row = if cursor_col == num_cols - 1 {
                    modules.len().saturating_sub(1) % rows_per_col
                } else {
                    rows_per_col.saturating_sub(1)
                };
                cursor_row = (cursor_row + visible_rows).min(max_row);
            }
            Key::None => {}
            Key::Char('s') | Key::Enter => {
                // Save and exit
                break;
            }
            Key::Char('q') | Key::Esc | Key::Ctrl('c') => {
                println!("Configuration cancelled.");
                return Ok(());
            }
            _ => {}
        }
    }

    // Generate config content
    let selected_modules: Vec<String> = modules
        .into_iter()
        .filter(|m| m.enabled)
        .map(|m| m.id)
        .collect();

    let content = match output_format {
        OutputFormat::Minimal => {
            let mut s = String::new();
            s.push_str("# omnifetch configuration file\n");
            s.push_str("# Generated by omnifetch interactive config generator\n\n");
            s.push_str(&format!("logo = \"{}\"\n", logo_type.to_config_str()));
            if logo_pos == LogoPos::Top {
                s.push_str("logo_top = true\n");
            }
            s.push_str("\nmodules = [\n");
            for m in &selected_modules {
                s.push_str(&format!("    \"{m}\",\n"));
            }
            s.push_str("]\n");
            s
        }
        OutputFormat::Full => {
            let mut template = crate::config::DEFAULT_CONFIG_TEMPLATE.to_string();
            // Replace logo
            template = template.replacen(
                "logo = \"auto\"",
                &format!("logo = \"{}\"", logo_type.to_config_str()),
                1,
            );
            if logo_pos == LogoPos::Top {
                template = template.replacen(
                    &format!("logo = \"{}\"\n", logo_type.to_config_str()),
                    &format!(
                        "logo = \"{}\"\nlogo_top = true\n",
                        logo_type.to_config_str()
                    ),
                    1,
                );
            }
            // Replace modules list
            let mut mod_str = String::from("modules = [\n");
            for m in &selected_modules {
                mod_str.push_str(&format!("    \"{m}\",\n"));
            }
            mod_str.push(']');

            if let Some(start) = template.find("modules = [")
                && let Some(end) = template[start..].find(']')
            {
                template.replace_range(start..start + end + 1, &mod_str);
            }
            template
        }
    };

    if let Some(parent) = target_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return Err(format!(
            "failed to create directory {}: {e}",
            parent.display()
        ));
    }

    if let Err(e) = fs::write(target_path, content) {
        return Err(format!("failed to write {}: {e}", target_path.display()));
    }

    println!("Generated config file written to {}", target_path.display());
    Ok(())
}
