# Командная строка на практике - omnifetch

## Повседневные команды

```bash
omnifetch                          # дефолтная раскладка, логотип автоопределяется
omnifetch --fast                   # только базовые модули, максимум скорости
omnifetch --all                    # все локальные модули
omnifetch -m host,gpu,devenv       # выбор собственных модулей
omnifetch --theme nord             # применение готовой темы
omnifetch --gradient cyberpunk     # градиент по логотипу
omnifetch --logo-top --border      # логотип сверху, вывод в рамке
omnifetch --git                    # статистика по текущему git-репозиторию
omnifetch --json | jq .            # машинно-читаемый вывод
omnifetch --timing                 # отчет о времени выполнения каждого модуля
```

## Выбор пресета

```bash
omnifetch -p fastfetch             # те же модули, что у fastfetch
omnifetch -p neofetch              # те же модули, что у neofetch
omnifetch -p detailed              # 34 модуля
omnifetch -p all                   # 86 модулей
```

- Пресеты и число модулей: [PRESETS.ru.md](PRESETS.ru.md)
- Полная справка по флагам: [CLI.ru.md](CLI.ru.md)

## Скрипты

`--json` печатает плоский массив пар `name` и `data` вместо таблицы:

```json
[
  { "name": "OS",  "data": "Arch Linux rolling [x86_64]" },
  { "name": "CPU", "data": "Intel(R) Core(TM) i5-3360M ..." }
]
```

Извлечение конкретных полей через `jq`:

```bash
omnifetch --json --no-color | jq -r '.[] | select(.name=="OS") | .data'
omnifetch -m os,cpu --json --no-color | jq -r '.[].data'
```

`--no-color` и `NO_COLOR=1` убирают escape-последовательности. `--timing` пишет в `stderr`, поэтому не засоряет поток `--json`. Досрочное закрытие пайпа (например, через `head` или `jq`) корректно завершает программу без ошибок.

## Понять, почему что-то медленно

`--timing` печатает время выполнения каждого модуля в `stderr`, помогая выявить медленные модули:

```bash
omnifetch --no-cache --timing >/dev/null
```

Модуль, помеченный как `empty`, не вывел данных на текущей машине. Обычно это означает отсутствие соответствующего оборудования, а не ошибку в работе.