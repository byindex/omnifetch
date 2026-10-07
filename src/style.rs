#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
}

impl Color {
    fn code(self) -> Option<u8> {
        match self {
            Color::Default => Some(39),
            Color::Black => Some(30),
            Color::Red => Some(31),
            Color::Green => Some(32),
            Color::Yellow => Some(33),
            Color::Blue => Some(34),
            Color::Magenta => Some(35),
            Color::Cyan => Some(36),
            Color::White => Some(37),
            Color::BrightBlack => Some(90),
            Color::BrightRed => Some(91),
            Color::BrightGreen => Some(92),
            Color::BrightYellow => Some(93),
            Color::BrightBlue => Some(94),
            Color::BrightMagenta => Some(95),
            Color::BrightCyan => Some(96),
            Color::BrightWhite => Some(97),
            Color::Rgb(_, _, _) => None,
        }
    }

    pub fn parse(s: &str) -> Option<Color> {
        let trimmed = s.trim();
        let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }

        match trimmed
            .to_ascii_lowercase()
            .replace(['-', '_'], "")
            .as_str()
        {
            "default" => Some(Color::Default),
            "black" => Some(Color::Black),
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "yellow" => Some(Color::Yellow),
            "blue" => Some(Color::Blue),
            "magenta" | "purple" => Some(Color::Magenta),
            "cyan" => Some(Color::Cyan),
            "white" => Some(Color::White),
            "brightblack" | "gray" | "grey" => Some(Color::BrightBlack),
            "brightred" => Some(Color::BrightRed),
            "brightgreen" => Some(Color::BrightGreen),
            "brightyellow" => Some(Color::BrightYellow),
            "brightblue" => Some(Color::BrightBlue),
            "brightmagenta" => Some(Color::BrightMagenta),
            "brightcyan" => Some(Color::BrightCyan),
            "brightwhite" => Some(Color::BrightWhite),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub key: Color,
    pub title: Color,
    pub value: Color,
    pub bold_key: bool,
    pub bold_title: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            key: Color::Cyan,
            title: Color::BrightBlack,
            value: Color::Default,
            bold_key: true,
            bold_title: true,
        }
    }
}

/// Colours set explicitly in the `[style]` table. A theme replaces the whole
/// `Style`, so these are re-applied on top of it afterwards: a theme plus a
/// custom `key_color` gives the theme with that one colour changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleOverrides {
    pub key: Option<Color>,
    pub title: Option<Color>,
    pub value: Option<Color>,
}

impl StyleOverrides {
    pub fn apply(&self, style: &mut Style) {
        if let Some(c) = self.key {
            style.key = c;
        }
        if let Some(c) = self.title {
            style.title = c;
        }
        if let Some(c) = self.value {
            style.value = c;
        }
    }
}

impl Color {
    /// Foreground escape sequence for this colour (empty for `Default`).
    pub fn fg_escape(self) -> String {
        match self {
            Color::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
            c => match c.code() {
                Some(39) | None => String::new(),
                Some(code) => format!("\x1b[{code}m"),
            },
        }
    }
}

pub struct Painter {
    enabled: bool,
}

impl Painter {
    pub fn new(enabled: bool) -> Self {
        Painter { enabled }
    }

    pub fn paint(&self, text: &str, color: Color, bold: bool) -> String {
        if !self.enabled || text.is_empty() || (color == Color::Default && !bold) {
            return text.to_string();
        }
        let mut s = String::with_capacity(text.len() + 20);
        s.push_str("\x1b[");
        if bold {
            s.push_str("1;");
        }
        match color {
            Color::Rgb(r, g, b) => {
                s.push_str("38;2;");
                push_u8_digits(&mut s, r);
                s.push(';');
                push_u8_digits(&mut s, g);
                s.push(';');
                push_u8_digits(&mut s, b);
                s.push('m');
            }
            c => {
                if let Some(code) = c.code() {
                    push_u8_digits(&mut s, code);
                    s.push('m');
                } else {
                    s.push_str("39m");
                }
            }
        }
        s.push_str(text);
        s.push_str("\x1b[0m");
        s
    }
}

#[inline]
fn push_u8_digits(s: &mut String, mut n: u8) {
    if n >= 100 {
        s.push((b'0' + (n / 100)) as char);
        n %= 100;
        s.push((b'0' + (n / 10)) as char);
        s.push((b'0' + (n % 10)) as char);
    } else if n >= 10 {
        s.push((b'0' + (n / 10)) as char);
        s.push((b'0' + (n % 10)) as char);
    } else {
        s.push((b'0' + n) as char);
    }
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub name: &'static str,
    pub style: Style,
    pub bar_fill: &'static str,
    pub bar_empty: &'static str,
}

pub fn get_theme(name: &str) -> Option<ThemeConfig> {
    match name.to_ascii_lowercase().replace(['-', '_'], "").as_str() {
        "catppuccin" | "catppuccinmocha" | "mocha" => Some(ThemeConfig {
            name: "catppuccin-mocha",
            style: Style {
                key: Color::Rgb(137, 180, 250),
                title: Color::Rgb(205, 214, 244),
                value: Color::Rgb(166, 173, 200),
                bold_key: true,
                bold_title: true,
            },
            bar_fill: "█",
            bar_empty: "░",
        }),
        "tokyonight" | "tokyo" => Some(ThemeConfig {
            name: "tokyo-night",
            style: Style {
                key: Color::Rgb(122, 162, 247),
                title: Color::Rgb(187, 154, 247),
                value: Color::Rgb(192, 202, 245),
                bold_key: true,
                bold_title: true,
            },
            bar_fill: "■",
            bar_empty: " ",
        }),
        "nord" => Some(ThemeConfig {
            name: "nord",
            style: Style {
                key: Color::Rgb(136, 192, 208),
                title: Color::Rgb(129, 161, 193),
                value: Color::Rgb(236, 239, 244),
                bold_key: true,
                bold_title: true,
            },
            bar_fill: "━",
            bar_empty: "─",
        }),
        "gruvbox" => Some(ThemeConfig {
            name: "gruvbox",
            style: Style {
                key: Color::Rgb(250, 189, 47),
                title: Color::Rgb(251, 73, 52),
                value: Color::Rgb(235, 219, 178),
                bold_key: true,
                bold_title: true,
            },
            bar_fill: "█",
            bar_empty: "░",
        }),
        "dracula" => Some(ThemeConfig {
            name: "dracula",
            style: Style {
                key: Color::Rgb(189, 147, 249),
                title: Color::Rgb(255, 121, 198),
                value: Color::Rgb(248, 248, 242),
                bold_key: true,
                bold_title: true,
            },
            bar_fill: "█",
            bar_empty: "░",
        }),
        "rosepine" | "rose" => Some(ThemeConfig {
            name: "rose-pine",
            style: Style {
                key: Color::Rgb(234, 154, 151),
                title: Color::Rgb(235, 188, 186),
                value: Color::Rgb(224, 222, 244),
                bold_key: true,
                bold_title: true,
            },
            bar_fill: "◆",
            bar_empty: "◇",
        }),
        _ => None,
    }
}

pub const THEMES: &[&str] = &[
    "catppuccin",
    "tokyo-night",
    "nord",
    "gruvbox",
    "dracula",
    "rose-pine",
];

/// Maximum number of colour stops a custom gradient may have.
pub const MAX_GRADIENT_STOPS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientPreset {
    Rainbow,
    Sunset,
    Cyberpunk,
    Synthwave,
    Fire,
    Ice,
    Matrix,
    Dracula,
    /// User-defined gradient from hex colours, e.g. `"#ff007f,#7928ca,#00dfd8"`.
    /// Stored inline so the type stays `Copy`.
    Custom {
        stops: [(u8, u8, u8); MAX_GRADIENT_STOPS],
        len: u8,
    },
}

impl GradientPreset {
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.contains(',') || trimmed.starts_with('#') {
            return Self::parse_custom(trimmed);
        }
        match trimmed
            .to_ascii_lowercase()
            .replace(['-', '_'], "")
            .as_str()
        {
            "rainbow" => Some(Self::Rainbow),
            "sunset" => Some(Self::Sunset),
            "cyberpunk" => Some(Self::Cyberpunk),
            "synthwave" | "synth" => Some(Self::Synthwave),
            "fire" => Some(Self::Fire),
            "ice" => Some(Self::Ice),
            "matrix" => Some(Self::Matrix),
            "dracula" => Some(Self::Dracula),
            _ => None,
        }
    }

    /// Parses a comma- or space-separated list of 1..=8 hex colours.
    pub fn parse_custom(s: &str) -> Option<Self> {
        let mut stops = [(0u8, 0u8, 0u8); MAX_GRADIENT_STOPS];
        let mut len = 0usize;
        for part in s.split([',', ' ']).filter(|p| !p.trim().is_empty()) {
            if len == MAX_GRADIENT_STOPS {
                return None;
            }
            let p = part.trim();
            let hex = p.strip_prefix('#').unwrap_or(p);
            if hex.len() != 6 {
                return None;
            }
            match Color::parse(hex)? {
                Color::Rgb(r, g, b) => stops[len] = (r, g, b),
                _ => return None,
            }
            len += 1;
        }
        if len == 0 {
            return None;
        }
        Some(Self::Custom {
            stops,
            len: len as u8,
        })
    }

    /// The canonical config-file spelling of this gradient.
    pub fn to_config_string(&self) -> String {
        match self {
            Self::Rainbow => "rainbow".into(),
            Self::Sunset => "sunset".into(),
            Self::Cyberpunk => "cyberpunk".into(),
            Self::Synthwave => "synthwave".into(),
            Self::Fire => "fire".into(),
            Self::Ice => "ice".into(),
            Self::Matrix => "matrix".into(),
            Self::Dracula => "dracula".into(),
            Self::Custom { .. } => self
                .colors()
                .iter()
                .map(|(r, g, b)| format!("#{r:02x}{g:02x}{b:02x}"))
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    pub fn colors(&self) -> &[(u8, u8, u8)] {
        match self {
            Self::Custom { stops, len } => &stops[..*len as usize],
            Self::Rainbow => &[
                (255, 0, 0),
                (255, 127, 0),
                (255, 255, 0),
                (0, 255, 0),
                (0, 255, 255),
                (0, 0, 255),
                (139, 0, 255),
            ],
            Self::Sunset => &[
                (255, 94, 77),
                (255, 154, 60),
                (238, 90, 153),
                (112, 111, 211),
            ],
            Self::Cyberpunk => &[(0, 240, 255), (255, 0, 128)],
            Self::Synthwave => &[(138, 43, 226), (255, 20, 147), (255, 140, 0)],
            Self::Fire => &[(255, 0, 0), (255, 140, 0), (255, 255, 0)],
            Self::Ice => &[(0, 100, 255), (0, 220, 255), (240, 255, 255)],
            Self::Matrix => &[(0, 60, 0), (0, 255, 70)],
            Self::Dracula => &[(255, 121, 198), (189, 147, 249), (139, 233, 253)],
        }
    }

    pub fn sample(&self, t: f64) -> (u8, u8, u8) {
        let colors = self.colors();
        if colors.is_empty() {
            return (255, 255, 255);
        }
        if colors.len() == 1 || t <= 0.0 {
            return colors[0];
        }
        if t >= 1.0 {
            return colors[colors.len() - 1];
        }

        let segments = colors.len() - 1;
        let scaled = t * segments as f64;
        let idx = scaled.floor() as usize;
        let frac = scaled - idx as f64;

        let c1 = colors[idx];
        let c2 = colors[(idx + 1).min(colors.len() - 1)];

        let r = (c1.0 as f64 + (c2.0 as f64 - c1.0 as f64) * frac).round() as u8;
        let g = (c1.1 as f64 + (c2.1 as f64 - c1.1 as f64) * frac).round() as u8;
        let b = (c1.2 as f64 + (c2.2 as f64 - c1.2 as f64) * frac).round() as u8;

        (r, g, b)
    }
}

pub const GRADIENTS: &[&str] = &[
    "rainbow",
    "sunset",
    "cyberpunk",
    "synthwave",
    "fire",
    "ice",
    "matrix",
    "dracula",
];

pub fn apply_gradient_to_text(text: &str, grad: GradientPreset) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len();
    if total == 0 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + total * 25);
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            out.push('\n');
            continue;
        }
        let t = if total <= 1 {
            0.5
        } else {
            i as f64 / (total - 1) as f64
        };
        let (r, g, b) = grad.sample(t);
        out.push_str(&format!("\x1b[38;2;{r};{g};{b}m{line}\x1b[0m\n"));
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(Color::parse("#ff007f"), Some(Color::Rgb(255, 0, 127)));
        assert_eq!(Color::parse("00ffcc"), Some(Color::Rgb(0, 255, 204)));
        assert_eq!(Color::parse("cyan"), Some(Color::Cyan));
    }

    #[test]
    fn test_painter_rgb() {
        let p = Painter::new(true);
        let s = p.paint("hello", Color::Rgb(10, 20, 30), false);
        assert_eq!(s, "\x1b[38;2;10;20;30mhello\x1b[0m");
    }

    #[test]
    fn test_themes_and_gradients() {
        assert!(get_theme("catppuccin").is_some());
        assert!(get_theme("tokyo-night").is_some());
        assert!(get_theme("nord").is_some());
        assert!(GradientPreset::parse("cyberpunk").is_some());
    }

    #[test]
    fn custom_hex_gradient_parses_and_round_trips() {
        let g = GradientPreset::parse("#ff007f, #7928ca,#00dfd8").unwrap();
        assert_eq!(g.colors(), &[(255, 0, 127), (121, 40, 202), (0, 223, 216)]);
        assert_eq!(g.to_config_string(), "#ff007f,#7928ca,#00dfd8");
        assert_eq!(GradientPreset::parse(&g.to_config_string()), Some(g));
        assert_eq!(g.sample(0.0), (255, 0, 127));
        assert_eq!(g.sample(1.0), (0, 223, 216));
        // a single colour is a solid fill
        assert!(GradientPreset::parse("#123456").is_some());
    }

    #[test]
    fn custom_hex_gradient_rejects_bad_input() {
        assert!(GradientPreset::parse("#ff00,#00ff00").is_none());
        assert!(GradientPreset::parse("red,blue").is_none());
        assert!(GradientPreset::parse(",").is_none());
        let nine = vec!["#000000"; 9].join(",");
        assert!(GradientPreset::parse(&nine).is_none());
    }

    #[test]
    fn style_overrides_win_over_theme() {
        let mut s = get_theme("nord").unwrap().style;
        let o = StyleOverrides {
            key: Some(Color::Rgb(1, 2, 3)),
            ..Default::default()
        };
        o.apply(&mut s);
        assert_eq!(s.key, Color::Rgb(1, 2, 3));
        assert_eq!(s.title, get_theme("nord").unwrap().style.title);
    }
}
