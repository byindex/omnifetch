FEATURES - omnifetch
====================

MODULES
-------

88 modules in the registry. 86 run under --all, the remaining two sit behind
--network. From core system information down to individual pieces of hardware
and the software around them.

  System        os, kernel, bootmgr, initsystem, security, host, board, bios,
                tpm, chassis, uptime, loadavg, processes, packages, datetime,
                locale, users, version

  Shell         shell, terminal, terminalfont, terminalsize, terminaltheme,
                editor, wm, wmtheme, de, theme, icons, font, cursor, wallpaper

  Hardware      cpu, cpucache, cputemp, cpuusage, powerprofile, poweradapter,
                gpu, gpudriver, vulkan, opengl, opencl, memory, swap, disk,
                physicaldisk, physicalmemory, diskio, brightness, battery,
                display, displayserver, resolution

  Peripherals   keyboard, mouse, touchpad, camera, gamepad, media, player,
                bluetooth, bluetoothradio, monitor, audioserver, sound

  Network       netadapter, localip, dns, wifi, netio, publicip, weather

  System detail btrfs, zpool, codec, lm, containers, command, custom, top

  Content       quote, git, devenv

  Layout/Style  title, separator, break, colors

The authoritative list with current values: omnifetch --list-modules

OUTPUT SHAPE
------------

  --logo, --logo-mini, --logo-top       choose or move the logo
  --border                             categorised rounded box
  --nerd, --nerd-only                  Nerd Font glyphs instead of text keys
  --theme, --gradient                  6 themes, 8 Truecolor gradients
  --image, --image-cols, --image-rows  PNG and Sixel through the Kitty
                                      Graphics Protocol
  --json                               raw JSON for scripts
  --timing                             per-module cost, on stderr

LAYOUT
------

  596 ASCII logos for distributions and programming languages, full-size and mini,
  indexed by binary search rather than a linear scan.
  14 presets, eight of which mirror a competitor's default output: PRESETS.md

CONFIGURATION
-------------

  One file drives everything: which modules run, in what order, under which
  labels, in which colours, with which bar widths and output templates.
  --gen-config builds it through an interactive TUI.
  The templating language has filters, unit converters, arithmetic and random
  ranges: FUNCTIONS.md

SCRIPTING AND DIAGNOSTICS
-------------------------

  --json             flat array of name and data pairs
  --timing           per-module timings on stderr
  --no-cache         bypass the cache entirely
  --network          enable the two modules that need internet
  --completion       bash, zsh, fish