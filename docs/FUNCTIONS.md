# Omnifetch Template Functions and Variables

Formatting strings live in `~/.config/omnifetch/config.toml`, in the `[format]` section. Any text outside `{braces}` is printed verbatim:

```toml
[format]
memory = "{f} free of 1337GB"
# -> 4.24 GiB free of 1337GB
```

Any module (including `os`, `host`, `kernel`, `gpu`, `chassis`, etc.) can be overridden with arbitrary text or customized with a template via `[format]`, `[values]`, or directly at the top level of the config file (e.g. `os = "Bubuntu x228_1337"`).

---

## 1. Module Variables

### `os`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{name}`, `{n}` | Distribution name | `Arch Linux` |
| `{version}`, `{v}` | Distribution version | `rolling` |
| `{arch}`, `{a}` | System architecture | `x86_64` |
| `{id}` | Distribution identifier | `arch` |
| `{val}`, `{value}` | Original full value | `Arch Linux rolling [x86_64]` |

### `memory`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{used}`, `{u}` | Used RAM, human units | `3.37 GiB` |
| `{total}`, `{t}` | Total RAM | `7.62 GiB` |
| `{free}`, `{f}` | Free RAM | `4.25 GiB` |
| `{available}`, `{a}` | Available RAM, same as free | `4.25 GiB` |
| `{pct}`, `{p}` | Percent used, no sign | `44` |
| `{bar}`, `{b}` | Progress bar | `[#########-----]` |

### `swap`

| Variable | Description |
| :--- | :--- |
| `{used}`, `{u}` | Used swap |
| `{total}`, `{t}` | Total swap |
| `{free}`, `{f}` | Free swap |
| `{pct}`, `{p}` | Percent used |
| `{bar}`, `{b}` | Progress bar |

### `disk`

| Variable | Description |
| :--- | :--- |
| `{mount}`, `{m}` | Mount point, padded to line up across disks |
| `{used}`, `{u}` | Used space |
| `{total}`, `{t}` | Total space |
| `{free}`, `{f}` | Free space |
| `{pct}`, `{p}` | Percent used |
| `{bar}`, `{b}` | Progress bar |

### `cpu`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{name}`, `{n}` | Full model name | `AMD Ryzen 7 7840HS` |
| `{cores}`, `{c}` | Core layout | `8c/16t` |
| `{freq}`, `{f}` | Current frequency | `3800 MHz` |
| `{mhz}` | Frequency without unit | `3800` |
| `{pcores}` | Physical cores | `8` |
| `{lcores}` | Logical cores | `16` |
| `{microarch}`, `{uarch}` | Microarchitecture | `Zen 4` |
| `{simd}` | SIMD extensions | `AVX-512, AVX2` |
| `{gflops}`, `{peak_gflops}` | Peak FP32 throughput | `486.4 GFLOP/s` |

### `cputemp`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{temp}`, `{t}` | Temperature with unit | `54°C` |
| `{celsius}`, `{c}` | Celsius as a number | `54` |
| `{fahrenheit}`, `{f}` | Fahrenheit as a number | `129` |
| `{color}` | Escape sequence: green (<60°C), yellow (<80°C), red (>=80°C) | |

### `cpuusage`

| Variable | Description |
| :--- | :--- |
| `{pct}`, `{p}` | Current utilisation percentage |
| `{bar}`, `{b}` | Progress bar |

### `battery`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{pct}`, `{p}` | Charge level percentage | `98` |
| `{status}` | Status with leading space | `(charging)` |
| `{s}` | Short status | `Charging` |
| `{charging}` | `true` while charging, empty otherwise | |
| `{discharging}` | `true` while discharging, empty otherwise | |
| `{wh}` | Watt-hours with brackets | `[45/50 Wh]` |
| `{energy_now}` | Raw watt-hours now | `45` |
| `{energy_full}` | Raw watt-hours full | `50` |
| `{raw_status}` | Lowercase status | `charging` |
| `{bar}`, `{b}` | Progress bar | |

### `title`

| Variable | Description |
| :--- | :--- |
| `{user}`, `{u}` | Current username |
| `{host}`, `{h}` | Hostname |

### `uptime`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{uptime}` | Human formatted | `2h 15m` |
| `{days}`, `{d}` | Days | `0` |
| `{hours}`, `{h}` | Hours | `2` |
| `{mins}`, `{m}` | Minutes | `15` |
| `{secs}`, `{s}` | Seconds | `42` |
| `{total_secs}` | Seconds in total | `8142` |

### `wifi`

| Variable | Description | Example |
| :--- | :--- | :--- |
| `{ssid}` | Network name | `MyNetwork` |
| `{signal}`, `{pct}`, `{p}` | Signal quality | `78` |
| `{dbm}` | Signal strength in dBm | `-52` |
| `{interface}`, `{i}` | Interface name | `wlan0` |

### `netio`

| Variable | Description |
| :--- | :--- |
| `{rx}` | Bytes received, human units |
| `{tx}` | Bytes sent, human units |
| `{total}`, `{t}` | Received plus sent |

### `cursor`

| Variable | Description |
| :--- | :--- |
| `{name}`, `{n}` | Cursor theme name |
| `{size}`, `{s}` | Cursor size in pixels |

### `media`

| Variable | Description |
| :--- | :--- |
| `{title}` | Track title from the active MPRIS player |
| `{artist}` | Track artist |

### `quote`

| Variable | Description |
| :--- | :--- |
| `{quote}`, `{q}` | The quote text |
| `{author}`, `{a}` | Who said it |

---

## 2. Functions

A function can be written three ways, and all three reach the same code with the same value:

- **call**: `{upper(name)}`, `{trunc(name, 20)}`, `{bar(15, '=', '-')}`
- **filter**: `{name \| upper}`, `{name \| remove('(R)') \| trunc(20)}`
- **colon**: `{used:gib}`, `{pct:round(1)}`, `{bar:15:=:-}`

Chains work too: `{name:remove('Intel(R) '):upper}`

Unit converters always get the raw number rather than the formatted text, so `{used:gib}` and `{gib(used)}` agree even though plain `{used}` prints `"3.37 GiB"`.

### Strings

| Function | Alias | Description |
| :--- | :--- | :--- |
| `upper(s)` / `s:upper` | `uppercase` | UPPERCASE |
| `lower(s)` / `s:lower` | `lowercase` | lowercase |
| `title(s)` / `s:title` | `capitalize` | Capitalise Each Word |
| `trunc(s, len)` / `s:trunc(len)` | `truncate` | At most `len` characters, ellipsis included (never splits UTF-8) |
| `replace(s, old, new)` | | Every occurrence of `old` becomes `new` |
| `remove(s, pattern)` | | Every occurrence of `pattern` is deleted |
| `trim(s)` | | Strips whitespace at both ends |
| `pad_left(s, width)` | | Pads on the left, text sits flush right |
| `pad_right(s, width)` | | Pads on the right, text sits flush left |

### Units and Numbers

| Function | Alias | Output Example |
| :--- | :--- | :--- |
| `gib(bytes)` / `s:gib` | `gb` | `7.62 GiB` |
| `mib(bytes)` / `s:mib` | `mb` | `7802.5 MiB` |
| `kib(bytes)` / `s:kib` | `kb` | `7990000 KiB` |
| `raw(bytes)` / `s:raw` | `bytes`, `b` | Bare integer, no unit |
| `human(bytes)` / `s:human` | | Auto-scaled: `B`, `KiB`, `MiB`, `GiB`, `TiB` |
| `round(number, decimals)` | | Rounded to specified decimals |

### Progress Bars

| Function | Description |
| :--- | :--- |
| `bar(width)` | Progress bar, default width and characters |
| `bar(width, fill, empty)` | Custom characters, quoted |
| `bar_color(width)` | Bar coloured by percentage: green (<60%), yellow (60-85%), red (>85%) |

### Fallbacks

| Function | Description | Example |
| :--- | :--- | :--- |
| `default(val, fallback)` | Fallback when value is empty | `{freq:default('N/A')}` |

### System & Date

| Function | Description |
| :--- | :--- |
| `env(NAME)` | Reads an environment variable |
| `date(format)` | `strftime`, current date and time |
| `time(format)` | Same as `date` |

### Colors and Attributes

- **Basic colors**: `{red}`, `{green}`, `{yellow}`, `{blue}`, `{magenta}`, `{cyan}`, `{white}`, `{black}`
- **Bright colors**: `{bright_red}`, `{bright_green}`, `{bright_yellow}`, `{bright_blue}`, `{bright_magenta}`, `{bright_cyan}`, `{bright_white}`
- **Grays**: `{gray}` and `{grey}` (both alias `{bright_black}`)
- **Formatting**: `{bold}`, `{reset}`

---

## 3. Arithmetic

### Byte Arithmetic (memory, swap, disk)

```text
{used + 2 GiB}     # Adds 2 GiB, result in human units
{used - 500 MiB}
{used * 2}
{used / 2}
{used + 2%}        # 2% of total, not of used
```

### Percentages and Numbers

```text
{pct + 5%}         # 45% becomes 50%
{pct - 2%}
{pct * 1.5}
```

Changing `{used}` moves `{bar}` and `{pct}` with it, since both read the adjusted metric rather than a recorded value.

### Units Accepted in Arithmetic

- Bytes: `b`, `byte`, `bytes`
- Kilobytes: `k`, `kb`, `kib`
- Megabytes: `m`, `mb`, `mib`
- Gigabytes: `g`, `gb`, `gib`
- Terabytes: `t`, `tb`, `tib`
- Percentage: `%`, `percent`

### Random Ranges

```text
{used + 2..4 GiB}  # Random value between 2 and 4 GiB, recalculated each run
{pct + 2..5%}
{2..4 GiB}         # Standalone random value
```

---

## 4. Configuration Example

```toml
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
```