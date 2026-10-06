УСТАНОВКА И СБОРКА - omnifetch
==============================

1. CARGO (CRATES.IO)
--------------------

Самый простой и быстрый способ установить omnifetch:

    cargo install omnifetch-rs

Команда компилирует и устанавливает бинарник `omnifetch` в `~/.cargo/bin`. Убедитесь, что каталог `~/.cargo/bin` добавлен в переменную окружения `$PATH`.

2. ARCH LINUX (AUR)
-------------------

Готовый бинарный пакет:

    yay -S omnifetch-bin
    # или: paru -S omnifetch-bin

Сборка из свежего исходного кода git master:

    yay -S omnifetch-git
    # или: paru -S omnifetch-git

3. ГОТОВЫЕ БИНАРНИКИ (GITHUB RELEASES)
--------------------------------------

Статически собранные архивы со скриптами автодополнений доступны на странице релизов:
https://github.com/byindex/omnifetch/releases

Скачайте архив и установите в систему:

    tar -xzf omnifetch-v*-linux-x86_64.tar.gz
    cd omnifetch-v*-linux-x86_64
    sudo install -Dm755 omnifetch /usr/local/bin/omnifetch
    sudo ./setup-aliases.sh   # опциональный мастер настройки псевдонимов

4. СБОРКА ИЗ ИСХОДНИКОВ
-----------------------

    git clone https://github.com/byindex/omnifetch
    cd omnifetch
    make
    sudo make install

make install кладёт бинарник и автодополнения для bash, zsh и fish.
sudo make uninstall удаляет их. Больше ничего не устанавливается.

Нужны только rustc и cargo (с доступом к сети для скачивания единственной
зависимости libc). Никаких системных библиотек доустанавливать не нужно, и никаких
зависимостей во время работы, потому что бинарник собран статически.

СТАТИЧЕСКАЯ И ДИНАМИЧЕСКАЯ СБОРКА
---------------------------------

По умолчанию сборка статическая: она не запускает динамический загрузчик при
каждом запуске и экономит примерно 0.8 мс. При общем времени около 2 мс это
заметная доля.

Для стандартной динамической сборки (например, для отладки или пакетирования под дистрибутивы):

    make build-dynamic

Две сборки попадают в разные каталоги. Установщик берёт ту, что есть, так что
переключаться между ними не нужно:

    make                        статическая, target/<triple>/release/omnifetch
    make build-dynamic          динамическая, target/release/omnifetch

ЧТО УСТАНАВЛИВАЕТСЯ
-------------------

    <prefix>/bin/omnifetch
    <prefix>/share/bash-completion/completions/omnifetch
    <prefix>/share/zsh/site-functions/_omnifetch
    <prefix>/share/fish/vendor_completions.d/omnifetch.fish

PREFIX по умолчанию /usr/local, его можно задать при установке:

    make install PREFIX=/usr

DESTDIR тоже учитывается, для сборки в промежуточный каталог:

    make install DESTDIR=/tmp/stage PREFIX=/usr