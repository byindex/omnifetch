INSTALLATION AND BUILD - omnifetch
==================================

BUILDING
--------

    git clone https://github.com/byindex/omnifetch
    cd omnifetch
    make
    sudo make install

make install places the binary and the completions for bash, zsh and fish.
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