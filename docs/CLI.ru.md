СПРАВКА ПО КОММАНДНОЙ СТРОКЕ - omnifetch
========================================

Сначала короткая форма, если она есть. Значения в угловых скобках
обязательны, если не сказано иное.


ОБЩИЕ
------

  -h, --help                   Показать эту справку
  -V, --version                Показать версию


ФОРМ ВЫВОДА
------------

  -l, --logo <NAME>            Логотип: auto (по умолчанию), none, mini
                               или id дистрибутива
      --logo-mini              Мини-логотип
      --logo-top               Логотип сверху, а не сбоку
      --border                 Обернуть вывод в Unicode-рамку
      --border-title <ALIGN>   Выравнивание названий категорий: left, center, right
      --nerd                   Иконки Nerd Fonts перед названиями модулей
      --nerd-only              Только иконки, без текстовых ключей
  -j, --json                   Сырой JSON вместо таблицы
      --export <TARGET>        Экспорт в SVG или HTML ("svg", "html", или имя файла)
      --no-color               Без цветов, то же что NO_COLOR=1
  -T, --timing                 Тайминги по модулям в stderr
      --no-cache               Полностью отключить кэш


ЦВЕТ И ОФОРМЛЕНИЕ
-----------------

  -t, --theme <NAME>           catppuccin, tokyo-night, nord, gruvbox,
                               dracula, rose-pine
      --list-themes            Показать встроенные темы и выйти
      --gradient <NAME>        rainbow, sunset, cyberpunk, synthwave, fire,
                               ice, matrix, dracula
      --completion <SH>        Автодополнение: bash, zsh, fish


ЧТО ЗАПУСКАТЬ
--------------

  -f, --fast                   Минимальный набор модулей, максимум скорости
  -a, --all                    Все доступные модули, то же что -p all
  -p, --preset <NAME>          Пресет вёрстки, см. --list-presets
      --list-presets           Показать пресеты с описаниями и выйти
  -m, --modules <LIST>         Список id модулей через запятую или пробелы
      --list-modules           Напечатать все id модулей и выйти
      --network                Включить два модуля, которым нужен интернет:
                               publicip и weather
      --git                    Статистика git для текущего репозитория
      --quotes-file <PATH>     Свой файл цитат, JSON или обычный текст


КАРТИНКИ
--------

  -i, --image <PATH>           PNG или Sixel через Kitty Graphics
                               Protocol
      --image-cols <NUM>       Ширина в знакоместах, по умолчанию 34
      --image-rows <NUM>       Высота в строках, по умолчанию 17


КОНФИГУРАЦИЯ
-------------

  -c, --config <PATH>          Взять модули и логотип из TOML-файла
      --gen-config <?PATH>     Интерактивный TUI-генератор. Без пути пишет в
                               ~/.config/omnifetch/config.toml, с путём
                               пишет туда, "-" печатает в stdout
      --gen-config-force       Перезаписать существующий файл без вопроса


ПРИМЕРЫ
-------

  omnifetch                          дефолтная раскладка, логотип сам
  omnifetch --logo mini --border     мини-логотип в рамке
  omnifetch --theme tokyo-night      применить тему
  omnifetch --gradient cyberpunk     градиент по логотипу
  omnifetch --nerd                   иконки Nerd Fonts
  omnifetch -p modern                пресет
  omnifetch -p detailed              34 модуля
  omnifetch -p fastfetch             те же модули, что у fastfetch
  omnifetch -p neofetch              те же модули, что у neofetch
  omnifetch --network                добавить Public IP и погоду
  omnifetch --all --no-cache         все модули, без кэша
  omnifetch --fast                   самый быстрый режим

Панели делятся на System, Visual, Hardware, Network, Devices и Quote.