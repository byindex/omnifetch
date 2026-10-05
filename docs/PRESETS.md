LAYOUT PRESETS - omnifetch
==========================

A preset is a fixed set of modules in a fixed order. Eight of them mirror a
competitor's default output one to one, so speed can be compared on identical
data. The Modules column counts the information modules a preset actually runs,
not the layout helpers around them.

  omnifetch --list-presets           every preset with a description

  omnifetch -p fastfetch             exactly what fastfetch shows
  omnifetch -p neofetch              ...and neofetch
  omnifetch -p sysprint
  omnifetch -p catnap
  omnifetch -p macchina
  omnifetch -p paleofetch
  omnifetch -p nitch
  omnifetch -p pfetch

  omnifetch -p detailed              34 modules
  omnifetch -p all                   86 modules, the default set plus extras


Preset       Modules  What is in it
---------   -------  -----------------------------------------------
minimal           5  just OS, kernel, uptime, memory
pfetch            7  same as pfetch
nitch             8  same as nitch
paleofetch       12  same as paleofetch
compact          14  dense, no blank lines
hardware         15  hardware audit
catnap           15  same as catnap
macchina         16  same as macchina
sysprint         17  same as sysprint, sectioned
neofetch         18  same as Neofetch 7.x
modern           20  temperatures, load, network
fastfetch        23  same as fastfetch
detailed         34  maximum information, no monitoring
all              86  every local module, --network adds the two others