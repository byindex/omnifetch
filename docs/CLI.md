COMMAND LINE REFERENCE - omnifetch
==================================

Short form first, where there is one. Values in angle brackets are required
unless stated otherwise.


GLOBAL
------

  -h, --help                   Print this help
  -V, --version                Print version


OUTPUT SHAPE
------------

  -l, --logo <NAME>            Logo: auto (default), none, mini, or a distro id
      --logo-mini              Use the mini ASCII logo
      --logo-top               Render the logo above the table instead of beside it
      --border                 Wrap the output in a unicode border
      --nerd                   Prefix module keys with Nerd Font icons
      --nerd-only              Show only the icons, hide the text keys
  -j, --json                   Print raw JSON instead of a table
      --no-color               Disable colours, same as NO_COLOR=1
  -T, --timing                 Print per-module timings to stderr
      --no-cache               Disable caching completely


COLOUR AND STYLE
----------------

  -t, --theme <NAME>           catppuccin, tokyo-night, nord, gruvbox,
                               dracula, rose-pine
      --list-themes            List the built-in themes and exit
      --gradient <NAME>        rainbow, sunset, cyberpunk, synthwave, fire,
                               ice, matrix, dracula
      --completion <SH>        Shell completion: bash, zsh, fish


WHAT TO RUN
-----------

  -f, --fast                   Minimal module set, maximum speed
  -a, --all                    Every available module, same as -p all
  -p, --preset <NAME>          Layout preset, see --list-presets
      --list-presets           List presets with descriptions and exit
  -m, --modules <LIST>         Comma-separated module ids
      --list-modules           Print every module id and exit
      --network                Enable the two modules that need internet:
                               publicip and weather
      --git                    Git statistics for the current repository
      --quotes-file <PATH>     Custom quotes, JSON or plain text


IMAGES
------

  -i, --image <PATH>           PNG or Sixel through the Kitty Graphics
                               Protocol
      --image-cols <NUM>       Width in terminal cells, default 34
      --image-rows <NUM>       Height in terminal lines, default 17


CONFIGURATION
-------------

  -c, --config <PATH>          Load modules and logo from a TOML file
      --gen-config <?PATH>     Interactive TUI generator. A bare --gen-config
                               writes to ~/.config/omnifetch/config.toml, a
                               path writes there, and "-" prints to stdout
      --gen-config-force       Overwrite an existing file without asking


EXAMPLES
--------

  omnifetch                          default layout, logo auto-detected
  omnifetch --logo mini --border     mini logo inside a box
  omnifetch --theme tokyo-night      apply a colour theme
  omnifetch --gradient cyberpunk     gradient across the logo
  omnifetch --nerd                   Nerd Font icons
  omnifetch -p modern                a preset
  omnifetch -p detailed              34 modules
  omnifetch -p fastfetch             same modules as fastfetch
  omnifetch -p neofetch              same modules as neofetch
  omnifetch --network                add public IP and weather
  omnifetch --all --no-cache         every module, nothing cached
  omnifetch --fast                   fastest mode

Panels are categorised as System, Visual, Hardware, Network, Devices and Quote.