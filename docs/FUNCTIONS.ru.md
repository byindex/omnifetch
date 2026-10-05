ФУНКЦИИ И ПЕРЕМЕННЫЕ ШАБЛОНИЗАТОРА OMNIFETCH
============================================

Шаблоны вывода живут в ~/.config/omnifetch/config.toml, в секции [format].
Любой текст вне {скобок} печатается дословно:

    memory = "{f} свободно из 1337GB"
    -> 4.24 GiB свободно из 1337GB

Модуль с единственным значением, например os или host, печатает это значение
как есть и секцию [format] игнорирует.


1. ПЕРЕМЕННЫЕ МОДУЛЕЙ
---------------------

memory
    {used} {u}           Занято ОЗУ, человеческие единицы   3.37 GiB
    {total} {t}          Всего ОЗУ                          7.62 GiB
    {free} {f}           Свободно
    {available} {a}      Доступно, то же что free
    {pct} {p}            Процент занято, без знака           44
    {bar} {b}            Полоска                             [#########-----]

swap
    {used} {u}           Занято подкачкой
    {total} {t}          Всего подкачки
    {free} {f}           Свободно подкачки
    {pct} {p}            Процент занято
    {bar} {b}            Полоска

disk
    {mount} {m}          Точка монтирования, выровнена по ширине
    {used} {u}           Занято
    {total} {t}          Всего
    {free} {f}           Свободно
    {pct} {p}            Процент занято
    {bar} {b}            Полоска

cpu
    {name} {n}           Полное название модели
    {cores} {c}          Число ядер                            2c/4t
    {freq} {f}           Текущая частота                       2592 MHz
    {mhz}                Частота без единицы измерения         2592
    {pcores}             Физических ядер                       2
    {lcores}             Логических ядер                       4
    {microarch} {uarch}  Микроархитектура                      Ivy Bridge
    {simd}               SIMD-расширения                       SSE4.2, AVX, AVX2
    {gflops}             Пик FP32                              83.2 GFLOP/s
    {peak_gflops}        То же самое

cputemp
    {temp} {t}           Температура с единицей измерения     54C
    {celsius} {c}        Цельсий числом                        54
    {fahrenheit} {f}     Фаренгейт числом                      129
    {color}              Escape-последовательность: зелёный до 60C,
                         жёлтый до 80C, выше красный

cpuusage
    {pct} {p}            Текущая загрузка
    {bar} {b}            Полоска

battery
    {pct} {p}            Уровень заряда                        98
    {status}             Статус с ведущим пробелом            (charging)
    {s}                 Короткий статус                       Charging
    {charging}           true при зарядке, иначе пусто
    {discharging}        true при разрядке, иначе пусто
    {wh}                 Ватт-часы в скобках                  [45/50 Wh]
    {energy_now}         Сырые ватт-часы сейчас               45
    {energy_full}        Сырые ватт-часы при полном заряде     50
    {raw_status}         Статус в нижнем регистре              charging
    {bar} {b}            Полоска

title
    {user} {u}           Имя пользователя
    {host} {h}           Имя узла

uptime
    {uptime}             По-человечески                        2h 15m
    {days} {d}           Дни
    {hours} {h}          Часы
    {mins} {m}           Минуты
    {secs} {s}           Секунды
    {total_secs}         Секунд всего                          8100

wifi
    {ssid}               Имя сети                              MyNetwork
    {signal} {pct} {p}   Качество сигнала                      78
    {dbm}                Мощность сигнала в dBm                -52
    {interface} {i}      Имя интерфейса                        wlan0

netio
    {rx}                 Принято, человеческие единицы
    {tx}                 Отправлено, человеческие единицы
    {total} {t}          Принято плюс отправлено

cursor
    {name} {n}           Название темы курсора
    {size} {s}           Размер курсора в пикселях

media
    {title}              Название трека из активного MPRIS-плеера
    {artist}             Исполнитель

quote
    {quote} {q}          Текст цитаты
    {author} {a}         Автор


2. ФУНКЦИИ
----------

Функцию можно записать тремя способами, и все три ведут в один и тот же код
с одним и тем же значением:

    вызов    {upper(name)}        {trunc(name, 20)}     {bar(15, '=', '-')}
    фильтр   {name | upper}       {name | remove('(R)') | trunc(20)}
    двоеточие {used:gib}          {pct:round(1)}        {bar:15:=:-}

Цепочки тоже работают: {name:remove('Intel(R) '):upper}

Конвертеры единиц всегда получают исходное число, а не отформатированный
текст, поэтому {used:gib} и {gib(used)} сходятся, хотя plain {used} печатает
"3.37 GiB".

Строки
    upper(s) / s:upper            ВЕРХНИЙ РЕГИСТР
    lower(s) / s:lower            нижний регистр
    title(s) / s:title            Заглавные Буквы Каждого Слова

    upper, uppercase              псевдоним upper
    lower, lowercase              псевдоним lower
    title, capitalize             псевдоним title

    trunc(s, len) / s:trunc(len)  Не больше len символов, многоточие входит
                                  в длину. Никогда не режет многобайтовый символ.
    truncate                      псевдоним trunc

    replace(s, old, new)          Каждое вхождение old заменяется на new
    remove(s, pattern)            Каждое вхождение pattern удаляется
    trim(s)                       Убирает пробелы с обоих концов
    pad_left(s, width)            Дополняет слева, текст прижат вправо
    pad_right(s, width)           Дополняет справа, текст прижат влево

Единицы и числа
    gib(bytes) / s:gib            7.62 GiB
    mib(bytes) / s:mib            7802.5 MiB
    kib(bytes) / s:kib            7990000 KiB
    raw(bytes) / s:raw            Голое целое, без единиц
    human(bytes) / s:human        Масштабируется сам: B, KiB, MiB, GiB, TiB
    round(number, decimals)       Округление до заданного числа знаков

    gb, mb, kb                    псевдонимы gib, mib, kib
    bytes, b                      псевдоним raw

Полоски
    bar(width)                    Полоска, ширина и символы по умолчанию
    bar(width, fill, empty)       Свои символы, в кавычках

    bar_color(width)              Полоска с цветом по проценту:
                                    зелёный до 60
                                    жёлтый с 60 до 85
                                    красный выше 85

Запасные значения
    default(value, fallback)      fallback, если value пусто
                                  {freq:default('N/A')}

Системные
    env(NAME)                     Читает переменную окружения
    date(format)                  strftime, текущие дата и время
    time(format)                  То же, что date

Цвета и атрибуты
    {red} {green} {yellow} {blue} {magenta} {cyan} {white} {black}
    {bright_red} {bright_green} {bright_yellow} {bright_blue}
    {bright_magenta} {bright_cyan} {bright_white}
    {gray} и {grey} означают {bright_black}
    {bold} {reset}


3. АРИФМЕТИКА
-------------

Байтовая арифметика для memory, swap и disk
    {used + 2 GiB}                Плюс 2 GiB, результат в человеческих единицах
    {used - 500 MiB}
    {used * 2}
    {used / 2}
    {used + 2%}                   2 процента от total, а не от used

Проценты и обычные числа
    {pct + 5%}                    45% превращается в 50%
    {pct - 2%}
    {pct * 1.5}

Изменение {used} двигает {bar} и {pct} вместе с собой, потому что оба
читают скорректированную метрику, а не записанное значение.

Единицы, допустимые в арифметике
    b    byte    bytes
    k    kb     kib
    m    mb     mib
    g    gb     gib
    t    tb     tib
    %    процент

Случайные диапазоны
    {used + 2..4 GiB}             Значение между 2 и 4 GiB, разное при каждом
                                  запуске. Полоска и процент следуют за ним.
    {pct + 2..5%}
    {2..4 GiB}                    Отдельное случайное значение


4. ПРИМЕР КОНФИГУРАЦИИ
----------------------

    # ~/.config/omnifetch/config.toml

    [bar]
    width = 16
    fill = "■"
    empty = " "

    [keys]
    memory = "RAM"
    swap = "SWAP"
    disk = "Storage"
    cpu = "Processor"

    [format]
    memory = "{used + 2..4 GiB} / {total} {bar} {pct}%"
    swap = "{used * 2} / {total} {bar} {pct}%"
    disk = "{mount} {used} / {total} {bar} ({pct}%)"
    cpu = "{name | remove('Intel(R) ') | remove('Core(TM) ') | upper} ({cores}) @ {freq}"
    battery = "{bar(10)} {pct}%{status}"
    title = "{bold}{cyan}{user}{reset}@{bold}{host}{reset}"