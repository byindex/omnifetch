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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeOpt {
    Default,
    Catppuccin,
    TokyoNight,
    Nord,
    Gruvbox,
    Dracula,
    RosePine,
}

impl ThemeOpt {
    pub fn next(self) -> Self {
        match self {
            ThemeOpt::Default => ThemeOpt::Catppuccin,
            ThemeOpt::Catppuccin => ThemeOpt::TokyoNight,
            ThemeOpt::TokyoNight => ThemeOpt::Nord,
            ThemeOpt::Nord => ThemeOpt::Gruvbox,
            ThemeOpt::Gruvbox => ThemeOpt::Dracula,
            ThemeOpt::Dracula => ThemeOpt::RosePine,
            ThemeOpt::RosePine => ThemeOpt::Default,
        }
    }

    pub fn to_config_str(self) -> Option<&'static str> {
        match self {
            ThemeOpt::Default => None,
            ThemeOpt::Catppuccin => Some("catppuccin"),
            ThemeOpt::TokyoNight => Some("tokyo-night"),
            ThemeOpt::Nord => Some("nord"),
            ThemeOpt::Gruvbox => Some("gruvbox"),
            ThemeOpt::Dracula => Some("dracula"),
            ThemeOpt::RosePine => Some("rose-pine"),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ThemeOpt::Default => "default",
            ThemeOpt::Catppuccin => "catppuccin",
            ThemeOpt::TokyoNight => "tokyo-night",
            ThemeOpt::Nord => "nord",
            ThemeOpt::Gruvbox => "gruvbox",
            ThemeOpt::Dracula => "dracula",
            ThemeOpt::RosePine => "rose-pine",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            ThemeOpt::Default => "Built-in default styling",
            ThemeOpt::Catppuccin => "Soothing pastel mocha palette",
            ThemeOpt::TokyoNight => "Clean, dark blue Tokyo Night theme",
            ThemeOpt::Nord => "Arctic, north-bluish clean palette",
            ThemeOpt::Gruvbox => "Retro groove warm earthy colors",
            ThemeOpt::Dracula => "High-contrast dark theme with purple/pink",
            ThemeOpt::RosePine => "All natural pine, warm gold, and love",
        }
    }

    pub fn swatch(self) -> &'static str {
        match self {
            ThemeOpt::Default => "\x1b[36m●\x1b[m",
            ThemeOpt::Catppuccin => "\x1b[38;2;137;180;250m●\x1b[m",
            ThemeOpt::TokyoNight => "\x1b[38;2;122;162;247m●\x1b[m",
            ThemeOpt::Nord => "\x1b[38;2;136;192;208m●\x1b[m",
            ThemeOpt::Gruvbox => "\x1b[38;2;250;189;47m●\x1b[m",
            ThemeOpt::Dracula => "\x1b[38;2;189;147;249m●\x1b[m",
            ThemeOpt::RosePine => "\x1b[38;2;234;154;151m●\x1b[m",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyColorOpt {
    Default,
    Cyan,
    Green,
    Yellow,
    Blue,
    Magenta,
    Red,
    White,
    Hex(String),
}

impl KeyColorOpt {
    pub fn next(&self) -> Self {
        match self {
            KeyColorOpt::Default => KeyColorOpt::Cyan,
            KeyColorOpt::Cyan => KeyColorOpt::Green,
            KeyColorOpt::Green => KeyColorOpt::Yellow,
            KeyColorOpt::Yellow => KeyColorOpt::Blue,
            KeyColorOpt::Blue => KeyColorOpt::Magenta,
            KeyColorOpt::Magenta => KeyColorOpt::Red,
            KeyColorOpt::Red => KeyColorOpt::White,
            KeyColorOpt::White => KeyColorOpt::Default,
            KeyColorOpt::Hex(_) => KeyColorOpt::Default,
        }
    }

    pub fn to_config_str(&self) -> Option<String> {
        match self {
            KeyColorOpt::Default => None,
            KeyColorOpt::Cyan => Some("cyan".into()),
            KeyColorOpt::Green => Some("green".into()),
            KeyColorOpt::Yellow => Some("yellow".into()),
            KeyColorOpt::Blue => Some("blue".into()),
            KeyColorOpt::Magenta => Some("magenta".into()),
            KeyColorOpt::Red => Some("red".into()),
            KeyColorOpt::White => Some("white".into()),
            KeyColorOpt::Hex(h) => Some(h.clone()),
        }
    }

    pub fn name(&self) -> String {
        match self {
            KeyColorOpt::Default => "default".into(),
            KeyColorOpt::Cyan => "cyan".into(),
            KeyColorOpt::Green => "green".into(),
            KeyColorOpt::Yellow => "yellow".into(),
            KeyColorOpt::Blue => "blue".into(),
            KeyColorOpt::Magenta => "magenta".into(),
            KeyColorOpt::Red => "red".into(),
            KeyColorOpt::White => "white".into(),
            KeyColorOpt::Hex(h) => h.clone(),
        }
    }

    pub fn swatch(&self) -> String {
        match self {
            KeyColorOpt::Default => "\x1b[36m●\x1b[m".into(),
            KeyColorOpt::Cyan => "\x1b[36m●\x1b[m".into(),
            KeyColorOpt::Green => "\x1b[32m●\x1b[m".into(),
            KeyColorOpt::Yellow => "\x1b[33m●\x1b[m".into(),
            KeyColorOpt::Blue => "\x1b[34m●\x1b[m".into(),
            KeyColorOpt::Magenta => "\x1b[35m●\x1b[m".into(),
            KeyColorOpt::Red => "\x1b[31m●\x1b[m".into(),
            KeyColorOpt::White => "\x1b[37m●\x1b[m".into(),
            KeyColorOpt::Hex(h) => {
                let clean = h.trim().strip_prefix('#').unwrap_or(h.trim());
                if let Some(crate::style::Color::Rgb(r, g, b)) = crate::style::Color::parse(clean) {
                    format!("\x1b[38;2;{r};{g};{b}m●\x1b[m")
                } else {
                    "●".into()
                }
            }
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

pub const MODULES_JSON: &str = include_str!("../assets/modules.json");

pub fn parse_modules_json(content: &str) -> Vec<ModuleItem> {
    let mut list = Vec::new();
    if let Ok(crate::json::JsonValue::Array(arr)) = crate::json::parse(content) {
        for item in arr {
            let id = item
                .get("id")
                .and_then(crate::json::JsonValue::as_str)
                .unwrap_or("");
            let name = item
                .get("name")
                .and_then(crate::json::JsonValue::as_str)
                .unwrap_or("");
            let desc = item
                .get("desc")
                .and_then(crate::json::JsonValue::as_str)
                .unwrap_or("");
            let enabled = item
                .get("enabled")
                .and_then(crate::json::JsonValue::as_bool)
                .unwrap_or(false);
            let is_special = item
                .get("is_special")
                .and_then(crate::json::JsonValue::as_bool)
                .unwrap_or(false);

            if !id.is_empty() {
                list.push(ModuleItem {
                    id: id.to_string(),
                    name: name.to_string(),
                    desc: desc.to_string(),
                    enabled,
                    is_special,
                });
            }
        }
    }
    list
}

fn initial_modules() -> Vec<ModuleItem> {
    parse_modules_json(MODULES_JSON)
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
    Tab,
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
                    b'Z' => return Key::Tab, // Shift-Tab
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

fn read_key_timeout(timeout_ms: i32) -> Key {
    let mut pfd = libc::pollfd {
        fd: libc::STDIN_FILENO,
        events: libc::POLLIN,
        revents: 0,
    };
    let pr = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
    if pr <= 0 {
        return Key::None;
    }

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
        0x09 => Key::Tab,
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

fn read_key() -> Key {
    read_key_timeout(-1)
}

pub fn module_matches(m: &ModuleItem, query: &str) -> bool {
    if query.is_empty() {
        return false;
    }
    let q = query.to_lowercase();
    // 1. Match module name or ID (case-insensitive substring)
    if m.name.to_lowercase().contains(&q) || m.id.to_lowercase().contains(&q) {
        return true;
    }
    // 2. Useful aliases
    if (q == "pkg" || q == "pkgs") && m.id == "packages" {
        return true;
    }
    if q == "ram" && (m.id == "memory" || m.id == "physicalmemory") {
        return true;
    }
    // 3. Category match: only for queries of 3+ letters that match the start of category title
    if q.len() >= 3 {
        let cat = crate::render::module_category(&m.id);
        if let Some(title) = crate::render::category_title(cat)
            && title.to_lowercase().starts_with(&q)
        {
            return true;
        }
    }
    false
}

pub fn find_next_match(
    modules: &[ModuleItem],
    query: &str,
    start_idx: usize,
    forward: bool,
) -> Option<usize> {
    if query.is_empty() || modules.is_empty() {
        return None;
    }
    let len = modules.len();
    for step in 1..=len {
        let idx = if forward {
            (start_idx + step) % len
        } else {
            (start_idx + len - (step % len)) % len
        };
        if module_matches(&modules[idx], query) {
            return Some(idx);
        }
    }
    if module_matches(&modules[start_idx % len], query) {
        return Some(start_idx % len);
    }
    None
}

pub fn run_interactive(target_path: &Path) -> Result<(), String> {
    let _raw = RawMode::enter().ok_or_else(|| "Failed to enter raw terminal mode".to_string())?;

    let mut logo_type = LogoType::Default;
    let mut logo_pos = LogoPos::Left;
    let mut output_format = OutputFormat::Minimal;
    let mut theme_opt = ThemeOpt::Default;
    let mut key_color_opt = KeyColorOpt::Default;
    let mut modules = initial_modules();

    let mut cursor_col: usize = 0;
    let mut cursor_row: usize = 0;
    let mut scroll_row: usize = 0;
    let num_cols = 4;

    let mut search_mode = false;
    let mut search_query = String::new();
    let mut hex_input_mode = false;
    let mut hex_input_buffer = String::new();
    let mut reset_confirm = false;
    let mut status_msg: Option<(String, std::time::Instant)> = None;

    loop {
        let rows_per_col = modules.len().div_ceil(num_cols).max(1);
        let size = crate::sys::terminal_size();
        let term_cols = size.map_or(80, |s| s.cols as usize).max(60);
        let term_rows = size.map_or(24, |s| s.rows as usize).max(16);

        let visible_rows = (term_rows.saturating_sub(13)).clamp(4, rows_per_col);
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
        out.push_str("  \x1b[1;4mL\x1b[24mogo type:\x1b[m      ");
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

        // Theme
        out.push_str(&format!(
            "  \x1b[1;4mT\x1b[24mheme:\x1b[m          [ < \x1b[1m{}\x1b[m > ]  {} \x1b[2m- {}\x1b[m\x1b[K\r\n",
            theme_opt.name(),
            theme_opt.swatch(),
            theme_opt.desc()
        ));

        // Key color
        let color_desc = match &key_color_opt {
            KeyColorOpt::Default => "theme or default cyan (x: cycle, H: custom hex)",
            KeyColorOpt::Hex(_) => "custom hex color (x: cycle, H: edit hex)",
            _ => "explicit key color (x: cycle, H: custom hex)",
        };
        out.push_str(&format!(
            "  \x1b[1mColor (\x1b[4mx\x1b[24m/\x1b[4mH\x1b[24m):\x1b[m    [ < \x1b[1m{}\x1b[m > ]  {} \x1b[2m- {}\x1b[m\x1b[K\r\n",
            key_color_opt.name(),
            key_color_opt.swatch(),
            color_desc
        ));

        // Output
        out.push_str("  \x1b[1;4mO\x1b[24mutput:\x1b[m         ");
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
                    let is_match = !search_query.is_empty() && module_matches(item, &search_query);

                    let mut text = String::new();
                    if item.is_special {
                        if item.enabled {
                            if is_focused {
                                let name_style = if is_match { "\x1b[7;33m" } else { "\x1b[7;36m" };
                                text.push_str(&format!(
                                    "\x1b[7;36m[-]\x1b[m {name_style}{}\x1b[m",
                                    item.name
                                ));
                            } else {
                                let name_style = if is_match { "\x1b[1;33m" } else { "\x1b[36m" };
                                text.push_str(&format!(
                                    "\x1b[36m[-]\x1b[m {name_style}{}\x1b[m",
                                    item.name
                                ));
                            }
                        } else if is_focused {
                            let name_style = if is_match { "\x1b[7;33m" } else { "\x1b[7m" };
                            text.push_str(&format!(
                                "\x1b[7m[ ]\x1b[m {name_style}{}\x1b[m",
                                item.name
                            ));
                        } else {
                            let name_style = if is_match { "\x1b[1;33m" } else { "" };
                            text.push_str(&format!("[ ] {name_style}{}\x1b[m", item.name));
                        }
                    } else if item.enabled {
                        if is_focused {
                            let name_style = if is_match { "\x1b[7;33m" } else { "\x1b[7;32m" };
                            text.push_str(&format!(
                                "\x1b[7;32m[x]\x1b[m {name_style}{}\x1b[m",
                                item.name
                            ));
                        } else {
                            let name_style = if is_match { "\x1b[1;33m" } else { "\x1b[32m" };
                            text.push_str(&format!(
                                "\x1b[32m[x]\x1b[m {name_style}{}\x1b[m",
                                item.name
                            ));
                        }
                    } else if is_focused {
                        let name_style = if is_match { "\x1b[7;33m" } else { "\x1b[7m" };
                        text.push_str(&format!("\x1b[7m[ ]\x1b[m {name_style}{}\x1b[m", item.name));
                    } else {
                        let name_style = if is_match { "\x1b[1;33m" } else { "" };
                        text.push_str(&format!("[ ] {name_style}{}\x1b[m", item.name));
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

        if hex_input_mode {
            let preview_hex = {
                let clean = hex_input_buffer
                    .trim()
                    .strip_prefix('#')
                    .unwrap_or(hex_input_buffer.trim());
                if let Some(crate::style::Color::Rgb(r, g, b)) = crate::style::Color::parse(clean) {
                    format!("  \x1b[38;2;{r};{g};{b}m●\x1b[m \x1b[32m✔ valid\x1b[m")
                } else if clean.len() == 6 {
                    "  \x1b[31m✖ invalid hex\x1b[m".to_string()
                } else {
                    format!("  \x1b[2m({}/6 hex digits)\x1b[m", clean.len())
                }
            };
            out.push_str(&format!(
                "  \x1b[1;35mCustom Hex Color:\x1b[m #{hex_input_buffer}\x1b[7m \x1b[m{preview_hex}\x1b[K\r\n"
            ));
            out.push_str("  \x1b[2mEnter: apply  Esc: cancel  Backspace: delete  (e.g. ff007f or 7928ca)\x1b[m\x1b[K");
        } else if search_mode {
            let match_count = modules
                .iter()
                .filter(|m| module_matches(m, &search_query))
                .count();
            let count_info = if !search_query.is_empty() {
                format!(" \x1b[2m({match_count} matches)\x1b[m")
            } else {
                String::new()
            };
            out.push_str(&format!(
                "  \x1b[1;33mSearch:\x1b[m /{search_query}\x1b[7m \x1b[m{count_info}\x1b[K\r\n"
            ));
            out.push_str("  \x1b[2mEnter: done  Esc: cancel  Tab/↓: next  ↑: prev  type to filter\x1b[m\x1b[K");
        } else if reset_confirm {
            out.push_str("  \x1b[1;31mReset all modules and settings to default?\x1b[m  \x1b[1;32m[y]\x1b[m Yes  \x1b[1;31m[n/Esc]\x1b[m Cancel\x1b[K\r\n");
            out.push_str("  \x1b[2mPress 'y' to reset modules to initial defaults, or any other key to cancel\x1b[m\x1b[K");
        } else {
            // Bottom line 1: prominent status message banner if active, otherwise module description
            let first_line = if let Some((ref msg, time)) = status_msg
                && time.elapsed().as_millis() < 2500
            {
                format!("  {msg}")
            } else {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx < modules.len() {
                    let m = &modules[cur_idx];
                    let badge = if m.is_special || m.id == "break" {
                        "\x1b[1;90m[Layout]\x1b[m".to_string()
                    } else if m.id == "separator" || m.id == "title" {
                        "\x1b[1;90m[Header]\x1b[m".to_string()
                    } else {
                        let cat = crate::render::module_category(&m.id);
                        match cat {
                            1 => "\x1b[1;36m[System]\x1b[m".to_string(),
                            2 => "\x1b[1;35m[Visual]\x1b[m".to_string(),
                            3 => "\x1b[1;33m[Hardware]\x1b[m".to_string(),
                            4 => "\x1b[1;34m[Network]\x1b[m".to_string(),
                            5 => "\x1b[1;32m[Devices]\x1b[m".to_string(),
                            6 => "\x1b[1;37m[Colors]\x1b[m".to_string(),
                            7 => "\x1b[1;36m[Quote]\x1b[m".to_string(),
                            _ => "\x1b[1;90m[Other]\x1b[m".to_string(),
                        }
                    };
                    let search_indicator = if !search_query.is_empty() {
                        format!("  \x1b[2m(filter: /{search_query})\x1b[m")
                    } else {
                        String::new()
                    };
                    format!("  {badge} {}{search_indicator}", m.desc)
                } else {
                    String::new()
                }
            };
            out.push_str(&format!("{first_line}\x1b[K\r\n"));

            // Hotkeys help
            out.push_str("  \x1b[36m↑/↓\x1b[m move  \x1b[36m←/→\x1b[m col  \x1b[36mSpace\x1b[m toggle  \x1b[36m/\x1b[m search  \x1b[36m[/]\x1b[m reorder  \x1b[36mt\x1b[m theme  \x1b[36mx/H\x1b[m color/hex\x1b[K\r\n");
            out.push_str("  \x1b[36mr\x1b[m reset  \x1b[36mf\x1b[m all  \x1b[36mb/B\x1b[m break/sep  \x1b[36ml\x1b[m logo  \x1b[36mp\x1b[m pos  \x1b[36mo\x1b[m format  \x1b[36mv\x1b[m preview  \x1b[36ms/Enter\x1b[m save  \x1b[36mq\x1b[m quit\x1b[K");
        }

        print!("{out}");
        let _ = io::stdout().flush();

        if reset_confirm {
            match read_key() {
                Key::Char('y') | Key::Char('Y') | Key::Char('н') | Key::Char('Н') => {
                    modules = initial_modules();
                    logo_type = LogoType::Default;
                    logo_pos = LogoPos::Left;
                    output_format = OutputFormat::Minimal;
                    theme_opt = ThemeOpt::Default;
                    key_color_opt = KeyColorOpt::Default;
                    cursor_col = 0;
                    cursor_row = 0;
                    scroll_row = 0;
                    search_query.clear();
                    status_msg = Some((
                        "\x1b[1;97;42m ✔ Reset to defaults! \x1b[m  \x1b[1;32mAll modules and settings restored to initial state\x1b[m".to_string(),
                        std::time::Instant::now(),
                    ));
                }
                Key::Ctrl('c') => {
                    print!("\x1b[2J\x1b[H\x1b[?25h");
                    let _ = io::stdout().flush();
                    return Ok(());
                }
                _ => {
                    status_msg = Some((
                        "\x1b[1;30;43m ✖ Reset cancelled \x1b[m  \x1b[1;33mNo changes made to configuration\x1b[m".to_string(),
                        std::time::Instant::now(),
                    ));
                }
            }
            reset_confirm = false;
            continue;
        }

        // Read user input with timeout so that temporary status banners automatically disappear
        let timeout_ms = if let Some((_, time)) = status_msg {
            let elapsed_ms = time.elapsed().as_millis();
            if elapsed_ms >= 2500 {
                status_msg = None;
                -1
            } else {
                (2500 - elapsed_ms).clamp(1, 2500) as i32
            }
        } else {
            -1
        };

        let key = read_key_timeout(timeout_ms);
        if key == Key::None {
            if let Some((_, time)) = status_msg
                && time.elapsed().as_millis() >= 2500
            {
                status_msg = None;
            }
            continue;
        }

        if hex_input_mode {
            match key {
                Key::Esc => {
                    hex_input_mode = false;
                    hex_input_buffer.clear();
                }
                Key::Enter => {
                    let clean = hex_input_buffer
                        .trim()
                        .strip_prefix('#')
                        .unwrap_or(hex_input_buffer.trim());
                    if let Some(crate::style::Color::Rgb(r, g, b)) =
                        crate::style::Color::parse(clean)
                    {
                        key_color_opt = KeyColorOpt::Hex(format!("#{clean}"));
                        status_msg = Some((
                            format!(
                                "\x1b[1;97;42m ✔ Color set \x1b[m  \x1b[1;38;2;{r};{g};{b}m●\x1b[m Custom hex color #{clean} applied"
                            ),
                            std::time::Instant::now(),
                        ));
                        hex_input_mode = false;
                        hex_input_buffer.clear();
                    } else if clean.is_empty() {
                        hex_input_mode = false;
                    } else {
                        status_msg = Some((
                            "\x1b[1;30;41m ✖ Invalid hex \x1b[m  \x1b[1;31mPlease enter 6 hex digits (e.g. ff007f)\x1b[m".to_string(),
                            std::time::Instant::now(),
                        ));
                    }
                }
                Key::Backspace | Key::Delete => {
                    hex_input_buffer.pop();
                }
                Key::Char(ch)
                    if (ch.is_ascii_hexdigit() || ch == '#') && hex_input_buffer.len() < 7 =>
                {
                    let ch_clean = ch.to_ascii_lowercase();
                    if ch_clean != '#' || hex_input_buffer.is_empty() {
                        hex_input_buffer.push(ch_clean);
                    }
                }
                _ => {}
            }
            continue;
        }

        if search_mode {
            match key {
                Key::Esc => {
                    search_mode = false;
                    search_query.clear();
                }
                Key::Enter => {
                    search_mode = false;
                }
                Key::Backspace | Key::Delete => {
                    search_query.pop();
                    if !search_query.is_empty() {
                        let cur_idx = cursor_col * rows_per_col + cursor_row;
                        let target = if module_matches(&modules[cur_idx], &search_query) {
                            Some(cur_idx)
                        } else {
                            find_next_match(&modules, &search_query, cur_idx, true)
                        };
                        if let Some(t) = target {
                            cursor_col = (t / rows_per_col).min(num_cols - 1);
                            cursor_row = t % rows_per_col;
                        }
                    }
                }
                Key::Tab | Key::Down => {
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    if let Some(next_idx) = find_next_match(&modules, &search_query, cur_idx, true)
                    {
                        cursor_col = (next_idx / rows_per_col).min(num_cols - 1);
                        cursor_row = next_idx % rows_per_col;
                    }
                }
                Key::Up => {
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    if let Some(prev_idx) = find_next_match(&modules, &search_query, cur_idx, false)
                    {
                        cursor_col = (prev_idx / rows_per_col).min(num_cols - 1);
                        cursor_row = prev_idx % rows_per_col;
                    }
                }
                Key::Char(ch) => {
                    search_query.push(ch);
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    let target = if module_matches(&modules[cur_idx], &search_query) {
                        Some(cur_idx)
                    } else {
                        find_next_match(&modules, &search_query, cur_idx, true)
                    };
                    if let Some(t) = target {
                        cursor_col = (t / rows_per_col).min(num_cols - 1);
                        cursor_row = t % rows_per_col;
                    }
                }
                _ => {}
            }
            continue;
        }

        // Handle user input
        match key {
            Key::Char('/') | Key::Char('?') => {
                search_mode = true;
                search_query.clear();
            }
            Key::Char('n') => {
                if !search_query.is_empty() {
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    if let Some(next_idx) = find_next_match(&modules, &search_query, cur_idx, true)
                    {
                        cursor_col = (next_idx / rows_per_col).min(num_cols - 1);
                        cursor_row = next_idx % rows_per_col;
                    }
                }
            }
            Key::Char('N') => {
                if !search_query.is_empty() {
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    if let Some(prev_idx) = find_next_match(&modules, &search_query, cur_idx, false)
                    {
                        cursor_col = (prev_idx / rows_per_col).min(num_cols - 1);
                        cursor_row = prev_idx % rows_per_col;
                    }
                }
            }
            Key::Char('v') | Key::Char('V') | Key::Char('м') | Key::Char('М') => {
                let temp_path = std::env::temp_dir()
                    .join(format!("omnifetch_prev_{}.toml", std::process::id()));
                let selected_modules: Vec<String> = modules
                    .iter()
                    .filter(|m| m.enabled)
                    .map(|m| m.id.clone())
                    .collect();

                let mut preview_cfg = String::new();
                if let Some(th) = theme_opt.to_config_str() {
                    preview_cfg.push_str(&format!("theme = \"{th}\"\n"));
                }
                preview_cfg.push_str(&format!("logo = \"{}\"\n", logo_type.to_config_str()));
                if logo_pos == LogoPos::Top {
                    preview_cfg.push_str("logo_top = true\n");
                }
                preview_cfg.push_str("\nmodules = [\n");
                for m in &selected_modules {
                    preview_cfg.push_str(&format!("    \"{m}\",\n"));
                }
                preview_cfg.push_str("]\n");
                if let Some(kc) = key_color_opt.to_config_str() {
                    preview_cfg.push_str("\n[style]\n");
                    preview_cfg.push_str(&format!("key_color = \"{kc}\"\n"));
                }

                if let Ok(()) = fs::write(&temp_path, &preview_cfg) {
                    let exe = std::env::current_exe()
                        .unwrap_or_else(|_| std::path::PathBuf::from("omnifetch"));
                    let output = std::process::Command::new(exe)
                        .arg("--config")
                        .arg(&temp_path)
                        .output();
                    let _ = fs::remove_file(&temp_path);

                    let preview_body = match output {
                        Ok(out) => {
                            let stdout = String::from_utf8_lossy(&out.stdout).replace('\n', "\r\n");
                            let stderr = String::from_utf8_lossy(&out.stderr).replace('\n', "\r\n");
                            if stdout.is_empty() && !stderr.is_empty() {
                                stderr
                            } else {
                                stdout
                            }
                        }
                        Err(e) => format!("Failed to run preview: {e}\r\n"),
                    };

                    print!(
                        "\x1b[2J\x1b[H\x1b[1;36m=== Live Omnifetch Preview ===\x1b[m\r\n\r\n{}\r\n\x1b[7;36m[ Press any key to return to editor ]\x1b[m",
                        preview_body
                    );
                    let _ = io::stdout().flush();
                    let _ = read_key();
                }
            }
            Key::Char('r') | Key::Char('R') | Key::Char('к') | Key::Char('К') => {
                reset_confirm = true;
            }
            Key::Up | Key::Char('k') | Key::Char('л') | Key::Char('Л') => {
                status_msg = None;
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
            Key::Down | Key::Char('j') | Key::Char('о') | Key::Char('О') => {
                status_msg = None;
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
            Key::Left | Key::Char('h') | Key::Char('р') | Key::Char('Р') => {
                status_msg = None;
                if cursor_col > 0 {
                    cursor_col -= 1;
                    let cur_idx = cursor_col * rows_per_col + cursor_row;
                    if cur_idx >= modules.len() {
                        cursor_row = modules.len().saturating_sub(1) % rows_per_col;
                    }
                }
            }
            Key::Right => {
                status_msg = None;
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
            Key::Char('[')
            | Key::Char('-')
            | Key::Char(',')
            | Key::Char('<')
            | Key::Char('u')
            | Key::Char('K')
            | Key::Char('х')
            | Key::Char('Х') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx > 0 && cur_idx < modules.len() {
                    modules.swap(cur_idx, cur_idx - 1);
                    let target = cur_idx - 1;
                    cursor_col = (target / rows_per_col).min(num_cols - 1);
                    cursor_row = target % rows_per_col;
                }
            }
            Key::Char(']')
            | Key::Char('=')
            | Key::Char('+')
            | Key::Char('.')
            | Key::Char('>')
            | Key::Char('m')
            | Key::Char('J')
            | Key::Char('ъ')
            | Key::Char('Ъ') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                if cur_idx + 1 < modules.len() {
                    modules.swap(cur_idx, cur_idx + 1);
                    let target = cur_idx + 1;
                    cursor_col = (target / rows_per_col).min(num_cols - 1);
                    cursor_row = target % rows_per_col;
                }
            }
            Key::Char('c') | Key::Char('C') | Key::Char('с') | Key::Char('С') => {
                let cur_idx = cursor_col * rows_per_col + cursor_row;
                let selected_id = if cur_idx < modules.len() {
                    Some(modules[cur_idx].id.clone())
                } else {
                    None
                };

                sort_modules_by_category(&mut modules);

                if let Some(id) = selected_id
                    && let Some(new_pos) = modules.iter().position(|m| m.id == id)
                {
                    cursor_col = (new_pos / rows_per_col).min(num_cols - 1);
                    cursor_row = new_pos % rows_per_col;
                }
                status_msg = Some((
                    "\x1b[1;97;44m ⇅ Categorized \x1b[m  \x1b[1;34mModules grouped and sorted by categories\x1b[m".to_string(),
                    std::time::Instant::now(),
                ));
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
            Key::Char('t') | Key::Char('T') | Key::Char('е') | Key::Char('Е') => {
                theme_opt = theme_opt.next();
                status_msg = Some((
                    format!(
                        "\x1b[1;97;45m 🎨 Theme \x1b[m  Theme set to \x1b[1m{}\x1b[m {}",
                        theme_opt.name(),
                        theme_opt.swatch()
                    ),
                    std::time::Instant::now(),
                ));
            }
            Key::Char('x') | Key::Char('ч') => {
                key_color_opt = key_color_opt.next();
                status_msg = Some((
                    format!(
                        "\x1b[1;97;46m 🌈 Key Color \x1b[m  Color set to \x1b[1m{}\x1b[m {}",
                        key_color_opt.name(),
                        key_color_opt.swatch()
                    ),
                    std::time::Instant::now(),
                ));
            }
            Key::Char('H') | Key::Char('X') | Key::Char('Ч') | Key::Char('#') => {
                hex_input_mode = true;
                hex_input_buffer.clear();
            }
            Key::Char('l') | Key::Char('L') | Key::Char('д') | Key::Char('Д') => {
                logo_type = logo_type.next();
            }
            Key::Char('p') | Key::Char('P') | Key::Char('з') | Key::Char('З') => {
                logo_pos = logo_pos.next();
            }
            Key::Char('o') | Key::Char('O') | Key::Char('щ') | Key::Char('Щ') => {
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
            Key::Char('s') | Key::Char('S') | Key::Char('ы') | Key::Char('Ы') | Key::Enter => {
                // Save and exit
                break;
            }
            Key::Char('q')
            | Key::Char('Q')
            | Key::Char('й')
            | Key::Char('Й')
            | Key::Esc
            | Key::Ctrl('c') => {
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
            if let Some(th) = theme_opt.to_config_str() {
                s.push_str(&format!("theme = \"{th}\"\n"));
            }
            s.push_str(&format!("logo = \"{}\"\n", logo_type.to_config_str()));
            if logo_pos == LogoPos::Top {
                s.push_str("logo_top = true\n");
            }
            s.push_str("\nmodules = [\n");
            for m in &selected_modules {
                s.push_str(&format!("    \"{m}\",\n"));
            }
            s.push_str("]\n");
            if let Some(kc) = key_color_opt.to_config_str() {
                s.push_str("\n[style]\n");
                s.push_str(&format!("key_color = \"{kc}\"\n"));
            }
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
            if let Some(th) = theme_opt.to_config_str() {
                template = template.replacen(
                    "# Disable color output\n# color = false\n",
                    &format!("# Disable color output\n# color = false\n\n# Color theme\ntheme = \"{th}\"\n"),
                    1,
                );
            }
            if let Some(kc) = key_color_opt.to_config_str() {
                template = template.replacen(
                    "# key_color = \"cyan\"",
                    &format!("key_color = \"{kc}\""),
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

pub fn sort_modules_by_category(modules: &mut [ModuleItem]) {
    modules.sort_by_key(|m| {
        if m.id == "title" {
            0
        } else if m.id == "separator" {
            1
        } else if m.id == "break" {
            200
        } else {
            match crate::render::module_category(&m.id) {
                0 => 2,
                1 => 10, // System
                2 => 20, // Visual
                3 => 30, // Hardware
                4 => 40, // Network
                5 => 50, // Devices
                6 => 60, // Colors
                7 => 70, // Quote
                _ => 100,
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_modules_by_category() {
        let mut mods = vec![
            ModuleItem {
                id: "cpu".into(),
                name: "CPU".into(),
                desc: "".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "os".into(),
                name: "OS".into(),
                desc: "".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "title".into(),
                name: "Title".into(),
                desc: "".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "weather".into(),
                name: "Weather".into(),
                desc: "".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "colors".into(),
                name: "Colors".into(),
                desc: "".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "wm".into(),
                name: "WM".into(),
                desc: "".into(),
                enabled: true,
                is_special: false,
            },
        ];
        sort_modules_by_category(&mut mods);
        let ids: Vec<&str> = mods.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["title", "os", "wm", "cpu", "weather", "colors"]);
    }

    #[test]
    fn test_module_matches_and_find_next_match() {
        let mods = vec![
            ModuleItem {
                id: "cpu".into(),
                name: "CPU".into(),
                desc: "Processor model".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "cputemp".into(),
                name: "CPUTemp".into(),
                desc: "CPU temperature".into(),
                enabled: true,
                is_special: false,
            },
            ModuleItem {
                id: "os".into(),
                name: "OS".into(),
                desc: "Operating system".into(),
                enabled: true,
                is_special: false,
            },
        ];
        assert!(module_matches(&mods[0], "cpu"));
        assert!(module_matches(&mods[1], "cpu"));
        assert!(!module_matches(&mods[2], "cpu"));
        assert!(module_matches(&mods[2], "system")); // matches category System

        // Test precision: "a" shouldn't match "cpu" or "cputemp"
        assert!(!module_matches(&mods[0], "a"));
        assert!(!module_matches(&mods[1], "a"));

        // Test "pac": matches packages, but does NOT match btrfs or cputemp
        let btrfs = ModuleItem {
            id: "btrfs".into(),
            name: "Btrfs".into(),
            desc: "Allocated and used space".into(),
            enabled: true,
            is_special: false,
        };
        let packages = ModuleItem {
            id: "packages".into(),
            name: "Packages".into(),
            desc: "Installed packages count".into(),
            enabled: true,
            is_special: false,
        };
        assert!(!module_matches(&btrfs, "pac"));
        assert!(!module_matches(&mods[1], "pac"));
        assert!(module_matches(&packages, "pac"));
        assert!(module_matches(&packages, "pkg"));

        assert_eq!(find_next_match(&mods, "cpu", 0, true), Some(1));
        assert_eq!(find_next_match(&mods, "cpu", 1, true), Some(0));
        assert_eq!(find_next_match(&mods, "cpu", 0, false), Some(1));
    }

    #[test]
    fn test_initial_modules_from_embedded_json() {
        let mods = initial_modules();
        assert_eq!(mods.len(), 88);
        assert_eq!(mods[0].id, "title");
        assert!(mods.iter().any(|m| m.id == "colors"));
        for &id in crate::modules::ALL_MODULE_IDS {
            assert!(
                mods.iter().any(|m| m.id == id),
                "embedded modules.json is missing module id: {id}"
            );
        }
    }

    #[test]
    fn test_theme_opt_cycle_and_conversion() {
        let mut t = ThemeOpt::Default;
        assert_eq!(t.to_config_str(), None);
        assert_eq!(t.name(), "default");

        t = t.next();
        assert_eq!(t, ThemeOpt::Catppuccin);
        assert_eq!(t.to_config_str(), Some("catppuccin".into()));

        t = t.next();
        assert_eq!(t, ThemeOpt::TokyoNight);
        t = t.next();
        assert_eq!(t, ThemeOpt::Nord);
        t = t.next();
        assert_eq!(t, ThemeOpt::Gruvbox);
        t = t.next();
        assert_eq!(t, ThemeOpt::Dracula);
        t = t.next();
        assert_eq!(t, ThemeOpt::RosePine);
        t = t.next();
        assert_eq!(t, ThemeOpt::Default);
    }

    #[test]
    fn test_key_color_opt_cycle_and_hex() {
        let mut c = KeyColorOpt::Default;
        assert_eq!(c.to_config_str(), None);
        assert_eq!(c.name(), "default");

        c = c.next();
        assert_eq!(c, KeyColorOpt::Cyan);
        assert_eq!(c.to_config_str(), Some("cyan".into()));

        c = KeyColorOpt::Hex("#ff007f".into());
        assert_eq!(c.to_config_str(), Some("#ff007f".into()));
        assert_eq!(c.name(), "#ff007f");
        assert!(c.swatch().contains("\x1b[38;2;255;0;127m"));

        c = c.next();
        assert_eq!(c, KeyColorOpt::Default);
    }
}
