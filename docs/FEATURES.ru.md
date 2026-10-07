# Возможности - omnifetch

## Модули

В реестре 88 модулей (86 выполняются при `--all`, а `publicip` и `weather` требуют `--network`). От базовой информации о системе до отдельных аппаратных компонентов:

- **Система**: `os`, `kernel`, `bootmgr`, `initsystem`, `security`, `host`, `board`, `bios`, `tpm`, `chassis`, `uptime`, `loadavg`, `processes`, `packages`, `datetime`, `locale`, `users`, `version`
- **Оболочка**: `shell`, `terminal`, `terminalfont`, `terminalsize`, `terminaltheme`, `editor`, `wm`, `wmtheme`, `de`, `theme`, `icons`, `font`, `cursor`, `wallpaper`
- **Железо**: `cpu`, `cpucache`, `cputemp`, `cpuusage`, `powerprofile`, `poweradapter`, `gpu`, `gpudriver`, `vulkan`, `opengl`, `opencl`, `memory`, `swap`, `disk`, `physicaldisk`, `physicalmemory`, `diskio`, `brightness`, `battery`, `display`, `displayserver`, `resolution`
- **Периферия**: `keyboard`, `mouse`, `touchpad`, `camera`, `gamepad`, `media`, `player`, `bluetooth`, `bluetoothradio`, `monitor`, `audioserver`, `sound`
- **Сеть**: `netadapter`, `localip`, `dns`, `wifi`, `netio`, `publicip`, `weather`
- **Детали системы**: `btrfs`, `zpool`, `codec`, `lm`, `containers`, `command`, `custom`, `top`
- **Содержимое**: `quote`, `git`, `devenv`
- **Разметка**: `title`, `separator`, `break`, `colors`

Актуальный список модулей: `omnifetch --list-modules`

## Форма вывода

- `--logo`, `--logo-mini`, `--logo-top` — Выбрать или переместить логотип
- `--border`, `--border-title` — Скруглённая рамка по категориям с настраиваемым выравниванием
- `--nerd`, `--nerd-only` — Глифы Nerd Fonts вместо текстовых ключей
- `--theme`, `--gradient` — 6 тем, 8 Truecolor-градиентов
- `--image`, `--image-cols`, `--image-rows` — PNG и Sixel через Kitty Graphics Protocol
- `--json` — Сырой JSON для скриптов
- `--timing` — Замеры времени каждого модуля в stderr

## Вёрстка

- 596 ASCII-логотипов дистрибутивов и языков программирования, полноразмерных и мини, с быстрым бинарным поиском.
- 14 пресетов, восемь из которых повторяют вывод популярных аналогов: [PRESETS.ru.md](PRESETS.ru.md)

## Конфигурация

- Один файл задаёт всё: какие модули и в каком порядке, под какими названиями, цветами, с какой шириной полосок и шаблонами.
- `--gen-config` собирает его через интерактивный TUI.
- Язык шаблонов поддерживает фильтры, конвертеры единиц, арифметику и случайные диапазоны: [FUNCTIONS.ru.md](FUNCTIONS.ru.md)

## Скрипты и диагностика

- `--json` — Плоский массив пар `name` и `data`
- `--timing` — Тайминги по модулям в stderr
- `--no-cache` — Полностью обойти кэш
- `--network` — Включить интернет-модули
- `--completion` — Автодополнение для `bash`, `zsh`, `fish`