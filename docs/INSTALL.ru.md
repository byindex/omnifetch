# Установка и сборка - omnifetch

## 1. Cargo (crates.io)

Самый простой и быстрый способ установить omnifetch:

```bash
cargo install omnifetch-rs
```

Команда компилирует и устанавливает бинарник `omnifetch` в `~/.cargo/bin`. Убедитесь, что каталог `~/.cargo/bin` добавлен в переменную окружения `$PATH`.

## 2. Arch Linux (AUR)

Готовый бинарный пакет:

```bash
yay -S omnifetch-bin
# или: paru -S omnifetch-bin
```

Сборка из свежего исходного кода git master:

```bash
yay -S omnifetch-git
# или: paru -S omnifetch-git
```

## 3. Готовые бинарники (GitHub Releases)

Статически собранные архивы со скриптами автодополнений доступны на странице [GitHub Releases](https://github.com/byindex/omnifetch/releases).

### Автоматическая установка

```bash
tar -xzf omnifetch-v*-linux-x86_64.tar.gz
cd omnifetch-v*-linux-x86_64
sudo ./install.sh
```

Или установка в одну строку через curl:

```bash
curl -fsSL https://raw.githubusercontent.com/byindex/omnifetch/main/install.sh | bash
```

### Ручная установка

```bash
sudo install -Dm755 omnifetch /usr/local/bin/omnifetch
sudo ./setup-aliases.sh   # опциональный мастер настройки псевдонимов

# Опционально: установка автодополнений для shell
sudo install -Dm644 completions/omnifetch.bash /usr/share/bash-completion/completions/omnifetch
sudo install -Dm644 completions/_omnifetch /usr/share/zsh/site-functions/_omnifetch
sudo install -Dm644 completions/omnifetch.fish /usr/share/fish/vendor_completions.d/omnifetch.fish
```

## 4. Сборка из исходников

```bash
git clone https://github.com/byindex/omnifetch
cd omnifetch
make
sudo make install
```

`make install` устанавливает бинарник и автодополнения для `bash`, `zsh` и `fish`.  
`sudo make uninstall` корректно удаляет их из системы.

Нужны только `rustc` и `cargo` (с доступом к сети для скачивания единственной зависимости `libc`). Никаких системных библиотек доустанавливать не нужно, и никаких runtime-зависимостей нет, потому что бинарник собран статически.

## Статическая и динамическая сборка

По умолчанию сборка статическая: она не запускает динамический загрузчик при каждом запуске и экономит примерно 0.8 мс. При общем времени около 2 мс это заметная доля.

Для стандартной динамической сборки (например, для отладки или дистрибутивного пакетирования):

```bash
make build-dynamic
```

Две сборки попадают в разные каталоги:

- `make` — статическая, `target/<triple>/release/omnifetch`
- `make build-dynamic` — динамическая, `target/release/omnifetch`

## Что устанавливается

```text
<prefix>/bin/omnifetch
<prefix>/share/bash-completion/completions/omnifetch
<prefix>/share/zsh/site-functions/_omnifetch
<prefix>/share/fish/vendor_completions.d/omnifetch.fish
```

`PREFIX` по умолчанию `/usr/local`, его можно переопределить при установке:

```bash
make install PREFIX=/usr
```

`DESTDIR` учитывается для промежуточного пакетирования:

```bash
make install DESTDIR=/tmp/stage PREFIX=/usr
```