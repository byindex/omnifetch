INSTALLATION AND BUILD - omnifetch
==================================

1. CARGO (CRATES.IO)
--------------------

The simplest and fastest way to install omnifetch:

    cargo install omnifetch-rs

This compiles and installs the binary `omnifetch` into `~/.cargo/bin`. Make sure `~/.cargo/bin` is in your `$PATH`.

2. ARCH LINUX (AUR)
-------------------

Precompiled binary package:

    yay -S omnifetch-bin
    # or: paru -S omnifetch-bin

Built from the latest git commit:

    yay -S omnifetch-git
    # or: paru -S omnifetch-git

3. PRECOMPILED BINARIES (GITHUB RELEASES)
-----------------------------------------

Statically linked release archives with shell completions are available on GitHub:
https://github.com/byindex/omnifetch/releases

Download and install:

    tar -xzf omnifetch-v*-linux-x86_64.tar.gz
    cd omnifetch-v*-linux-x86_64
    sudo install -Dm755 omnifetch /usr/local/bin/omnifetch
    sudo ./setup-aliases.sh   # optional shell aliases wizard

4. BUILDING FROM SOURCE
-----------------------

    git clone https://github.com/byindex/omnifetch
    cd omnifetch
    make
    sudo make install

make install places the binary and completions for bash, zsh, and fish.
sudo make uninstall removes them again. Nothing else is written.

Only rustc and cargo (with network access to fetch the single dependency libc)
are required. There are no system libraries to install first, and no runtime
dependencies, because the binary is linked statically.

STATIC AND DYNAMIC
------------------

The default build is static, which skips the dynamic loader on every run and
saves roughly 0.8 ms. That is a noticeable share of the total runtime, since the
whole program starts in about 2 ms.

For a standard dynamic build (e.g. for debugging or distro packaging):

    make build-dynamic

The two builds land in different directories. The installer picks whichever one
is present, so switching between them needs no extra step:

    make                        static, target/<triple>/release/omnifetch
    make build-dynamic          dynamic, target/release/omnifetch

WHAT GETS INSTALLED
-------------------

    <prefix>/bin/omnifetch
    <prefix>/share/bash-completion/completions/omnifetch
    <prefix>/share/zsh/site-functions/_omnifetch
    <prefix>/share/fish/vendor_completions.d/omnifetch.fish

PREFIX defaults to /usr/local and can be overridden at install time:

    make install PREFIX=/usr

DESTDIR is honoured for staged installs:

    make install DESTDIR=/tmp/stage PREFIX=/usr