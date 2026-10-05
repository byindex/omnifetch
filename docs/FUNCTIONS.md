OMNIFETCH TEMPLATE FUNCTIONS AND VARIABLES
==========================================

Formatting strings live in ~/.config/omnifetch/config.toml, in the [format]
section. Any text outside {braces} is printed verbatim:

    memory = "{f} free of 1337GB"
    -> 4.24 GiB free of 1337GB

A module with a single value, such as os or host, prints that value as-is and
ignores [format].


1. MODULE VARIABLES
-------------------

memory
    {used} {u}           Used RAM, human units        3.37 GiB
    {total} {t}          Total RAM                    7.62 GiB
    {free} {f}           Free RAM
    {available} {a}      Available RAM, same as free
    {pct} {p}            Percent used, no sign        44
    {bar} {b}            Progress bar                 [#########-----]

swap
    {used} {u}           Used swap
    {total} {t}          Total swap
    {free} {f}           Free swap
    {pct} {p}            Percent used
    {bar} {b}            Progress bar

disk
    {mount} {m}          Mount point, padded to line up across disks
    {used} {u}           Used space
    {total} {t}          Total space
    {free} {f}           Free space
    {pct} {p}            Percent used
    {bar} {b}            Progress bar

cpu
    {name} {n}           Full model name
    {cores} {c}          Core layout                   2c/4t
    {freq} {f}           Current frequency             2592 MHz
    {mhz}                Frequency without the unit    2592
    {pcores}             Physical cores                2
    {lcores}             Logical cores                 4
    {microarch} {uarch}  Microarchitecture            Ivy Bridge
    {simd}               SIMD extensions               SSE4.2, AVX, AVX2
    {gflops}             Peak FP32 throughput          83.2 GFLOP/s
    {peak_gflops}        Same as above

cputemp
    {temp} {t}           Temperature with unit         54C
    {celsius} {c}        Celsius as a number           54
    {fahrenheit} {f}     Fahrenheit as a number        129
    {color}              Escape sequence: green under 60C, yellow under 80C,
                         red above

cpuusage
    {pct} {p}            Current utilisation
    {bar} {b}            Progress bar

battery
    {pct} {p}            Charge level                  98
    {status}             Status with leading space     (charging)
    {s}                 Short status                  Charging
    {charging}           true while charging, empty otherwise
    {discharging}        true while discharging, empty otherwise
    {wh}                 Watt-hours with brackets     [45/50 Wh]
    {energy_now}         Raw watt-hours now           45
    {energy_full}        Raw watt-hours full          50
    {raw_status}         Lowercase status             charging
    {bar} {b}            Progress bar

title
    {user} {u}           Current user name
    {host} {h}           Host name

uptime
    {uptime}             Human formatted              2h 15m
    {days} {d}           Days
    {hours} {h}          Hours
    {mins} {m}           Minutes
    {secs} {s}           Seconds
    {total_secs}         Seconds in total             8100

wifi
    {ssid}               Network name                  MyNetwork
    {signal} {pct} {p}   Signal quality                78
    {dbm}                Signal strength in dBm       -52
    {interface} {i}      Interface name                wlan0

netio
    {rx}                 Bytes received, human units
    {tx}                 Bytes sent, human units
    {total} {t}          Received plus sent

cursor
    {name} {n}           Cursor theme name
    {size} {s}           Cursor size in pixels

media
    {title}              Track title from the active MPRIS player
    {artist}             Track artist

quote
    {quote} {q}          The quote text
    {author} {a}         Who said it


2. FUNCTIONS
------------

A function can be written three ways, and all three reach the same code with the
same value:

    call     {upper(name)}        {trunc(name, 20)}     {bar(15, '=', '-')}
    filter   {name | upper}       {name | remove('(R)') | trunc(20)}
    colon    {used:gib}           {pct:round(1)}        {bar:15:=:-}

Chains work too: {name:remove('Intel(R) '):upper}

Unit converters always get the raw number rather than the formatted text, so
{used:gib} and {gib(used)} agree even though plain {used} prints "3.37 GiB".

Strings
    upper(s) / s:upper            UPPERCASE
    lower(s) / s:lower            lowercase
    title(s) / s:title            Capitalise Each Word

    upper, uppercase              alias for upper
    lower, lowercase              alias for lower
    title, capitalize             alias for title

    trunc(s, len) / s:trunc(len)  At most len characters, ellipsis included.
                                  Never splits a multi-byte character.
    truncate                      alias for trunc

    replace(s, old, new)          Every occurrence of old becomes new
    remove(s, pattern)            Every occurrence of pattern is deleted
    trim(s)                       Strips whitespace at both ends
    pad_left(s, width)            Pads on the left, text sits flush right
    pad_right(s, width)           Pads on the right, text sits flush left

Units and numbers
    gib(bytes) / s:gib            7.62 GiB
    mib(bytes) / s:mib            7802.5 MiB
    kib(bytes) / s:kib            7990000 KiB
    raw(bytes) / s:raw            Bare integer, no unit
    human(bytes) / s:human        Auto-scaled: B, KiB, MiB, GiB, TiB
    round(number, decimals)       Rounded to that many decimals

    gb, mb, kb                    aliases for gib, mib, kib
    bytes, b                      alias for raw

Bars
    bar(width)                    Progress bar, default width and characters
    bar(width, fill, empty)       Custom characters, quoted

    bar_color(width)              Bar coloured by percentage:
                                    green below 60
                                    yellow from 60 to 85
                                    red above 85

Fallbacks
    default(value, fallback)      fallback when value is empty
                                  {freq:default('N/A')}

System
    env(NAME)                     Reads an environment variable
    date(format)                  strftime, current date and time
    time(format)                  same as date

Colours and attributes
    {red} {green} {yellow} {blue} {magenta} {cyan} {white} {black}
    {bright_red} {bright_green} {bright_yellow} {bright_blue}
    {bright_magenta} {bright_cyan} {bright_white}
    {gray} and {grey} both mean {bright_black}
    {bold} {reset}


3. ARITHMETIC
-------------

Byte arithmetic, on memory, swap and disk
    {used + 2 GiB}                Adds 2 GiB, result in human units
    {used - 500 MiB}
    {used * 2}
    {used / 2}
    {used + 2%}                   2% of total, not of used

Percentages and plain numbers
    {pct + 5%}                    45% becomes 50%
    {pct - 2%}
    {pct * 1.5}

Changing {used} moves {bar} and {pct} with it, since both read the adjusted
metric rather than a recorded value.

Units accepted in arithmetic
    b    byte    bytes
    k    kb     kib
    m    mb     mib
    g    gb     gib
    t    tb     tib
    %    percent

Random ranges
    {used + 2..4 GiB}             A value between 2 and 4 GiB, different each
                                  run. Bar and percentage follow it.
    {pct + 2..5%}
    {2..4 GiB}                    Standalone random value


4. CONFIGURATION EXAMPLE
------------------------

    # ~/.config/omnifetch/config.toml

    [bar]
    width = 16
    fill = "■"
    empty = " "

    [keys]
    memory = "RAM"
    swap = "SWAP"
    disk = "Storage"
    cpu = "Processor"

    [format]
    memory = "{used + 2..4 GiB} / {total} {bar} {pct}%"
    swap = "{used * 2} / {total} {bar} {pct}%"
    disk = "{mount} {used} / {total} {bar} ({pct}%)"
    cpu = "{name | remove('Intel(R) ') | remove('Core(TM) ') | upper} ({cores}) @ {freq}"
    battery = "{bar(10)} {pct}%{status}"
    title = "{bold}{cyan}{user}{reset}@{bold}{host}{reset}"