# Omnifetch

**English** · [Русский](README_ru.md)

> A fast, highly configurable and good-looking system information fetcher for Linux, written in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/omnifetch-rs.svg)](https://crates.io/crates/omnifetch-rs)
[![Rust](https://img.shields.io/badge/Rust-2024%2B-orange.svg)](https://www.rust-lang.org/)
[![Speed](https://img.shields.io/badge/warm%20cache-2.0ms-brightgreen.svg)](benchmark_results.md)
[![Repo](https://img.shields.io/badge/github-byindex%2Fomnifetch-8da0cb?logo=github)](https://github.com/byindex/omnifetch)

<p align="center">
  <img src="assets/preview.png" alt="Omnifetch Preview" width="850">
</p>

---

## What it is

**88 modules** — from OS and kernel down to mice, touchpads, webcams, gamepads, Bluetooth,
and quotes. The kernel is read directly through `/proc`, `/sys`, DMI, DRM and sockets, with
zero `fork`/`exec` on the standard path. Full list: `omnifetch --list-modules`, per-module notes in
[docs/FEATURES.md](docs/FEATURES.md).

**The config is the main feature.** Which modules run, in what order, under which labels,
colours, bar widths and output templates all live in one file, `~/.config/omnifetch/config.toml`,
assembled interactively through `omnifetch --gen-config` and applied without a restart.
Details: [docs/CONFIG.md](docs/CONFIG.md) and [docs/FUNCTIONS.md](docs/FUNCTIONS.md).

**A cache in `/dev/shm`** holds whatever does not change between runs and reads back in
0.000 ms. It clears itself on reboot, on package install and on a config edit. Dynamic
peripherals are still probed live, in roughly 0.2 ms. How that works:
[docs/DESIGN.md](docs/DESIGN.md).

**2.0 ms** warm, **11-13 ms** for all modules uncached (up to 20 ms on cold disk IO).

**14 layout presets**, eight of them reproducing a competitor's output one to one, so
speed can be compared on identical data: [docs/PRESETS.md](docs/PRESETS.md).

**596 ASCII logos** with binary search, 6 themes, 8 gradients, Nerd Fonts, Kitty and Sixel
graphics, SVG/HTML and JSON export, per-module timings, completions for bash, zsh and fish.

## Installation

### Quick Install (Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/byindex/omnifetch/main/install.sh | bash
```

### Cargo (crates.io)

```bash
cargo install omnifetch-rs
```
*(installs the `omnifetch` binary to your PATH)*

### Arch Linux (AUR)

Packages `omnifetch-bin` and `omnifetch-git` will be published to AUR as soon as new registrations open.

### Building from Source

```bash
git clone https://github.com/byindex/omnifetch
cd omnifetch
make
sudo make install
```

Only `rustc` is required. The build is static; for a dynamic one use `make build-dynamic`.
Details: [docs/INSTALL.md](docs/INSTALL.md).

## Usage

```bash
omnifetch                          # default output
omnifetch --fast                   # core modules only
omnifetch --all                    # everything
omnifetch -m host,gpu,devenv       # pick your own modules
omnifetch -p fastfetch             # same as fastfetch
omnifetch --theme nord             # built-in theme
omnifetch --json | jq .            # script-friendly
```

All flags: [docs/CLI.md](docs/CLI.md). Presets: [docs/PRESETS.md](docs/PRESETS.md).

## Configuration

Path: `$OMNIFETCH_CONFIG`, then `$XDG_CONFIG_HOME/omnifetch/config.toml`,
then `~/.config/omnifetch/config.toml`. `-c` overrides all of them.

```bash
omnifetch --gen-config              # interactive TUI generator
```

File reference and TUI keys: [docs/CONFIG.md](docs/CONFIG.md).
Templating variables and functions: [docs/FUNCTIONS.md](docs/FUNCTIONS.md).

## Benchmarks

<p align="center">
  <img src="assets/benchmark.png" alt="Omnifetch Benchmarks" width="850">
</p>

- Details — [benchmark_results.md](benchmark_results.md)
- Reproduce — [benchmark.py](benchmark.py)

## Roadmap

- [ ] **Extended Configuration Capabilities:**
  - Conditional expressions (`if` / `else`) in the templating engine for conditionally displaying rows or altering formatting based on metric values.
  - Extended environment variables and custom script integrations with output caching.
- [ ] **Config Converter from Other Fetchers:**
  - Built-in migration tool from other fetch tools (Fastfetch `config.jsonc`, Neofetch `config.conf`) into native `omnifetch/config.toml`.
- [ ] **Distribution Packaging:**
  - Publish to package repositories (Arch AUR, Alpine aports, Debian/Ubuntu PPA).

### Out of Scope (Won't Do):
- ❌ **Cross-platform support (Windows / macOS):** Omnifetch is built exclusively for Linux. The engine relies directly on native Linux kernel interfaces (`/proc`, `/sys`, DMI, DRM) and raw Linux syscalls with zero abstraction overhead. Supporting other operating systems would compromise speed, bloat the codebase, and dilute the project's core philosophy (possibly in the distant future, but definitely not on the roadmap right now).

## Documentation

- [docs/DESIGN.md](docs/DESIGN.md) — why it is fast
- [docs/FEATURES.md](docs/FEATURES.md) — every feature
- [docs/INSTALL.md](docs/INSTALL.md) — installation and build
- [docs/USAGE.md](docs/USAGE.md) — the command line in practice
- [docs/CLI.md](docs/CLI.md) — every flag
- [docs/PRESETS.md](docs/PRESETS.md) — layout presets
- [docs/CONFIG.md](docs/CONFIG.md) — config file and TUI
- [docs/FUNCTIONS.md](docs/FUNCTIONS.md) — templating
- **CLI** — `omnifetch --help`, `--list-modules`, `--list-presets`, `--list-themes`

## Credits

ASCII logos are taken and adapted from [Fastfetch](https://github.com/fastfetch-cli/fastfetch). Ideas and output formats are inspired by [Onefetch](https://github.com/o2sh/onefetch), [Cpufetch](https://github.com/Dr-Noob/cpufetch), [Hyfetch](https://github.com/hykilpikonna/hyfetch), [Nitch](https://github.com/ssleert/nitch), [Catnap](https://github.com/iinsertNameHere/catnap), [Macchina](https://github.com/Macchina-CLI/macchina), [Pfetch](https://github.com/dylanaraps/pfetch), and [Neofetch](https://github.com/dylanaraps/neofetch).

Details in [CREDITS.md](CREDITS.md).

## License

MIT