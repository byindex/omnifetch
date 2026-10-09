# Configuration - omnifetch

Template variables and functions: [FUNCTIONS.md](FUNCTIONS.md)

## Where the File Lives

`config.toml` is looked for in this order (first match wins):

1. The path in `$OMNIFETCH_CONFIG`
2. `$XDG_CONFIG_HOME/omnifetch/config.toml`
3. `~/.config/omnifetch/config.toml`

The `-c` or `--config` flag takes a path directly and overrides all of the above.

## Interactive Generator

The module list, its order and display settings can be assembled interactively in a TUI:

```bash
omnifetch --gen-config                # interactive, saves to ~/.config/omnifetch/config.toml
omnifetch --gen-config /path/to.toml  # saves to specified path
omnifetch --gen-config -              # prints generated config to stdout
omnifetch --gen-config-force          # overwrites without confirmation
```

### Keys Inside the TUI

- `↑`, `↓`, `←`, `→` (or `k`, `j`, `h`, `l`) — Move between modules
- `Space` — Toggle module on/off (or remove separator/break)
- `/` — Search modules by name, ID, category or description (`n` / `N`: next / prev)
- `[` and `]`, `-` and `=`, or `K` / `J` — Move selected module up or down in the output
- `c` — Group / sort modules by category (System, Visual, Hardware...)
- `v` — Live omnifetch preview of current configuration
- `r` — Reset all modules and settings to defaults (with confirmation)
- `b` and `B` — Insert a blank line (`break`) or a `separator`
- `d` — Delete the selected separator or break
- `f` and `F` — Select all or invert selection
- `l` — Logo kind: `default`, `small`, `none`
- `p` — Logo position: `auto`, `left`, `right`, `top`
- `o` — Output mode: `minimal` or `full`
- `s` or `Enter` — Save configuration
- `q` or `Esc` — Leave without saving

## Example Configuration

```toml
logo = "auto"          # auto, mini, none, or a distro id
fast = false           # minimal module set
cache = true           # false turns the /dev/shm cache off
color = true

modules = [
    "title", "separator", "os", "host", "kernel", "uptime",
    "packages", "shell", "cpu", "gpu", "memory", "disk"
]

[bar]
width = 20
fill = "█"
empty = "░"

[keys]                 # custom labels instead of defaults
memory = "RAM"
disk = "Storage"

[style]
key_color = "cyan"     # color name or hex (#89b4fa)
title_color = "bright_black"
bold_key = true

[format]               # custom output templates per module
memory = "{used} / {total} {bar} {pct}%"
disk = "{mount} {used} / {total} ({pct}%)"
cpu = "{name | remove('Intel(R) ') | remove('Core(TM) ') | upper}"

[network]              # endpoints for internet-dependent modules
weather_ip = "5.9.243.187"          # empty string means DNS only
weather_host = "wttr.in"
publicip_host = "myip.opendns.com"
publicip_fallback = "icanhazip.com"
```

## Text Outside Braces

Anything not inside braces is printed as written. Inside braces you get the module's variables, filter functions, arithmetic and random ranges:

```toml
[format]
memory = "{used + 2..4 GiB} / {total} {bar} {pct}%"   # random offset
pct    = "{pct:round(1)}%"                            # 44.1%
```

## Overriding Module Output

You can override or customize the output of any module (useful for testing setups or custom output):

- Directly at the top level of the config file: `os = "Bubuntu x228_1337"`
- In the `[format]` section: `os = "Bubuntu x228_1337"` or `gpu = "2 × ASUS ROG Strix GeForce RTX 5090"`
- In the `[values]` section: `host = "ASUS Pro WS WRX90E-SAGE SE"`

Even if a module does not detect any hardware on the current machine, the specified value will be rendered.

## Unquoted Identifiers & Bare Words

omnifetch supports clean, unquoted identifiers (`bare words`) for global settings, modes, colors, themes, and module lists:

```toml
# Global options without quotes:
logo = auto               # auto, mini, none
theme = dracula           # Sets global omnifetch color palette
preset = modern           # modern, compact, etc.
modules = [ os, kernel, uptime, break, shell, terminal, cpu, memory, disk, colors ]

[style]
theme = nord
key_color = cyan
title_color = magenta

# Quoted strings are used for custom text and module overrides:
os = "Bubuntu x228_1337"
theme = "Adwaita-Dark"    # Quoted string overrides the desktop theme module output!
host = "WRX90 Workstation"
```

### Theme Setting vs `theme` Module

To avoid naming conflicts between the global color palette and the desktop GTK/Qt `theme` module:
- `theme = dracula` (**unquoted bare identifier**) sets the global color palette of omnifetch.
- `theme = "Adwaita-Dark"` (**quoted string**) overrides the output of the desktop `theme` module.
- Inside `[style]`, `theme = dracula` sets the UI theme palette.
- Inside `[values]` or `[format]`, `theme = "Adwaita-Dark"` overrides the module value.

## Custom and Command Modules

To use the `custom` and `command` modules:
1. Include `"custom"` and/or `"command"` in your `modules` list.
2. Specify what to display under `[format]`:

```toml
[format]
custom = "My custom information"
command = "uname -r"
```

## Border Settings

Wrap output in unicode border boxes with optional category titles:

```toml
border = true                 # shorthand to enable borders

[border]
enabled = true
title_align = "left"          # "left", "center", or "right"
```

On the command line:

```bash
omnifetch --border
omnifetch --border-title center
omnifetch --border-title right
```