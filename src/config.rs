use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::style::{Color, Style};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

pub fn set_global(cfg: Config) {
    let _ = GLOBAL_CONFIG.set(cfg);
}

pub fn get() -> &'static Config {
    static DEFAULT_CFG: OnceLock<Config> = OnceLock::new();
    GLOBAL_CONFIG
        .get()
        .unwrap_or_else(|| DEFAULT_CFG.get_or_init(Config::default))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Logo {
    Auto,
    Mini,
    None,
    Named(String),
    Image(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarConfig {
    pub width: usize,
    pub fill: String,
    pub empty: String,
}

impl Default for BarConfig {
    fn default() -> Self {
        BarConfig {
            width: 20,
            fill: "\u{2588}".to_string(),
            empty: "\u{2591}".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConfig {
    pub weather_ip: String,
    pub weather_host: String,
    pub publicip_host: String,
    pub publicip_fallback: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            weather_ip: "5.9.243.187".to_string(),
            weather_host: "wttr.in".to_string(),
            publicip_host: "myip.opendns.com".to_string(),
            publicip_fallback: "icanhazip.com".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TitleAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl TitleAlign {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "left" => Some(Self::Left),
            "center" | "middle" => Some(Self::Center),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub logo: Logo,
    pub preset: Option<String>,
    pub image_cols: usize,
    pub image_rows: usize,
    pub modules: Option<Vec<String>>,
    pub json: bool,
    pub color: bool,
    pub config_path: Option<PathBuf>,
    pub list: bool,
    pub timing: bool,
    pub no_cache: bool,
    pub fast: bool,
    pub all: bool,
    pub logo_top: bool,
    pub border: bool,
    pub border_title: TitleAlign,
    pub nerd: bool,
    pub nerd_icons_only: bool,
    pub theme: Option<String>,
    pub gradient: Option<crate::style::GradientPreset>,
    pub git: bool,
    pub network: bool,
    pub quotes_file: Option<PathBuf>,
    pub style: Style,
    pub style_overrides: crate::style::StyleOverrides,
    pub bar: BarConfig,
    pub keys: HashMap<String, String>,
    pub format: HashMap<String, String>,
    pub export: Option<String>,
    pub network_cfg: NetworkConfig,
    pub raw_config_content: Option<String>,
    pub warning: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logo: Logo::Auto,
            preset: None,
            image_cols: 34,
            image_rows: 17,
            modules: None,
            json: false,
            color: true,
            config_path: None,
            list: false,
            timing: false,
            no_cache: false,
            fast: false,
            all: false,
            network: false,
            logo_top: false,
            border: false,
            border_title: TitleAlign::Left,
            nerd: false,
            nerd_icons_only: false,
            theme: None,
            gradient: None,
            git: false,
            quotes_file: None,
            export: None,
            style: Style::default(),
            style_overrides: crate::style::StyleOverrides::default(),
            bar: BarConfig::default(),
            keys: HashMap::new(),
            format: HashMap::new(),
            network_cfg: NetworkConfig::default(),
            raw_config_content: None,
            warning: None,
        }
    }
}

impl Config {
    pub fn format(&self, module_id: &str) -> Option<&str> {
        self.format.get(module_id).map(|s| s.as_str())
    }

    pub fn key_label<'a>(&'a self, name: &'a str) -> &'a str {
        if self.keys.is_empty() {
            return name;
        }
        if let Some(custom) = self.keys.get(name) {
            return custom.as_str();
        }
        let lower = name.to_ascii_lowercase();
        if let Some(custom) = self.keys.get(&lower) {
            return custom.as_str();
        }
        name
    }

    pub fn hash_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        if let Some(ref raw) = self.raw_config_content {
            raw.hash(&mut hasher);
        } else {
            self.bar.width.hash(&mut hasher);
            self.bar.fill.hash(&mut hasher);
            self.bar.empty.hash(&mut hasher);
            self.network_cfg.weather_ip.hash(&mut hasher);
            self.network_cfg.weather_host.hash(&mut hasher);
            self.network_cfg.publicip_host.hash(&mut hasher);
            self.network_cfg.publicip_fallback.hash(&mut hasher);
            for (k, v) in &self.format {
                k.hash(&mut hasher);
                v.hash(&mut hasher);
            }
            for (k, v) in &self.keys {
                k.hash(&mut hasher);
                v.hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

#[derive(Debug)]
pub enum Parsed {
    Run(Box<Config>),
    Help,
    Version,
    GenConfig { path: Option<String>, force: bool },
    ListThemes,
    ListPresets,
    Completion(String),
    Error(String),
}

pub const HELP: &str = "\
omnifetch - a fast system info fetcher

USAGE:
    omnifetch [OPTIONS]

OPTIONS:
    -l, --logo <NAME>      Logo to use: auto (default), none, mini, or a distro id
        --logo-mini        Use mini ASCII logo
        --logo-top         Render logo on top instead of left
        --border           Wrap system info in unicode border box
        --border-title <ALIGN> Alignment for category titles in border: left, center, right (default: left)
        --nerd             Prefix module keys with Nerd Font icons
        --nerd-only        Display only Nerd Font icons (hide text keys)
    -t, --theme <NAME>     Apply color theme: catppuccin, tokyo-night, nord, gruvbox, dracula, rose-pine
        --list-themes      List available built-in themes and exit
        --list-presets     List available layout presets and exit
        --gradient <NAME>  Apply color gradient: rainbow, sunset, cyberpunk, synthwave, fire, ice, matrix, dracula,
                           or custom hex colours: \"#ff007f,#7928ca,#00dfd8\"
        --git              Display current git repository statistics
        --completion <SH>  Generate shell completion (bash, zsh, fish)
    -i, --image <PATH>     Display a graphic image (PNG/Sixel) using Kitty Graphics Protocol
        --image-cols <NUM> Width of image in terminal character cells (default: 34)
        --image-rows <NUM> Height of image in terminal lines (default: 17)
    -p, --preset <NAME>    Use a built-in layout preset (see --list-presets)
    -a, --all              Run all available system modules (shorthand for -p all)
    -f, --fast             Run minimal set of modules for maximum speed
    -m, --modules <LIST>   Comma- or space-separated module ids to run
    -c, --config <PATH>    Load modules/logo from a TOML config
        --gen-config <?PATH> Interactively generate a config file at the specified path (use - for stdout)
        --gen-config-force Overwrite existing config file without confirmation prompt
        --list-modules     Print all module ids and exit
    -T, --timing           Print per-module timings to stderr
        --no-cache         Disable caching completely (always fetch fresh data)
        --network          Enable modules requiring external internet requests (Public IP, Weather)
        --quotes-file <PATH> Custom quotes JSON or text file to load quotes from
    -j, --json             Print raw JSON instead of a table
        --export <TARGET>  Export output as SVG or HTML (\"svg\", \"html\", or filename .svg/.html)
        --no-color         Disable colors (same as NO_COLOR=1)
    -h, --help             Print this help
    -V, --version          Print version

EXAMPLES:
    omnifetch                          default layout with auto-detected logo
    omnifetch --logo mini --border     mini logo with decorative border
    omnifetch --theme tokyo-night      apply Tokyo Night color theme
    omnifetch --gradient cyberpunk     neon cyberpunk gradient on logo
    omnifetch --nerd                   Nerd Font icons
    omnifetch -p modern                modern layout preset
    omnifetch -p detailed              everything, in full
    omnifetch -p fastfetch             same modules as fastfetch
    omnifetch -p neofetch              same modules as neofetch
    omnifetch --network                add Public IP and Weather
    omnifetch --all --no-cache         every module, nothing cached
    omnifetch --fast                   fastest execution mode
";

pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"# omnifetch configuration file
# Location: ~/.config/omnifetch/config.toml or $XDG_CONFIG_HOME/omnifetch/config.toml

# Logo: "auto", "mini", "none", or distro name (e.g. "arch", "ubuntu", "fedora", "debian")
logo = "auto"

# Disable color output
# color = false

# Disable caching completely (always fetch fresh data without /dev/shm)
# cache = false

# Run minimal set of modules for highest speed
# fast = false

# Modules to display, in order.
# NOTE: this key must stay above any [section] header below. TOML attaches a
# bare key to the section that precedes it, so putting it after [bar] would
# silently parse it as bar.modules and your list would be ignored.
modules = [
    # Header
    "title",
    "separator",
    "break",

    # System
    "os",
    "host",
    "kernel",
    "uptime",
    "packages",
    "break",

    # Desktop & Interface
    "shell",
    "display",
    "de",
    "wm",
    "theme",
    "icons",
    "font",
    "terminal",
    "break",

    # Hardware & Performance
    "cpu",
    "gpu",
    "sound",
    "memory",
    "swap",
    "disk",
    "battery",
    "break",

    # Miscellaneous & Colors
    "locale",
    "break",
    "colors",
]

# Progress bar styling
[bar]
# Default width of progress bars
width = 20
# Filled character
fill = "█"
# Empty character
empty = "░"

# Custom labels for module keys
[keys]
# memory = "RAM"
# swap = "SWAP"
# cpu = "CPU"
# disk = "Storage"

# Colors and styling
[style]
# key_color = "cyan"
# title_color = "bright_black"
# bold_key = true
# bold_title = true

# Custom format strings for modules
# Variables and functions:
#   memory:  {used}/{u}, {total}/{t}, {free}/{f}, {pct}/{p}, {bar}/{b}, {bar:N}
#   swap:    {used}/{u}, {total}/{t}, {free}/{f}, {pct}/{p}, {bar}/{b}, {bar:N}
#   disk:    {mount}/{m}, {used}/{u}, {total}/{t}, {free}/{f}, {pct}/{p}, {bar}/{b}, {bar:N}
#   cpu:     {name}/{n}, {cores}/{c}, {freq}/{f}
#   battery: {pct}/{p}, {bar}/{b}, {status}/{s}, {wh}
#   title:   {user}/{u}, {host}/{h}
#   uptime:  {uptime}, {days}/{d}, {hours}/{h}, {mins}/{m}, {secs}/{s}
# Colors:    {red}, {green}, {yellow}, {blue}, {magenta}, {cyan}, {white}, {bold}, {reset}
# Custom & Command modules:
#   custom:  displays arbitrary text: custom = "My custom text"
#   command: runs a shell command:   command = "uname -r"
# custom = "Custom Text"
# command = "whoami"
[format]
memory = "{used} / {total} {bar} {pct}%"
swap = "{used} / {total} {bar} {pct}%"
disk = "{mount} {used} / {total} {bar} ({pct}%)"
battery = "{bar} {pct}%{status}{wh}"
cpu = "{name} ({cores}) @ {freq}"
title = "{user}@{host}"
uptime = "{uptime}"

# Network endpoints for internet-dependent modules
[network]
weather_ip = "5.9.243.187"          # пустая строка = только DNS
weather_host = "wttr.in"
publicip_host = "myip.opendns.com"
publicip_fallback = "icanhazip.com"
"#;

pub fn parse(args: &[String]) -> Parsed {
    // Normalize flags (strip accidental trailing whitespace/NBSP from flags starting with '-')
    let sanitized_args: Vec<String>;
    let args: &[String] = if args
        .iter()
        .any(|s| s.starts_with('-') && s.ends_with(|c: char| c.is_whitespace() || c == '\u{a0}'))
    {
        sanitized_args = args
            .iter()
            .map(|s| {
                if s.starts_with('-') {
                    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{a0}')
                        .to_string()
                } else {
                    s.clone()
                }
            })
            .collect();
        &sanitized_args
    } else {
        args
    };

    // 1. Immediate flags short-circuit before any config file is loaded.
    let mut k = 0;
    while k < args.len() {
        let a = args[k].as_str();
        match a {
            "-h" | "--help" => return Parsed::Help,
            "-V" | "--version" => return Parsed::Version,
            "--gen-config" => {
                let mut path = None;
                let mut force = false;
                if let Some(next) = args.get(k + 1) {
                    if next == "--gen-config-force" {
                        force = true;
                    } else if next == "-" || !next.starts_with('-') {
                        path = Some(next.clone());
                        if let Some(after) = args.get(k + 2)
                            && after == "--gen-config-force"
                        {
                            force = true;
                        }
                    }
                }
                return Parsed::GenConfig { path, force };
            }
            "--gen-config-force" => {
                let mut path = None;
                if let Some(next) = args.get(k + 1)
                    && (next == "-" || !next.starts_with('-'))
                {
                    path = Some(next.clone());
                }
                return Parsed::GenConfig { path, force: true };
            }
            "--list-themes" => return Parsed::ListThemes,
            "--list-presets" => return Parsed::ListPresets,
            "--completion" => {
                let sh = match args.get(k + 1) {
                    Some(s) => s.clone(),
                    None => return Parsed::Error("--completion requires a shell name".into()),
                };
                return Parsed::Completion(sh);
            }
            _ => {}
        }
        k += 1;
    }

    // 2. Discover config file path (either explicit via -c / --config, or default)
    let mut config_path = None;
    let mut explicit_config = false;
    let mut j = 0;
    while j < args.len() {
        let a = args[j].as_str();
        if a == "-c" || a == "--config" {
            if let Some(val) = args.get(j + 1) {
                config_path = Some(PathBuf::from(val));
                explicit_config = true;
                j += 1;
            } else {
                return Parsed::Error(format!("{a} requires a value"));
            }
        }
        j += 1;
    }

    if config_path.is_none() {
        config_path = find_default_config_path();
    }

    let mut cfg = Config::default();
    if let Some(p) = config_path {
        cfg.config_path = Some(p.clone());
        match std::fs::read_to_string(&p) {
            Ok(text) => match crate::toml::parse(&text) {
                Ok(v) => {
                    cfg.raw_config_content = Some(text);
                    apply_toml(&mut cfg, &v);
                }
                Err(e) => return Parsed::Error(format!("{}: {e}", p.display())),
            },
            Err(e) => {
                if explicit_config {
                    return Parsed::Error(format!("{}: {e}", p.display()));
                }
            }
        }
    }

    // 3. Parse CLI args: command-line flags have highest precedence and override config file settings.
    let mut i = 0;
    let mut fast_pos: Option<usize> = None;
    let mut no_cache_pos: Option<usize> = None;

    let need = |i: usize, flag: &str, args: &[String]| -> Result<String, String> {
        args.get(i + 1)
            .cloned()
            .ok_or_else(|| format!("{flag} requires a value"))
    };

    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" | "--help" => return Parsed::Help,
            "-V" | "--version" => return Parsed::Version,
            "--gen-config" | "--gen-config-force" => {}
            "--no-color" => cfg.color = false,
            "--list-modules" | "--list" => cfg.list = true,
            "--timing" | "-T" => cfg.timing = true,
            "--no-cache" => {
                if no_cache_pos.is_none() {
                    no_cache_pos = Some(i);
                }
                cfg.no_cache = true;
            }
            "-a" | "--all" => {
                cfg.all = true;
                cfg.preset = Some("all".to_string());
                cfg.modules = None;
            }
            "-f" | "--fast" => {
                if fast_pos.is_none() {
                    fast_pos = Some(i);
                }
                cfg.fast = true;
                cfg.preset = None;
                cfg.modules = None;
            }
            "--network" => cfg.network = true,
            "--logo-mini" => cfg.logo = Logo::Mini,
            "--logo-top" => cfg.logo_top = true,
            "--border" => cfg.border = true,
            "--border-title" | "--border-align" => match need(i, a, args) {
                Ok(v) => {
                    if let Some(align) = TitleAlign::parse(&v) {
                        cfg.border_title = align;
                        cfg.border = true;
                    } else {
                        return Parsed::Error(format!(
                            "unknown border title alignment '{v}'. Available alignments: left, center, right."
                        ));
                    }
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            a if a.starts_with("--border-title=") || a.starts_with("--border-align=") => {
                let v = a.split_once('=').map(|(_, val)| val).unwrap_or("");
                if let Some(align) = TitleAlign::parse(v) {
                    cfg.border_title = align;
                    cfg.border = true;
                } else {
                    return Parsed::Error(format!(
                        "unknown border title alignment '{v}'. Available alignments: left, center, right."
                    ));
                }
            }
            "-j" | "--json" => cfg.json = true,
            "--list-themes" => return Parsed::ListThemes,
            "--list-presets" => return Parsed::ListPresets,
            "--nerd" => cfg.nerd = true,
            "--nerd-only" => {
                cfg.nerd = true;
                cfg.nerd_icons_only = true;
            }
            "--git" => cfg.git = true,
            "--export" => match need(i, a, args) {
                Ok(v) => {
                    cfg.export = Some(v);
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "--quotes-file" | "--quotes" => match need(i, a, args) {
                Ok(v) => {
                    cfg.quotes_file = Some(PathBuf::from(v));
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "-t" | "--theme" => match need(i, a, args) {
                Ok(v) => {
                    cfg.theme = Some(v);
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "--gradient" => match need(i, a, args) {
                Ok(v) => {
                    if let Some(g) = crate::style::GradientPreset::parse(&v) {
                        cfg.gradient = Some(g);
                    } else {
                        return Parsed::Error(format!(
                            "unknown gradient '{v}'. Available gradients: rainbow, sunset, cyberpunk, synthwave, fire, ice, matrix, dracula, or 1-8 hex colours such as \"#ff007f,#00dfd8\"."
                        ));
                    }
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "--completion" => match need(i, a, args) {
                Ok(v) => return Parsed::Completion(v),
                Err(e) => return Parsed::Error(e),
            },
            "-p" | "--preset" => match need(i, a, args) {
                Ok(v) => {
                    cfg.preset = Some(v);
                    cfg.modules = None;
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "-i" | "--image" | "--logo-image" => match need(i, a, args) {
                Ok(v) => {
                    let pb = PathBuf::from(v);
                    cfg.logo = Logo::Image(pb);
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "--image-cols" => match need(i, a, args) {
                Ok(v) => {
                    if let Ok(num) = v.parse::<usize>() {
                        cfg.image_cols = num;
                    }
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "--image-rows" => match need(i, a, args) {
                Ok(v) => {
                    if let Ok(num) = v.parse::<usize>() {
                        cfg.image_rows = num;
                    }
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            "-l" | "--logo" => match need(i, a, args) {
                Ok(v) => {
                    cfg.logo = parse_logo(&v);
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            a if a.starts_with("-m=")
                || a.starts_with("--modules=")
                || a.starts_with("--module=") =>
            {
                let v = a.split_once('=').map(|(_, val)| val).unwrap_or("");
                let list: Vec<String> = v
                    .split(',')
                    .map(|s| s.trim().to_ascii_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                if list.is_empty() {
                    return Parsed::Error(
                        "-m/--modules requires at least one module name".to_string(),
                    );
                }
                if list.iter().any(|s| s == "all") {
                    cfg.all = true;
                }
                cfg.preset = None;
                cfg.modules = Some(list);
            }
            "-m" | "--modules" | "--module" => {
                let mut list: Vec<String> = Vec::new();
                let mut next_i = i + 1;
                while next_i < args.len() && !args[next_i].starts_with('-') {
                    for part in args[next_i].split(',') {
                        let clean = part.trim().to_ascii_lowercase();
                        if !clean.is_empty() {
                            list.push(clean);
                        }
                    }
                    next_i += 1;
                }
                if list.is_empty() {
                    return Parsed::Error(
                        "-m/--modules requires at least one module name".to_string(),
                    );
                }
                if list.iter().any(|s| s == "all") {
                    cfg.all = true;
                }
                cfg.preset = None;
                cfg.modules = Some(list);
                i = next_i - 1;
            }
            "-c" | "--config" => match need(i, a, args) {
                Ok(v) => {
                    cfg.config_path = Some(PathBuf::from(v));
                    i += 1;
                }
                Err(e) => return Parsed::Error(e),
            },
            _ => return Parsed::Error(format!("unknown option: {a}")),
        }
        i += 1;
    }

    if let (Some(f_idx), Some(nc_idx)) = (fast_pos, no_cache_pos) {
        if f_idx < nc_idx {
            cfg.fast = true;
            cfg.no_cache = false;
            cfg.warning = Some(
                "--fast forcibly uses cache for maximum speed (--no-cache ignored)".to_string(),
            );
        } else {
            cfg.fast = false;
            cfg.no_cache = true;
            cfg.warning = Some("--no-cache disables cache completely (--fast ignored)".to_string());
        }
    }

    if cfg.fast && cfg.network {
        return Parsed::Error(
            "cannot use --network with --fast (network requests contradict fast mode)".to_string(),
        );
    }

    Parsed::Run(Box::new(cfg))
}

pub fn default_config_write_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg).join("omnifetch/config.toml");
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        return PathBuf::from(home).join(".config/omnifetch/config.toml");
    }
    PathBuf::from("config.toml")
}

fn find_default_config_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("OMNIFETCH_CONFIG") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let pb = PathBuf::from(xdg).join("omnifetch/config.toml");
        if pb.is_file() {
            return Some(pb);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let pb = PathBuf::from(home).join(".config/omnifetch/config.toml");
        if pb.is_file() {
            return Some(pb);
        }
    }
    None
}

fn parse_logo(v: &str) -> Logo {
    if let Some(path) = v.strip_prefix("image:") {
        return Logo::Image(PathBuf::from(path));
    }
    match v.to_ascii_lowercase().as_str() {
        "none" | "off" | "false" => Logo::None,
        "mini" | "small" => Logo::Mini,
        "auto" | "on" | "true" | "normal" | "large" => Logo::Auto,
        other => Logo::Named(other.to_string()),
    }
}

const KNOWN_CONFIG_KEYS: &[&str] = &[
    "logo",
    "logo_mini",
    "preset",
    "image",
    "image_cols",
    "image_rows",
    "color",
    "cache",
    "no_cache",
    "fast",
    "logo_top",
    "border",
    "border_title",
    "border_align",
    "nerd",
    "nerd_icons_only",
    "theme",
    "gradient",
    "git",
    "network",
    "quotes_file",
    "quotes",
    "quote",
    "modules",
    "bar",
    "keys",
    "format",
    "style",
];

fn apply_toml(cfg: &mut Config, v: &crate::toml::TomlValue) {
    if let Some(tbl) = v.as_table() {
        for (k, val) in tbl {
            let lower_k = k.to_ascii_lowercase();
            if !KNOWN_CONFIG_KEYS.contains(&k.as_str())
                && !crate::modules::ALL_MODULE_IDS.contains(&lower_k.as_str())
                && k != "values"
            {
                eprintln!("omnifetch: warning: unknown config key '{k}' in config file");
            } else if crate::modules::ALL_MODULE_IDS.contains(&lower_k.as_str()) {
                let val_str = match val {
                    crate::toml::TomlValue::String(s) => Some(s.clone()),
                    crate::toml::TomlValue::Integer(i) => Some(i.to_string()),
                    crate::toml::TomlValue::Float(f) => Some(f.to_string()),
                    crate::toml::TomlValue::Boolean(b) => Some(b.to_string()),
                    _ => None,
                };
                if let Some(s) = val_str {
                    cfg.format.insert(lower_k, s);
                }
            }
        }
    }
    if let Some(vals) = v.get("values").and_then(crate::toml::TomlValue::as_table) {
        for (k, val) in vals {
            let val_str = match val {
                crate::toml::TomlValue::String(s) => Some(s.clone()),
                crate::toml::TomlValue::Integer(i) => Some(i.to_string()),
                crate::toml::TomlValue::Float(f) => Some(f.to_string()),
                crate::toml::TomlValue::Boolean(b) => Some(b.to_string()),
                _ => None,
            };
            if let Some(s) = val_str {
                cfg.format.insert(k.to_ascii_lowercase(), s);
            }
        }
    }
    if let Some(l) = v.get("logo").and_then(crate::toml::TomlValue::as_str) {
        cfg.logo = parse_logo(l);
    }
    if let Some(m) = v.get("logo_mini").and_then(crate::toml::TomlValue::as_bool)
        && m
    {
        cfg.logo = Logo::Mini;
    }
    if let Some(p) = v.get("preset").and_then(crate::toml::TomlValue::as_str) {
        cfg.preset = Some(p.to_string());
    }
    if let Some(img) = v.get("image").and_then(crate::toml::TomlValue::as_str) {
        let pb = PathBuf::from(img);
        cfg.logo = Logo::Image(pb);
    }
    if let Some(cols) = v
        .get("image_cols")
        .and_then(crate::toml::TomlValue::as_integer)
    {
        cfg.image_cols = cols as usize;
    }
    if let Some(rows) = v
        .get("image_rows")
        .and_then(crate::toml::TomlValue::as_integer)
    {
        cfg.image_rows = rows as usize;
    }
    if let Some(c) = v.get("color").and_then(crate::toml::TomlValue::as_bool) {
        cfg.color = c;
    }
    if let Some(c) = v.get("cache").and_then(crate::toml::TomlValue::as_bool) {
        cfg.no_cache = !c;
    }
    if let Some(nc) = v.get("no_cache").and_then(crate::toml::TomlValue::as_bool) {
        cfg.no_cache = nc;
    }
    if let Some(f) = v.get("fast").and_then(crate::toml::TomlValue::as_bool) {
        cfg.fast = f;
    }
    if let Some(lt) = v.get("logo_top").and_then(crate::toml::TomlValue::as_bool) {
        cfg.logo_top = lt;
    }
    if let Some(b) = v.get("border").and_then(crate::toml::TomlValue::as_bool) {
        cfg.border = b;
    }
    if let Some(border_table) = v.get("border").and_then(crate::toml::TomlValue::as_table) {
        if let Some(enabled) = border_table
            .get("enabled")
            .and_then(crate::toml::TomlValue::as_bool)
        {
            cfg.border = enabled;
        }
        if let Some(align_str) = border_table
            .get("title_align")
            .or_else(|| border_table.get("align"))
            .and_then(crate::toml::TomlValue::as_str)
            && let Some(align) = TitleAlign::parse(align_str)
        {
            cfg.border_title = align;
        }
    }
    if let Some(align_str) = v
        .get("border_title")
        .or_else(|| v.get("border_align"))
        .and_then(crate::toml::TomlValue::as_str)
        && let Some(align) = TitleAlign::parse(align_str)
    {
        cfg.border_title = align;
    }
    if let Some(n) = v.get("nerd").and_then(crate::toml::TomlValue::as_bool) {
        cfg.nerd = n;
    }
    if let Some(n) = v
        .get("nerd_icons_only")
        .and_then(crate::toml::TomlValue::as_bool)
    {
        cfg.nerd_icons_only = n;
    }
    if let Some(th) = v.get("theme").and_then(crate::toml::TomlValue::as_str) {
        cfg.theme = Some(th.to_string());
    }
    if let Some(gr) = v.get("gradient").and_then(crate::toml::TomlValue::as_str) {
        if let Some(preset) = crate::style::GradientPreset::parse(gr) {
            cfg.gradient = Some(preset);
        } else {
            eprintln!(
                "omnifetch: warning: unknown gradient '{gr}' in config. Available gradients: rainbow, sunset, cyberpunk, synthwave, fire, ice, matrix, dracula, or 1-8 hex colours such as \"#ff007f,#00dfd8\"."
            );
        }
    }
    if let Some(g) = v.get("git").and_then(crate::toml::TomlValue::as_bool) {
        cfg.git = g;
    }
    if let Some(net) = v.get("network").and_then(crate::toml::TomlValue::as_bool) {
        cfg.network = net;
    }
    if let Some(q) = v
        .get("quotes_file")
        .or_else(|| v.get("quotes"))
        .and_then(crate::toml::TomlValue::as_str)
    {
        cfg.quotes_file = Some(PathBuf::from(q));
    }
    if let Some(qt) = v.get("quote").and_then(crate::toml::TomlValue::as_table)
        && let Some(f) = qt
            .get("file")
            .or_else(|| qt.get("path"))
            .and_then(crate::toml::TomlValue::as_str)
    {
        cfg.quotes_file = Some(PathBuf::from(f));
    }
    if let Some(list) = v.get("modules").and_then(crate::toml::TomlValue::as_array) {
        cfg.modules = Some(
            list.iter()
                .filter_map(crate::toml::TomlValue::as_str)
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
        );
    }
    if let Some(bar) = v.get("bar").and_then(crate::toml::TomlValue::as_table) {
        if let Some(w) = bar
            .get("width")
            .and_then(crate::toml::TomlValue::as_integer)
        {
            cfg.bar.width = w.clamp(1, 200) as usize;
        }
        if let Some(f) = bar.get("fill").and_then(crate::toml::TomlValue::as_str) {
            cfg.bar.fill = f.to_string();
        }
        if let Some(e) = bar.get("empty").and_then(crate::toml::TomlValue::as_str) {
            cfg.bar.empty = e.to_string();
        }
    }
    if let Some(keys) = v.get("keys").and_then(crate::toml::TomlValue::as_table) {
        for (k, val) in keys {
            if let Some(s) = val.as_str() {
                cfg.keys.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(fmt) = v.get("format").and_then(crate::toml::TomlValue::as_table) {
        for (k, val) in fmt {
            let val_str = match val {
                crate::toml::TomlValue::String(s) => Some(s.clone()),
                crate::toml::TomlValue::Integer(i) => Some(i.to_string()),
                crate::toml::TomlValue::Float(f) => Some(f.to_string()),
                crate::toml::TomlValue::Boolean(b) => Some(b.to_string()),
                _ => None,
            };
            if let Some(s) = val_str {
                cfg.format.insert(k.to_ascii_lowercase(), s);
            }
        }
    }
    if let Some(style) = v.get("style").and_then(crate::toml::TomlValue::as_table) {
        if let Some(k) = style
            .get("key_color")
            .and_then(crate::toml::TomlValue::as_str)
            && let Some(c) = Color::parse(k)
        {
            cfg.style.key = c;
            cfg.style_overrides.key = Some(c);
        }
        if let Some(t) = style
            .get("title_color")
            .and_then(crate::toml::TomlValue::as_str)
            && let Some(c) = Color::parse(t)
        {
            cfg.style.title = c;
            cfg.style_overrides.title = Some(c);
        }
        if let Some(val) = style
            .get("value_color")
            .and_then(crate::toml::TomlValue::as_str)
            && let Some(c) = Color::parse(val)
        {
            cfg.style.value = c;
            cfg.style_overrides.value = Some(c);
        }
        if let Some(b) = style
            .get("bold_key")
            .and_then(crate::toml::TomlValue::as_bool)
        {
            cfg.style.bold_key = b;
        }
        if let Some(b) = style
            .get("bold_title")
            .and_then(crate::toml::TomlValue::as_bool)
        {
            cfg.style.bold_title = b;
        }
    }
    if let Some(net) = v.get("network").and_then(crate::toml::TomlValue::as_table) {
        if let Some(ip) = net
            .get("weather_ip")
            .and_then(crate::toml::TomlValue::as_str)
        {
            cfg.network_cfg.weather_ip = ip.trim().to_string();
        }
        if let Some(host) = net
            .get("weather_host")
            .and_then(crate::toml::TomlValue::as_str)
        {
            cfg.network_cfg.weather_host = host.trim().to_string();
        }
        if let Some(host) = net
            .get("publicip_host")
            .and_then(crate::toml::TomlValue::as_str)
        {
            cfg.network_cfg.publicip_host = host.trim().to_string();
        }
        if let Some(fb) = net
            .get("publicip_fallback")
            .and_then(crate::toml::TomlValue::as_str)
        {
            cfg.network_cfg.publicip_fallback = fb.trim().to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty config on a temp path, so `parse` never picks up the developer's own.
    struct ScratchConfig(PathBuf);

    impl ScratchConfig {
        fn new(tag: &str) -> Self {
            let mut p = std::env::temp_dir();
            p.push(format!(
                "omnifetch_test_{tag}_{}_{:?}.toml",
                std::process::id(),
                std::thread::current().id()
            ));
            std::fs::write(&p, "").unwrap();
            Self(p)
        }

        fn path(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }
    }

    impl Drop for ScratchConfig {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn args_with_config(tag: &str, extra: &[&str]) -> (ScratchConfig, Vec<String>) {
        let scratch = ScratchConfig::new(tag);
        let mut v = vec!["-c".to_string(), scratch.path()];
        v.extend(extra.iter().map(|s| s.to_string()));
        (scratch, v)
    }

    fn run(tag: &str, extra: &[&str]) -> Config {
        let (_scratch, args) = args_with_config(tag, extra);
        match parse(&args) {
            Parsed::Run(cfg) => *cfg,
            Parsed::Error(e) => panic!("expected a runnable config, got error: {e}"),
            other => panic!("expected Parsed::Run, got {other:?}"),
        }
    }

    fn parse_err(tag: &str, extra: &[&str]) -> String {
        let (_scratch, args) = args_with_config(tag, extra);
        match parse(&args) {
            Parsed::Error(e) => e,
            _ => panic!("expected Parsed::Error"),
        }
    }

    #[test]
    fn boolean_flags_map_to_config_fields() {
        let cfg = run(
            "flags",
            &["--all", "--json", "--timing", "--border", "--logo-top"],
        );
        assert!(cfg.all && cfg.json && cfg.timing && cfg.border && cfg.logo_top);
        assert!(!cfg.nerd && !cfg.no_cache && !cfg.fast);
    }

    #[test]
    fn nerd_only_implies_nerd() {
        let cfg = run("nerd", &["--nerd-only"]);
        assert!(cfg.nerd, "--nerd-only must also switch on icons");
        assert!(cfg.nerd_icons_only, "text keys stay hidden");
    }

    #[test]
    fn modules_list_is_trimmed_and_lowercased() {
        let cfg = run("mods", &["-m", " OS , Cpu ,gpu "]);
        assert_eq!(cfg.modules.unwrap(), vec!["os", "cpu", "gpu"]);
        assert!(!cfg.all);
    }

    #[test]
    fn modules_all_alias_sets_all_flag() {
        let cfg = run("modsall", &["-m", "os,all"]);
        assert!(cfg.all);
    }

    #[test]
    fn empty_modules_list_is_an_error() {
        let err = parse_err("modempty", &["-m", "  ,  "]);
        assert!(err.contains("requires at least one module name"));
    }

    #[test]
    fn logo_accepts_known_keywords_and_names() {
        assert_eq!(parse_logo("auto"), Logo::Auto);
        assert_eq!(parse_logo("MINI"), Logo::Mini);
        assert_eq!(parse_logo("none"), Logo::None);
        assert_eq!(parse_logo("arch"), Logo::Named("arch".to_string()));
        assert_eq!(
            parse_logo("image:/tmp/x.png"),
            Logo::Image(PathBuf::from("/tmp/x.png"))
        );
    }

    #[test]
    fn image_flag_selects_image_logo_and_geometry() {
        let cfg = run(
            "img",
            &[
                "-i",
                "/tmp/p.png",
                "--image-cols",
                "40",
                "--image-rows",
                "20",
            ],
        );
        assert_eq!(cfg.logo, Logo::Image(PathBuf::from("/tmp/p.png")));
        assert_eq!(cfg.image_cols, 40);
        assert_eq!(cfg.image_rows, 20);
    }

    #[test]
    fn bad_numeric_geometry_keeps_the_default() {
        let cfg = run("badnum", &["--image-cols", "abc"]);
        assert_eq!(cfg.image_cols, Config::default().image_cols);
    }

    #[test]
    fn unknown_gradient_is_rejected_with_a_hint() {
        let e = parse_err("grad", &["--gradient", "nope"]);
        assert!(e.contains("nope"), "message should name the input: {e}");
        assert!(e.contains("cyberpunk"), "message should list options: {e}");
    }

    #[test]
    fn valid_gradient_is_accepted() {
        assert!(
            run("gradok", &["--gradient", "cyberpunk"])
                .gradient
                .is_some()
        );
    }

    #[test]
    fn unknown_option_and_missing_value_are_errors() {
        assert!(parse_err("opt", &["--definitely-not-a-flag"]).contains("unknown option"));
        assert!(parse_err("val", &["--theme"]).contains("requires a value"));
    }

    #[test]
    fn immediate_flags_short_circuit_parsing() {
        assert!(matches!(parse(&["-h".to_string()]), Parsed::Help));
        assert!(matches!(parse(&["-V".to_string()]), Parsed::Version));
        assert!(matches!(
            parse(&["--gen-config".to_string()]),
            Parsed::GenConfig {
                path: None,
                force: false
            }
        ));
        assert!(matches!(
            parse(&["--gen-config".to_string(), "-".to_string()]),
            Parsed::GenConfig { path: Some(ref p), force: false } if p == "-"
        ));
        assert!(matches!(
            parse(&["--gen-config".to_string(), "myconfig.toml".to_string(), "--gen-config-force".to_string()]),
            Parsed::GenConfig { path: Some(ref p), force: true } if p == "myconfig.toml"
        ));
        assert!(matches!(
            parse(&["--gen-config-force".to_string()]),
            Parsed::GenConfig {
                path: None,
                force: true
            }
        ));
        assert!(matches!(
            parse(&["--list-themes".to_string()]),
            Parsed::ListThemes
        ));
        assert!(matches!(
            parse(&["--list-presets".to_string()]),
            Parsed::ListPresets
        ));
        assert!(matches!(
            parse(&["--completion".to_string(), "zsh".to_string()]),
            Parsed::Completion(s) if s == "zsh"
        ));
    }

    #[test]
    fn toml_sections_are_applied() {
        let scratch = ScratchConfig::new("toml");
        std::fs::write(
            &scratch.0,
            r##"
fast = true
cache = false

[bar]
width = 8
fill = "#"
empty = "-"

[keys]
memory = "RAM"

[format]
memory = "{used} / {total}"

[style]
key_color = "#ff007f"
bold_key = false
"##,
        )
        .unwrap();

        let cfg = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(cfg) => *cfg,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };

        assert!(cfg.fast);
        assert!(cfg.no_cache, "cache = false means no_cache");
        assert_eq!(cfg.bar.width, 8);
        assert_eq!(cfg.bar.fill, "#");
        assert_eq!(cfg.keys.get("memory").map(String::as_str), Some("RAM"));
        assert_eq!(cfg.format("memory"), Some("{used} / {total}"));
        assert!(!cfg.style.bold_key);
    }

    #[test]
    fn hash_tracks_what_affects_cached_values() {
        // The rendered text bakes in format, label and bar width.
        let scratch = ScratchConfig::new("hash");
        let base = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(cfg) => *cfg,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };

        std::fs::write(&scratch.0, "[format]\nmemory = \"{used}\"\n").unwrap();
        let fmt = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(cfg) => *cfg,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_ne!(
            base.hash_value(),
            fmt.hash_value(),
            "format must invalidate"
        );

        std::fs::write(&scratch.0, "[keys]\nmemory = \"RAM\"\n").unwrap();
        let keyed = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(cfg) => *cfg,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_ne!(
            base.hash_value(),
            keyed.hash_value(),
            "labels must invalidate"
        );

        std::fs::write(&scratch.0, "[bar]\nwidth = 42\n").unwrap();
        let bar = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(cfg) => *cfg,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_ne!(
            base.hash_value(),
            bar.hash_value(),
            "bar width must invalidate"
        );

        let again = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(cfg) => *cfg,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_eq!(bar.hash_value(), again.hash_value());
    }

    #[test]
    fn module_list_does_not_affect_the_hash() {
        // The module list is not baked into the values, only the order is.
        assert_eq!(
            run("hl1", &[]).hash_value(),
            run("hl2", &["-m", "os,cpu"]).hash_value()
        );
    }

    #[test]
    fn config_file_parse_errors_surface_with_the_path() {
        let scratch = ScratchConfig::new("bad");
        std::fs::write(&scratch.0, "this is not = valid = toml").unwrap();
        match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Error(e) => assert!(e.contains("omnifetch_test_bad"), "{e}"),
            other => panic!("expected Parsed::Error, got {other:?}"),
        }
    }

    #[test]
    fn missing_explicit_config_file_is_an_error() {
        assert!(
            parse_err(
                "missing",
                &["-c", "/nonexistent/omnifetch/definitely-not-here.toml"]
            )
            .contains("No such file")
        );
    }

    #[test]
    fn gen_config_template_is_valid_toml_and_documents_everything() {
        let v: crate::toml::TomlValue = crate::toml::parse(DEFAULT_CONFIG_TEMPLATE).unwrap();
        for key in [
            "logo", "modules", "bar", "keys", "style", "format", "network",
        ] {
            assert!(
                v.get(key).is_some(),
                "template is missing top-level '{key}'"
            );
        }
        // These must stay bare keys, not get absorbed into the [bar] table above.
        assert!(v.get("bar").unwrap().get("modules").is_none());
        assert!(v["bar"].get("width").is_some());
        assert!(v["modules"].as_array().is_some_and(|a| !a.is_empty()));
        // The template has to stay parseable by the loader itself.
        let scratch = ScratchConfig::new("template");
        std::fs::write(&scratch.0, DEFAULT_CONFIG_TEMPLATE).unwrap();
        assert!(matches!(
            parse(&["-c".to_string(), scratch.path()]),
            Parsed::Run(_)
        ));
    }

    #[test]
    fn test_fast_and_no_cache_incompatibility() {
        let args1 = vec!["--fast".to_string(), "--no-cache".to_string()];
        if let Parsed::Run(cfg) = parse(&args1) {
            assert!(cfg.fast);
            assert!(!cfg.no_cache);
            assert!(cfg.warning.is_some());
            assert!(
                cfg.warning
                    .as_ref()
                    .unwrap()
                    .contains("--fast forcibly uses cache")
            );
        } else {
            panic!("Expected Parsed::Run");
        }

        let args2 = vec!["--no-cache".to_string(), "--fast".to_string()];
        if let Parsed::Run(cfg) = parse(&args2) {
            assert!(!cfg.fast);
            assert!(cfg.no_cache);
            assert!(cfg.warning.is_some());
            assert!(
                cfg.warning
                    .as_ref()
                    .unwrap()
                    .contains("--no-cache disables cache completely")
            );
        } else {
            panic!("Expected Parsed::Run");
        }
    }

    #[test]
    fn test_network_and_fast_incompatibility() {
        let args = vec!["--fast".to_string(), "--network".to_string()];
        match parse(&args) {
            Parsed::Error(e) => assert!(e.contains("cannot use --network with --fast")),
            other => panic!("expected Parsed::Error, got {other:?}"),
        }

        let args2 = vec!["--network".to_string(), "--fast".to_string()];
        match parse(&args2) {
            Parsed::Error(e) => assert!(e.contains("cannot use --network with --fast")),
            other => panic!("expected Parsed::Error, got {other:?}"),
        }
    }

    #[test]
    fn test_network_flag_enables_network() {
        let args = vec!["--network".to_string()];
        match parse(&args) {
            Parsed::Run(cfg) => assert!(cfg.network),
            other => panic!("expected Parsed::Run, got {other:?}"),
        }
    }

    #[test]
    fn network_config_defaults_are_populated() {
        let cfg = run("net_def", &[]);
        assert_eq!(cfg.network_cfg.weather_ip, "5.9.243.187");
        assert_eq!(cfg.network_cfg.weather_host, "wttr.in");
        assert_eq!(cfg.network_cfg.publicip_host, "myip.opendns.com");
        assert_eq!(cfg.network_cfg.publicip_fallback, "icanhazip.com");
    }

    #[test]
    fn network_config_overrides_defaults_from_toml() {
        let scratch = ScratchConfig::new("net_override");
        let toml_data = r#"
[network]
weather_ip = "1.2.3.4"
weather_host = "custom.weather.org"
publicip_host = "dns.custom.org"
publicip_fallback = "ip.custom.org"
"#;
        std::fs::write(&scratch.0, toml_data).unwrap();
        let cfg = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(c) => *c,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_eq!(cfg.network_cfg.weather_ip, "1.2.3.4");
        assert_eq!(cfg.network_cfg.weather_host, "custom.weather.org");
        assert_eq!(cfg.network_cfg.publicip_host, "dns.custom.org");
        assert_eq!(cfg.network_cfg.publicip_fallback, "ip.custom.org");
    }

    #[test]
    fn network_config_partial_override_keeps_other_defaults() {
        let scratch = ScratchConfig::new("net_partial");
        let toml_data = r#"
[network]
weather_host = "test.wttr.in"
"#;
        std::fs::write(&scratch.0, toml_data).unwrap();
        let cfg = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(c) => *c,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_eq!(cfg.network_cfg.weather_host, "test.wttr.in");
        assert_eq!(cfg.network_cfg.weather_ip, "5.9.243.187");
        assert_eq!(cfg.network_cfg.publicip_host, "myip.opendns.com");
        assert_eq!(cfg.network_cfg.publicip_fallback, "icanhazip.com");
    }

    #[test]
    fn test_flag_with_trailing_nbsp() {
        let args = vec!["--all".to_string(), "--network\u{a0}".to_string()];
        match parse(&args) {
            Parsed::Run(cfg) => {
                assert!(cfg.all);
                assert!(cfg.network);
            }
            _ => panic!("failed to parse flag with trailing nbsp"),
        }
    }

    #[test]
    fn test_module_overrides_top_level_and_tables() {
        let scratch = ScratchConfig::new("mod_overrides");
        let toml_data = r#"
os = "Bubuntu x228_1337"
gpu = "2x NVIDIA RTX 5090"

[values]
host = "WRX90 Workstation"

[format]
kernel = "7.1.6-zen-custom"
"#;
        std::fs::write(&scratch.0, toml_data).unwrap();
        let cfg = match parse(&["-c".to_string(), scratch.path()]) {
            Parsed::Run(c) => *c,
            other => panic!("expected Parsed::Run, got {other:?}"),
        };
        assert_eq!(cfg.format("os"), Some("Bubuntu x228_1337"));
        assert_eq!(cfg.format("gpu"), Some("2x NVIDIA RTX 5090"));
        assert_eq!(cfg.format("host"), Some("WRX90 Workstation"));
        assert_eq!(cfg.format("kernel"), Some("7.1.6-zen-custom"));
    }
}
