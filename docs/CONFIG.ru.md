# Конфигурация - omnifetch

Переменные и функции шаблонизатора: [FUNCTIONS.ru.md](FUNCTIONS.ru.md)

## Где лежит файл

`config.toml` ищется в следующем порядке (побеждает первое совпадение):

1. Путь из переменной окружения `$OMNIFETCH_CONFIG`
2. `$XDG_CONFIG_HOME/omnifetch/config.toml`
3. `~/.config/omnifetch/config.toml`

Флаг `-c` или `--config` принимает путь напрямую и перекрывает всё перечисленное.

## Интерактивный генератор

Набор модулей, порядок их вывода и параметры оформления настраиваются в TUI:

```bash
omnifetch --gen-config                # интерактивно, сохраняет в ~/.config/omnifetch/config.toml
omnifetch --gen-config /path/to.toml  # сохраняет по указанному пути
omnifetch --gen-config -              # выводит сгенерированный конфиг в stdout
omnifetch --gen-config-force          # перезаписывает файл без подтверждения
```

### Клавиши в TUI

- `↑`, `↓`, `←`, `→` (или `k`, `j`, `h`, `l`) — Перемещение по модулям
- `Space` — Включить / выключить модуль (или удалить строку `break` / `separator`)
- `/` — Поиск модулей по имени, ID, категории или описанию (`n` / `N`: след. / пред.)
- `[` и `]`, `-` и `=`, или `K` / `J` — Переместить выбранный модуль выше или ниже
- `c` — Сгруппировать / отсортировать модули по категориям (System, Visual, Hardware...)
- `v` — Живой предпросмотр вывода `omnifetch` с текущими настройками
- `r` — Сбросить все модули и настройки к дефолтным (с подтверждением)
- `b` и `B` — Вставить пустую строку (`break`) или горизонтальный разделитель (`separator`)
- `d` — Удалить выбранный разделитель или пустую строку
- `f` и `F` — Выделить все модули или инвертировать выбор
- `l` — Вид логотипа: `default`, `small`, `none`
- `p` — Позиция логотипа: `auto`, `left`, `right`, `top`
- `o` — Режим вывода: `minimal` или `full`
- `s` или `Enter` — Сохранить конфигурацию
- `q` или `Esc` — Выйти без сохранения

## Пример конфигурации

```toml
logo = "auto"          # auto, mini, none или id дистрибутива
fast = false           # минимальный набор модулей
cache = true           # false отключает кэш в /dev/shm
color = true

modules = [
    "title", "separator", "os", "host", "kernel", "uptime",
    "packages", "shell", "cpu", "gpu", "memory", "disk"
]

[bar]
width = 20
fill = "█"
empty = "░"

[keys]                 # свои названия вместо дефолтных
memory = "RAM"
disk = "Storage"

[style]
key_color = "cyan"     # имя цвета или hex, например #89b4fa
title_color = "bright_black"
bold_key = true

[format]               # свой шаблон вывода для каждого модуля
memory = "{used} / {total} {bar} {pct}%"
disk = "{mount} {used} / {total} ({pct}%)"
cpu = "{name | remove('Intel(R) ') | remove('Core(TM) ') | upper}"

[network]              # адреса для сетевых модулей
weather_ip = "5.9.243.187"          # пустая строка = только DNS
weather_host = "wttr.in"
publicip_host = "myip.opendns.com"
publicip_fallback = "icanhazip.com"
```

## Текст вне скобок

Всё вне фигурных скобок печатается как написано. Внутри скобок работают переменные модуля, функции-фильтры, арифметика и случайные диапазоны:

```toml
[format]
memory = "{used + 2..4 GiB} / {total} {bar} {pct}%"   # случайный прирост
pct    = "{pct:round(1)}%"                            # 44.1%
```

## Переопределение значений модулей

Вы можете задать или переопределить то, что выводит любой модуль (например, для тестирования сетапов или кастомного текста):

- Напрямую на верхнем уровне файла конфигурации: `os = "Bubuntu x228_1337"`
- В секции `[format]`: `os = "Bubuntu x228_1337"` или `gpu = "2 × ASUS ROG Strix GeForce RTX 5090"`
- В секции `[values]`: `host = "ASUS Pro WS WRX90E-SAGE SE"`

Даже если модуль не определил устройство в вашей системе, указанное значение будет выведено.

## Модули custom и command

Для использования модулей `custom` и `command`:
1. Добавьте `"custom"` и/или `"command"` в список `modules`.
2. Задайте их содержимое в секции `[format]`:

```toml
[format]
custom = "Любой произвольный текст"
command = "uname -r"
```

## Настройки рамки

Вывод информации в блоках с рамками Unicode и заголовками категорий:

```toml
border = true                 # краткое включение рамок

[border]
enabled = true
title_align = "left"          # "left", "center" или "right"
```

В командной строке:

```bash
omnifetch --border
omnifetch --border-title center
omnifetch --border-title right
```