# Command Line in Practice - omnifetch

## Everyday Commands

```bash
omnifetch                          # default layout, logo auto-detected
omnifetch --fast                   # core modules only, maximum speed
omnifetch --all                    # every local module
omnifetch -m host,gpu,devenv       # pick your own modules
omnifetch --theme nord             # apply a built-in theme
omnifetch --gradient cyberpunk     # gradient across the logo
omnifetch --logo-top --border      # logo above, everything boxed
omnifetch --git                    # statistics for the current git repo
omnifetch --json | jq .            # machine readable output
omnifetch --timing                 # report execution time for each module
```

## Picking a Preset

```bash
omnifetch -p fastfetch             # same modules as fastfetch
omnifetch -p neofetch              # same modules as neofetch
omnifetch -p detailed              # 34 modules
omnifetch -p all                   # 86 modules
```

- Presets and module counts: [PRESETS.md](PRESETS.md)
- Complete CLI reference: [CLI.md](CLI.md)

## Scripting

`--json` prints a flat array of name and data pairs instead of a table:

```json
[
  { "name": "OS",  "data": "Arch Linux rolling [x86_64]" },
  { "name": "CPU", "data": "Intel(R) Core(TM) i5-3360M ..." }
]
```

Pulling specific fields with `jq`:

```bash
omnifetch --json --no-color | jq -r '.[] | select(.name=="OS") | .data'
omnifetch -m os,cpu --json --no-color | jq -r '.[].data'
```

`--no-color` and `NO_COLOR=1` both strip escape sequences. `--timing` writes to `stderr`, so it does not pollute a `--json` pipe. Closing the pipe early (e.g. `head` or `jq`) stops the program quietly without panicking.

## Finding Out Why Something is Slow

`--timing` prints one line per module to `stderr`, which helps pinpoint modules that take longer than expected:

```bash
omnifetch --no-cache --timing >/dev/null
```

A module listed as empty produced no output on this machine. That usually means the underlying hardware is not present rather than an error.