ВОЗМОЖНОСТИ - omnifetch
=======================

МОДУЛИ
------

В реестре 88 модулей. 86 выполняются при --all, остальные два спрятаны за
--network. От базовой информации о системе до отдельных кусков железа и
софта вокруг него.

  Система        os, kernel, bootmgr, initsystem, security, host, board,
                 bios, tpm, chassis, uptime, loadavg, processes, packages,
                 datetime, locale, users, version

  Оболочка       shell, terminal, terminalfont, terminalsize, terminaltheme,
                 editor, wm, wmtheme, de, theme, icons, font, cursor,
                 wallpaper

  Железо         cpu, cpucache, cputemp, cpuusage, powerprofile, poweradapter,
                 gpu, gpudriver, vulkan, opengl, opencl, memory, swap, disk,
                 physicaldisk, physicalmemory, diskio, brightness, battery,
                 display, displayserver, resolution

  Периферия      keyboard, mouse, touchpad, camera, gamepad, media, player,
                 bluetooth, bluetoothradio, monitor, audioserver, sound

  Сеть           netadapter, localip, dns, wifi, netio, publicip, weather

  Детали системы btrfs, zpool, codec, lm, containers, command, custom, top

  Содержимое     quote, git, devenv

  Разметка       title, separator, break, colors

Актуальный список со значениями: omnifetch --list-modules

ФОРМ ВЫВОДА
------------

  --logo, --logo-mini, --logo-top       выбрать или переместить логотип
  --border                             скруглённая рамка по категориям
  --nerd, --nerd-only                  глифы Nerd Fonts вместо текстовых ключей
  --theme, --gradient                  6 тем, 8 Truecolor-градиентов
  --image, --image-cols, --image-rows  PNG и Sixel через Kitty
                                      Graphics Protocol
  --json                               сырой JSON для скриптов
  --timing                             стоимость каждого модуля, в stderr

ВЁРСТКА
--------

  596 ASCII-логотипов дистрибутивов и языков программирования, обычных и мини,
  с бинарным поиском вместо линейного сканирования.
  14 пресетов, восемь из них повторяют дефолтный вывод конкурента: PRESETS.ru.md

КОНФИГУРАЦИЯ
------------

  Один файл задаёт всё: какие модули и в каком порядке, под какими названиями,
  какими цветами, с какой шириной полосок и по каким шаблонам.
  --gen-config собирает его через интерактивный TUI.
  В языке шаблонов есть фильтры, конвертеры единиц, арифметика и случайные
  диапазоны: FUNCTIONS.ru.md

СКРИПТЫ И ДИАГНОСТИКА
---------------------

  --json             плоский массив пар name и data
  --timing           тайминги по модулям, в stderr
  --no-cache         полностью обойти кэш
  --network          включить два модуля, которым нужен интернет
  --completion       bash, zsh, fish