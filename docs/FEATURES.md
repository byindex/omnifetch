# Features - omnifetch

## Modules

88 modules in the registry (86 run under `--all`, while `publicip` and `weather` sit behind `--network`). From core system information down to individual pieces of hardware:

- **System**: `os`, `kernel`, `bootmgr`, `initsystem`, `security`, `host`, `board`, `bios`, `tpm`, `chassis`, `uptime`, `loadavg`, `processes`, `packages`, `datetime`, `locale`, `users`, `version`
- **Shell**: `shell`, `terminal`, `terminalfont`, `terminalsize`, `terminaltheme`, `editor`, `wm`, `wmtheme`, `de`, `theme`, `icons`, `font`, `cursor`, `wallpaper`
- **Hardware**: `cpu`, `cpucache`, `cputemp`, `cpuusage`, `powerprofile`, `poweradapter`, `gpu`, `gpudriver`, `vulkan`, `opengl`, `opencl`, `memory`, `swap`, `disk`, `physicaldisk`, `physicalmemory`, `diskio`, `brightness`, `battery`, `display`, `displayserver`, `resolution`
- **Peripherals**: `keyboard`, `mouse`, `touchpad`, `camera`, `gamepad`, `media`, `player`, `bluetooth`, `bluetoothradio`, `monitor`, `audioserver`, `sound`
- **Network**: `netadapter`, `localip`, `dns`, `wifi`, `netio`, `publicip`, `weather`
- **System Detail**: `btrfs`, `zpool`, `codec`, `lm`, `containers`, `command`, `custom`, `top`
- **Content**: `quote`, `git`, `devenv`
- **Layout/Style**: `title`, `separator`, `break`, `colors`

The authoritative list with current values: `omnifetch --list-modules`

## Output Shape

- `--logo`, `--logo-mini`, `--logo-top` — Choose or move the logo
- `--border`, `--border-title` — Categorised rounded box with custom header alignment
- `--nerd`, `--nerd-only` — Nerd Font glyphs instead of text keys
- `--theme`, `--gradient` — 6 themes, 8 Truecolor gradients
- `--image`, `--image-cols`, `--image-rows` — PNG and Sixel through the Kitty Graphics Protocol
- `--json` — Raw JSON output for scripting
- `--timing` — Per-module cost, printed to stderr

## Layout

- 596 ASCII logos for distributions and programming languages, full-size and mini, indexed by binary search rather than a linear scan.
- 14 presets, eight of which mirror a competitor's default output: [PRESETS.md](PRESETS.md)

## Configuration

- One file drives everything: which modules run, in what order, under which labels, in which colours, with which bar widths and output templates.
- `--gen-config` builds it through an interactive TUI.
- The templating language has filters, unit converters, arithmetic and random ranges: [FUNCTIONS.md](FUNCTIONS.md)

## Scripting and Diagnostics

- `--json` — Flat array of name and data pairs
- `--timing` — Per-module timings on stderr
- `--no-cache` — Bypass the cache entirely
- `--network` — Enable the two modules that need internet
- `--completion` — Shell completions for `bash`, `zsh`, `fish`