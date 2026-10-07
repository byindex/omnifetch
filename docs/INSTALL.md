# Installation and Build - omnifetch

## 1. Cargo (crates.io)

The simplest and fastest way to install omnifetch:

```bash
cargo install omnifetch-rs
```

This compiles and installs the binary `omnifetch` into `~/.cargo/bin`. Make sure `~/.cargo/bin` is in your `$PATH`.

## 2. Arch Linux (AUR)

Precompiled binary package:

```bash
yay -S omnifetch-bin
# or: paru -S omnifetch-bin
```

Built from the latest git commit:

```bash
yay -S omnifetch-git
# or: paru -S omnifetch-git
```

## 3. Precompiled Binaries (GitHub Releases)

Statically linked release archives with shell completions are available on [GitHub Releases](https://github.com/byindex/omnifetch/releases).

### Automated Install

```bash
tar -xzf omnifetch-v*-linux-x86_64.tar.gz
cd omnifetch-v*-linux-x86_64
sudo ./install.sh
```

Or install directly in one command via curl:

```bash
curl -fsSL https://raw.githubusercontent.com/byindex/omnifetch/main/install.sh | bash
```

### Manual Install

```bash
sudo install -Dm755 omnifetch /usr/local/bin/omnifetch
sudo ./setup-aliases.sh   # optional shell aliases wizard

# Optional: install shell completions
sudo install -Dm644 completions/omnifetch.bash /usr/share/bash-completion/completions/omnifetch
sudo install -Dm644 completions/_omnifetch /usr/share/zsh/site-functions/_omnifetch
sudo install -Dm644 completions/omnifetch.fish /usr/share/fish/vendor_completions.d/omnifetch.fish
```

## 4. Building from Source

```bash
git clone https://github.com/byindex/omnifetch
cd omnifetch
make
sudo make install
```

`make install` places the binary and completions for `bash`, `zsh`, and `fish`.  
`sudo make uninstall` removes them cleanly.

Only `rustc` and `cargo` (with network access to fetch the single dependency `libc`) are required. There are no system libraries to install first, and no runtime dependencies, because the binary is linked statically.

## Static and Dynamic Builds

The default build is static, which skips the dynamic loader on every run and saves roughly 0.8 ms. That is a noticeable share of the total runtime, since the whole program starts in about 2 ms.

For a standard dynamic build (e.g. for debugging or distro packaging):

```bash
make build-dynamic
```

The two builds land in different directories:

- `make` — static, `target/<triple>/release/omnifetch`
- `make build-dynamic` — dynamic, `target/release/omnifetch`

## What Gets Installed

```text
<prefix>/bin/omnifetch
<prefix>/share/bash-completion/completions/omnifetch
<prefix>/share/zsh/site-functions/_omnifetch
<prefix>/share/fish/vendor_completions.d/omnifetch.fish
```

`PREFIX` defaults to `/usr/local` and can be overridden at install time:

```bash
make install PREFIX=/usr
```

`DESTDIR` is honoured for staged packaging:

```bash
make install DESTDIR=/tmp/stage PREFIX=/usr
```