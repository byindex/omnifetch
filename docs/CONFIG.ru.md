КОНФИГУРАЦИЯ - omnifetch
=======================

Переменные и функции шаблонизатора: FUNCTIONS.ru.md

ГДЕ ЛЕЖИТ ФАЙЛ
---------------

config.toml ищется в таком порядке, побеждает первый совпавший:

  1. путь из переменной окружения $OMNIFETCH_CONFIG
  2. $XDG_CONFIG_HOME/omnifetch/config.toml
  3. ~/.config/omnifetch/config.toml

Флаг -c или --config принимает путь напрямую и перекрывает всё перечисленное.

ИНТЕРАКТИВНЫЙ ГЕНЕРАТОР
-----------------------

Набор модулей, его порядок и параметры отображения собираются в TUI:

    omnifetch --gen-config                интерактивно, пишет в обычный путь
    omnifetch --gen-config /path/to.toml  пишет по этому пути
    omnifetch --gen-config -              печатает конфиг в stdout
    omnifetch --gen-config-force          перезаписать без вопроса

Клавиши в TUI:

  стрелки, или k j h     перемещение по модулям
  Space                  включить или выключить модуль, убрать break и separator
  /                      поиск модулей по имени, ID, категории или описанию (n/N: след./пред.)
  [ и ], или - и = (K/J) поднять или опустить модуль в порядке вывода
  c                      сгруппировать модули по категориям (System, Visual, Hardware...)
  v                      живой предпросмотр вывода omnifetch с текущими настройками
  r                      сбросить все модули и настройки к дефолтным (с подтверждением)
  b и B                  вставить пустую строку или разделитель
  d                      удалить выбранный разделитель или break
  f и F                  выделить все или инвертировать выбор
  l                      вид логотипа: default, small, none
  p                      позиция логотипа: auto, left, right, top
  o                      режим вывода: minimal или full
  s или Enter            сохранить
  q или Esc              выйти без сохранения

ПРИМЕР КОНФИГА
---------------

    logo = "auto"          # auto, mini, none или id дистрибутива
    fast = false           # минимальный набор модулей
    cache = true           # false отключает кэш в /dev/shm
    color = true

    modules = ["title", "separator", "os", "host", "kernel", "uptime",
               "packages", "shell", "cpu", "gpu", "memory", "disk"]

    [bar]
    width = 20
    fill = "█"
    empty = "░"

    [keys]                # свои названия вместо дефолтных
    memory = "RAM"
    disk = "Storage"

    [style]
    key_color = "cyan"     # имя цвета или hex, например #89b4fa
    title_color = "bright_black"
    bold_key = true

    [format]              # свой шаблон вывода для каждого модуля
    memory = "{used} / {total} {bar} {pct}%"
    disk = "{mount} {used} / {total} ({pct}%)"
    cpu = "{name | remove('Intel(R) ') | remove('Core(TM) ') | upper}"

    [network]             # адреса для сетевых модулей
    weather_ip = "5.9.243.187"          # пустая строка = только DNS
    weather_host = "wttr.in"
    publicip_host = "myip.opendns.com"
    publicip_fallback = "icanhazip.com"

ТЕКСТ ВНЕ СКОБОК
-----------------

Всё вне фигурных скобок печатается как написано. Внутри скобок работают
переменные модуля, функции-фильтры, арифметика и случайные диапазоны:

    [format]
    memory = "{used + 2..4 GiB} / {total} {bar} {pct}%"   # случайный прирост
    pct    = "{pct:round(1)}%"                            # 44.1%

СЕТЕВЫЕ НАСТРОЙКИ
-----------------

Секция network задает адреса для модулей, обращающихся к сети:

    [network]
    weather_ip = "5.9.243.187"          # пустая строка = только DNS
    weather_host = "wttr.in"
    publicip_host = "myip.opendns.com"
    publicip_fallback = "icanhazip.com"

МОДУЛИ CUSTOM И COMMAND
-----------------------

Для использования модулей `custom` и `command`:
1. Добавьте `"custom"` и/или `"command"` в список `modules`.
2. Задайте их содержимое в секции `[format]`:

    [format]
    custom = "Любой произвольный текст"
    command = "uname -r"

НАСТРОЙКИ РАМКИ
---------------

Вывод информации в блоках с рамками Unicode и заголовками категорий:

    border = true                 # краткое включение рамок

    [border]
    enabled = true
    title_align = "left"          # "left", "center" или "right"

В командной строке:

    omnifetch --border
    omnifetch --border-title center
    omnifetch --border-title right