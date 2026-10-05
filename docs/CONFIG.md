CONFIGURATION - omnifetch
=========================

Template variables and functions: FUNCTIONS.md

WHERE THE FILE LIVES
--------------------

config.toml is looked for in this order, first match wins:

  1. the path in $OMNIFETCH_CONFIG
  2. $XDG_CONFIG_HOME/omnifetch/config.toml
  3. ~/.config/omnifetch/config.toml

The -c or --config flag takes a path directly and overrides all of the above.

INTERACTIVE GENERATOR
---------------------

The module list, its order and the display settings can be assembled in a TUI:

    omnifetch --gen-config                interactive, saves to the usual path
    omnifetch --gen-config /path/to.toml  saves to that path
    omnifetch --gen-config -              prints the config to stdout
    omnifetch --gen-config-force          overwrites without asking

Keys inside the TUI:

  Up Down Left Right, or k j h    move between modules
  Space                            toggle a module, removes break and separator
  K and J                          move the module up or down in the output
  b and B                          insert a blank line, or a separator
  d                                delete the selected separator or break
  f and F                          select all, or invert the selection
  l                                logo kind: default, small, none
  p                                logo position: auto, left, right, top
  o                                output mode: minimal or full
  s or Enter                       save
  q or Esc                         leave without saving

AN EXAMPLE CONFIG
-----------------

    logo = "auto"          # auto, mini, none, or a distro id
    fast = false           # minimal module set
    cache = true           # false turns the /dev/shm cache off
    color = true

    modules = ["title", "separator", "os", "host", "kernel", "uptime",
               "packages", "shell", "cpu", "gpu", "memory", "disk"]

    [bar]
    width = 20
    fill = "█"
    empty = "░"

    [keys]                # labels of your own instead of the defaults
    memory = "RAM"
    disk = "Storage"

    [style]
    key_color = "cyan"     # a colour name or hex, for example #89b4fa
    title_color = "bright_black"
    bold_key = true

    [format]              # an output template of your own, per module
    memory = "{used} / {total} {bar} {pct}%"
    disk = "{mount} {used} / {total} ({pct}%)"
    cpu = "{name | remove('Intel(R) ') | remove('Core(TM) ') | upper}"

    [network]             # endpoints for internet-dependent modules
    weather_ip = "5.9.243.187"          # empty string means DNS only
    weather_host = "wttr.in"
    publicip_host = "myip.opendns.com"
    publicip_fallback = "icanhazip.com"

TEXT OUTSIDE BRACES
-------------------

Anything not inside braces is printed as written. Inside braces you get the
module's variables, filter functions, arithmetic and random ranges:

    [format]
    memory = "{used + 2..4 GiB} / {total} {bar} {pct}%"   # random offset
    pct    = "{pct:round(1)}%"                            # 44.1%

NETWORK SETTINGS
----------------

The network section defines remote endpoints used by internet modules:

    [network]
    weather_ip = "5.9.243.187"          # empty string means DNS only
    weather_host = "wttr.in"
    publicip_host = "myip.opendns.com"
    publicip_fallback = "icanhazip.com"

CUSTOM AND COMMAND MODULES
--------------------------

To use the `custom` and `command` modules:
1. Include `"custom"` and/or `"command"` in your `modules` list.
2. Specify what to display under `[format]`:

    [format]
    custom = "My custom information"
    command = "uname -r"