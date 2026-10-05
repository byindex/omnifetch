КОМАНДНАЯ СТРОКА НА ПРАКТИКЕ - omnifetch
==========================================

ПОВСЕДНЕВНЫЕ КОМАНДЫ
---------------------

    omnifetch                          дефолтная раскладка, логотип сам
    omnifetch --fast                   только базовые модули, максимум скорости
    omnifetch --all                    все локальные модули
    omnifetch -m host,gpu,devenv       свои модули
    omnifetch --theme nord             готовая тема
    omnifetch --gradient cyberpunk     градиент по логотипу
    omnifetch --logo-top --border      логотип сверху, всё в рамке
    omnifetch --git                    статистика по текущему git-репозиторию
    omnifetch --json | jq .            машинно-читаемый вывод
    omnifetch --timing                 сколько стоил каждый модуль

ВЫБОР ПРЕСЕТА
--------------

    omnifetch -p fastfetch             те же модули, что у fastfetch
    omnifetch -p neofetch              те же модули, что у neofetch
    omnifetch -p detailed              34 модуля
    omnifetch -p all                   86 модулей

ПРЕСЕТЫ И ЧИСЛО МОДУЛЕЙ: PRESETS.ru.md
ВСЕ ФЛАГИ: CLI.ru.md

СКРИПТЫ
-------

--json печатает плоский массив пар name и data вместо таблицы:

    [
      { "name": "OS",  "data": "Arch Linux rolling [x86_64]" },
      { "name": "CPU", "data": "Intel(R) Core(TM) i5-3360M ..." }
    ]

Чтобы достать одно поле:

    omnifetch --json --no-color | jq -r '.[] | select(.name=="OS") | .data'
    omnifetch -m os,cpu --json --no-color | jq -r '.[].data'

--no-color и NO_COLOR=1 убирают escape-последовательности. --timing пишет в
stderr, поэтому не портит канал с --json. Если закрыть канал заранее, как это
делают head или jq, программа тихо остановится, а не упадёт.

ПОНЯТЬ, ПОЧЕМУ ЧТО-ТО МЕДЛЕННО
--------------------------------

--timing печатает по строке на модуль в stderr. Это первое место, куда стоит
смотреть, если модуль работает дольше ожидаемого:

    omnifetch --no-cache --timing >/dev/null

Модуль, помеченный как empty, ничего не вывел на этой машине. Чаще всего это
значит, что оборудования просто нет, а не что модуль сломан.