# ==============================================================================
# Omnifetch Makefile
# ==============================================================================

PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
BASHCOMPDIR ?= $(PREFIX)/share/bash-completion/completions
ZSHCOMPDIR ?= $(PREFIX)/share/zsh/site-functions
FISHCOMPDIR ?= $(PREFIX)/share/fish/vendor_completions.d

HOST := $(shell rustc -vV | sed -n 's/host: //p')

# The binary is linked statically by default: skipping the dynamic loader is
# worth roughly 0.8 ms per run, which is a large share of the total runtime.
# It needs an explicit --target, otherwise crt-static would also be handed to
# proc-macro crates, which cannot be built that way.
#
# `make build` follows the dynamic build instead, which is handy when debugging
# or when a dependency needs a dynamic libssl/libcurl.
STATIC ?= 1

ifeq ($(STATIC),1)
CARGO_TARGET_FLAGS := --target "$(HOST)"
RUSTFLAGS_ENV := RUSTFLAGS="-C target-feature=+crt-static -C force-unwind-tables=no -C link-arg=-Wl,--icf=all"
BINPATH := target/$(HOST)/release/omnifetch
else
CARGO_TARGET_FLAGS :=
RUSTFLAGS_ENV := RUSTFLAGS="-C force-unwind-tables=no -C link-arg=-Wl,--icf=all"
BINPATH := target/release/omnifetch
endif

# Falls back to whichever layout is actually on disk, so `make install` still
# works after a plain `cargo build`.
TARGET ?= $(shell test -x $(BINPATH) && echo $(BINPATH) || (test -x target/release/omnifetch && echo target/release/omnifetch || ls -1 target/*/release/omnifetch 2>/dev/null | head -1))

.PHONY: all build build-dynamic test clean install uninstall

all: build

build:
	$(RUSTFLAGS_ENV) cargo build --release $(CARGO_TARGET_FLAGS)

build-dynamic:
	$(MAKE) build STATIC=0

test:
	cargo test

clean:
	cargo clean

install:
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 755 $(TARGET) "$(DESTDIR)$(BINDIR)/omnifetch"
	install -d "$(DESTDIR)$(BASHCOMPDIR)"
	$(TARGET) --completion bash > "$(DESTDIR)$(BASHCOMPDIR)/omnifetch"
	install -d "$(DESTDIR)$(ZSHCOMPDIR)"
	$(TARGET) --completion zsh > "$(DESTDIR)$(ZSHCOMPDIR)/_omnifetch"
	install -d "$(DESTDIR)$(FISHCOMPDIR)"
	$(TARGET) --completion fish > "$(DESTDIR)$(FISHCOMPDIR)/omnifetch.fish"

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/omnifetch"
	rm -f "$(DESTDIR)$(BASHCOMPDIR)/omnifetch"
	rm -f "$(DESTDIR)$(ZSHCOMPDIR)/_omnifetch"
	rm -f "$(DESTDIR)$(FISHCOMPDIR)/omnifetch.fish"
