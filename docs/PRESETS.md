# Layout Presets - omnifetch

A preset is a fixed set of modules in a fixed order. Eight of them mirror a competitor's default output one to one, so speed can be compared on identical data. The Modules column counts the information modules a preset actually runs, not the layout helpers around them.

```bash
omnifetch --list-presets           # every preset with a description

omnifetch -p fastfetch             # exactly what fastfetch shows
omnifetch -p neofetch              # ...and neofetch
omnifetch -p sysprint
omnifetch -p catnap
omnifetch -p macchina
omnifetch -p paleofetch
omnifetch -p nitch
omnifetch -p pfetch

omnifetch -p detailed              # 34 modules
omnifetch -p all                   # 86 modules, the default set plus extras
```

## Available Presets

| Preset | Modules | What is in it |
| :--- | :---: | :--- |
| `minimal` | 5 | Just OS, kernel, uptime, memory |
| `pfetch` | 7 | Same as pfetch |
| `nitch` | 8 | Same as nitch |
| `paleofetch` | 12 | Same as paleofetch |
| `compact` | 14 | Dense, no blank lines |
| `hardware` | 15 | Hardware audit |
| `catnap` | 15 | Same as catnap |
| `macchina` | 16 | Same as macchina |
| `sysprint` | 17 | Same as sysprint, sectioned |
| `neofetch` | 18 | Same as Neofetch 7.x |
| `modern` | 20 | Temperatures, load, network |
| `fastfetch` | 23 | Same as fastfetch |
| `detailed` | 34 | Maximum information, no monitoring |
| `all` | 86 | Every local module (`--network` adds the two others) |