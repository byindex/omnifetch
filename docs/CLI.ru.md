# Справка по командной строке - omnifetch

Сначала указана короткая форма, если она есть. Значения в угловых скобках обязательны, если не сказано иное.

## Общие

- `-h, --help` — Показать эту справку
- `-V, --version` — Показать версию

## Форма вывода

- `-l, --logo <NAME>` — Логотип: `auto` (по умолчанию), `none`, `mini` или ID дистрибутива
- `--logo-mini` — Использовать мини-логотип
- `--logo-top` — Выводить логотип сверху, а не сбоку
- `--border` — Обернуть вывод в Unicode-рамку
- `--border-title <ALIGN>` — Выравнивание названий категорий: `left`, `center`, `right`
- `--nerd` — Иконки Nerd Fonts перед названиями модулей
- `--nerd-only` — Показывать только иконки, без текстовых ключей
- `-j, --json` — Выводить сырой JSON вместо таблицы
- `--export <TARGET>` — Экспорт в SVG или HTML (`svg`, `html` или имя файла)
- `--no-color` — Без цветов (аналогично `NO_COLOR=1`)
- `-T, --timing` — Выводить замеры времени по модулям в stderr
- `--no-cache` — Полностью отключить кэширование

## Цвет и оформление

- `-t, --theme <NAME>` — `catppuccin`, `tokyo-night`, `nord`, `gruvbox`, `dracula`, `rose-pine`
- `--list-themes` — Показать встроенные темы и выйти
- `--gradient <NAME>` — `rainbow`, `sunset`, `cyberpunk`, `synthwave`, `fire`, `ice`, `matrix`, `dracula`
- `--completion <SH>` — Сгенерировать автодополнение: `bash`, `zsh`, `fish`

## Что запускать

- `-f, --fast` — Минимальный набор модулей, максимум скорости
- `-a, --all` — Все доступные модули (аналогично `-p all`)
- `-p, --preset <NAME>` — Пресет вёрстки (см. `--list-presets`)
- `--list-presets` — Показать пресеты с описаниями и выйти
- `-m, --modules <LIST>` — Список ID модулей через запятую или пробелы
- `--list-modules` — Напечатать все ID модулей и выйти
- `--network` — Включить два модуля, которым нужен интернет: `publicip` и `weather`
- `--git` — Статистика Git для текущего репозитория
- `--quotes-file <PATH>` — Пользовательский файл цитат (JSON или обычный текст)

## Изображения

- `-i, --image <PATH>` — PNG или Sixel через Kitty Graphics Protocol
- `--image-cols <NUM>` — Ширина в знакоместах (по умолчанию 34)
- `--image-rows <NUM>` — Высота в строках (по умолчанию 17)

## Конфигурация

- `-c, --config <PATH>` — Загрузить модули и оформление из файла TOML
- `--gen-config [PATH]` — Интерактивный TUI-генератор. Без пути пишет в `~/.config/omnifetch/config.toml`, с путём — туда, `"-"` печатает в stdout
- `--gen-config-force` — Перезаписать существующий файл без подтверждения

## Примеры

```bash
omnifetch                          # дефолтная раскладка, логотип автоопределяется
omnifetch --logo mini --border     # мини-логотип в рамке
omnifetch --theme tokyo-night      # применить цветовую тему
omnifetch --gradient cyberpunk     # градиент по логотипу
omnifetch --nerd                   # иконки Nerd Fonts
omnifetch -p modern                # пресет
omnifetch -p detailed              # 34 модуля
omnifetch -p fastfetch             # те же модули, что у fastfetch
omnifetch -p neofetch              # те же модули, что у neofetch
omnifetch --network                # добавить Public IP и погоду
omnifetch --all --no-cache         # все модули, без кэша
omnifetch --fast                   # максимально быстрый режим
```

Панели делятся на категории: `System`, `Visual`, `Hardware`, `Network`, `Devices` и `Quote`.