COMMAND LINE IN PRACTICE - omnifetch
====================================

EVERYDAY COMMANDS
-----------------

    omnifetch                          default layout, logo auto-detected
    omnifetch --fast                   core modules only, maximum speed
    omnifetch --all                    every local module
    omnifetch -m host,gpu,devenv       pick your own modules
    omnifetch --theme nord             apply a built-in theme
    omnifetch --gradient cyberpunk     gradient across the logo
    omnifetch --logo-top --border      logo above, everything boxed
    omnifetch --git                    statistics for the current git repo
    omnifetch --json | jq .            machine readable
    omnifetch --timing                 what each module cost

PICKING A PRESET
----------------

    omnifetch -p fastfetch             same modules as fastfetch
    omnifetch -p neofetch              same modules as neofetch
    omnifetch -p detailed              34 modules
    omnifetch -p all                   86 modules

PRESETS AND MODULE COUNTS: PRESETS.md
EVERY FLAG: CLI.md

SCRIPTING
---------

--json prints a flat array of name and data pairs instead of a table:

    [
      { "name": "OS",  "data": "Arch Linux rolling [x86_64]" },
      { "name": "CPU", "data": "Intel(R) Core(TM) i5-3360M ..." }
    ]

Pulling one field out of it:

    omnifetch --json --no-color | jq -r '.[] | select(.name=="OS") | .data'
    omnifetch -m os,cpu --json --no-color | jq -r '.[].data'

--no-color and NO_COLOR=1 both drop escape sequences. --timing writes to
stderr, so it does not pollute a --json pipe. Closing the pipe early, the way
head or jq does, stops the program quietly instead of failing.

FINDING OUT WHY SOMETHING IS SLOW
----------------------------------

--timing prints one line per module to stderr, which is where to look when a
module takes longer than expected:

    omnifetch --no-cache --timing >/dev/null

A module listed as empty produced no output on this machine. That usually means
the hardware is not present rather than a failure.