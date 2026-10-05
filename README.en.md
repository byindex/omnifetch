# Omnifetch

**English** · [Русский](README.md)

> A fast, highly configurable and good-looking system information fetcher for Linux, written in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024%2B-orange.svg)](https://www.rust-lang.org/)
[![Speed](https://img.shields.io/badge/warm%20cache-2.0ms-brightgreen.svg)](https://github.com/byindex/omnifetch)
[![Repo](https://img.shields.io/badge/github-byindex%2Fomnifetch-8da0cb?logo=github)](https://github.com/byindex/omnifetch)

<p align="center">
  <img src="assets/preview.png" alt="Omnifetch Preview" width="850">
</p>

---

## Benchmarks

<p align="center">
  <img src="assets/benchmark.png" alt="Omnifetch Benchmarks" width="850">
</p>

Measured on Arch Linux with hyperfine, static build, lower is better. Run
`python3 benchmark.py` to reproduce on your own machine:
[benchmark_results.md](benchmark_results.md).

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

**2.0 ms** warm, **11-13 ms** for all modules uncached (up to 20 ms on cold disk IO). Measurements and how to
repeat them: `python3 benchmark.py`, results in
[benchmark_results.md](benchmark_results.md).

**14 layout presets**, eight of them reproducing a competitor's output one to one, so
speed can be compared on identical data: [docs/PRESETS.md](docs/PRESETS.md).

**596 ASCII logos** with binary search, 6 themes, 8 gradients, Nerd Fonts, Kitty and Sixel
graphics, JSON export, per-module timings, completions for bash, zsh and fish.

## Installation

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

ASCII logos are taken and adapted from [Fastfetch](https://github.com/fastfetch-cli/fastfetch). Ideas and output formats are inspired by [Onefetch](https://github.com/o2sh/onefetch), [Cpufetch](https://github.com/robinhargreaves/cpufetch), [Hyfetch](https://github.com/andreansaraiva/hyfetch), [Nitch](https://github.com/nicolatos/nitch), [Catnap](https://github.com/lvyaoyu/catnap), [Macchina](https://github.com/SeptemberFoxworth/macchina), [Pfetch](https://github.com/dvander/pub), and [Neofetch](https://github.com/dylanaraps/neofetch).

Details in [CREDITS.md](CREDITS.md).

## License

MIT