#!/usr/bin/env bash

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[0;90m'
NC='\033[0m'

MARKER_START="# >>> omnifetch aliases >>>"
MARKER_END="# <<< omnifetch aliases <<<"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo ".")"

find_local_binary() {
    local candidates=(
        "$SCRIPT_DIR/target/x86_64-unknown-linux-gnu/release/omnifetch"
        "$SCRIPT_DIR/target/release/omnifetch"
        "./target/x86_64-unknown-linux-gnu/release/omnifetch"
        "./target/release/omnifetch"
        "$HOME/.local/bin/omnifetch"
        "$HOME/bin/omnifetch"
        "/usr/local/bin/omnifetch"
    )
    for c in "${candidates[@]}"; do
        if [ -f "$c" ]; then
            realpath "$c" 2>/dev/null || echo "$c"
            return 0
        fi
    done
    return 1
}

banner() {
    clear 2>/dev/null || true
    echo -e "${CYAN}${BOLD}"
    echo "  ╔═══════════════════════════════════════════════════════════╗"
    echo "  ║        ⚡ OMNIFETCH — Мастер управления алиасами          ║"
    echo "  ╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    if command -v omnifetch >/dev/null 2>&1; then
        local bin_path
        bin_path=$(command -v omnifetch)
        echo -e "  Статус: ${GREEN}✔ omnifetch найден в PATH${NC} ${GRAY}($bin_path)${NC}"
    else
        local local_bin
        if local_bin=$(find_local_binary); then
            echo -e "  Статус: ${YELLOW}⚠ omnifetch не найден в PATH${NC} ${GRAY}(найден локальный: $local_bin)${NC}"
        else
            echo -e "  Статус: ${RED}⚠ omnifetch не найден в PATH и не установлен${NC}"
        fi
    fi
    echo ""
}

detect_default_shell() {
    local sh_name
    sh_name=$(basename "${SHELL:-bash}")
    case "$sh_name" in
        zsh)  echo "zsh" ;;
        fish) echo "fish" ;;
        bash) echo "bash" ;;
        *)    echo "bash" ;;
    esac
}

get_config_path_for_shell() {
    local sh="$1"
    case "$sh" in
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                echo "$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                echo "$HOME/.bash_profile"
            else
                echo "$HOME/.bashrc"
            fi
            ;;
        zsh)
            echo "${ZDOTDIR:-$HOME}/.zshrc"
            ;;
        fish)
            local fish_dir="${XDG_CONFIG_HOME:-$HOME/.config}/fish"
            echo "$fish_dir/config.fish"
            ;;
        *)
            echo "$HOME/.profile"
            ;;
    esac
}

select_shell_and_config() {
    local def_sh
    def_sh=$(detect_default_shell)

    echo -e "${BOLD}1. Выберите ваш командный шелл:${NC}"
    echo -e "   ${GREEN}1)${NC} Bash       ${GRAY}(автоопределен: $def_sh)${NC}"
    echo -e "   ${GREEN}2)${NC} Zsh"
    echo -e "   ${GREEN}3)${NC} Fish"
    echo -e "   ${GREEN}4)${NC} Другой / указать путь к конфигу вручную"
    echo ""
    read -r -p "Введите номер [по умолчанию $def_sh]: " sh_choice

    local chosen_sh="$def_sh"
    case "$sh_choice" in
        1) chosen_sh="bash" ;;
        2) chosen_sh="zsh" ;;
        3) chosen_sh="fish" ;;
        4) chosen_sh="custom" ;;
        *)
            if [ -n "$sh_choice" ]; then
                echo -e "${YELLOW}Неизвестный ввод, используется $def_sh.${NC}"
            fi
            ;;
    esac

    local config_file=""
    if [ "$chosen_sh" = "custom" ]; then
        echo ""
        read -r -p "Введите абсолютный или домашний путь к файлу конфигурации: " custom_path
        config_file="${custom_path/#\~/$HOME}"
    else
        config_file=$(get_config_path_for_shell "$chosen_sh")
    fi

    echo ""
    echo -e "Используемый файл: ${BOLD}${CYAN}$config_file${NC}"
    read -r -p "Использовать этот файл? [Y/n]: " confirm_file
    if [[ "$confirm_file" =~ ^[Nn] ]]; then
        read -r -p "Введите желаемый путь к конфигурационному файлу: " custom_path
        config_file="${custom_path/#\~/$HOME}"
    fi

    SELECTED_SHELL="$chosen_sh"
    SELECTED_CONFIG="$config_file"
}

resolve_omnifetch_executable() {
    local default_target="omnifetch"

    if command -v omnifetch >/dev/null 2>&1; then
        RESOLVED_CMD="omnifetch"
        return 0
    fi

    # omnifetch не найден в PATH — выводим предупреждение и варианты решения
    echo -e "${YELLOW}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║  ⚠ ПРЕДУПРЕЖДЕНИЕ: команда 'omnifetch' не найдена в PATH!     ║${NC}"
    echo -e "${YELLOW}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo -e "${GRAY}Если создать алиас сейчас со стандартной командой 'omnifetch',${NC}"
    echo -e "${GRAY}он выдаст ошибку 'command not found' при запуске в шелле.${NC}"
    echo ""

    local local_bin=""
    local_bin=$(find_local_binary || true)

    if [ -n "$local_bin" ]; then
        echo -e "Обнаружен скомпилированный бинарник:"
        echo -e "  ${CYAN}${BOLD}$local_bin${NC}"
        echo ""
        echo -e "${BOLD}Как вы хотите привязать команду Omnifetch?${NC}"
        echo -e "   ${GREEN}1)${NC} Использовать найденный бинарник ${GRAY}($local_bin)${NC}"
        echo -e "   ${GREEN}2)${NC} Указать путь к бинарнику вручную"
        echo -e "   ${GREEN}3)${NC} Собрать и установить в систему (${BOLD}make build && sudo make install${NC})"
        echo -e "   ${GREEN}4)${NC} Оставить имя 'omnifetch' ${GRAY}(установлю позже в PATH)${NC}"
        echo ""
        read -r -p "Выберите вариант [1-4, по умолчанию 1]: " bin_choice

        case "$bin_choice" in
            2)
                prompt_custom_binary_path
                ;;
            3)
                install_omnifetch_system
                ;;
            4)
                RESOLVED_CMD="omnifetch"
                ;;
            1|*)
                RESOLVED_CMD="$local_bin"
                ;;
        esac
    else
        echo -e "${BOLD}Как вы хотите привязать команду Omnifetch?${NC}"
        echo -e "   ${GREEN}1)${NC} Указать путь к исполняемому файлу бинарника вручную"
        echo -e "   ${GREEN}2)${NC} Собрать и установить в систему (${BOLD}make build && sudo make install${NC})"
        echo -e "   ${GREEN}3)${NC} Оставить имя 'omnifetch' ${GRAY}(установлю позже в PATH)${NC}"
        echo ""
        read -r -p "Выберите вариант [1-3, по умолчанию 1]: " bin_choice

        case "$bin_choice" in
            2)
                install_omnifetch_system
                ;;
            3)
                RESOLVED_CMD="omnifetch"
                ;;
            1|*)
                prompt_custom_binary_path
                ;;
        esac
    fi
}

prompt_custom_binary_path() {
    echo ""
    read -r -p "Введите абсолютный или относительный путь к бинарнику omnifetch: " user_bin
    user_bin="${user_bin/#\~/$HOME}"
    user_bin=$(echo "$user_bin" | xargs)

    if [ -z "$user_bin" ]; then
        echo -e "${YELLOW}Путь не указан. Используется стандартное имя 'omnifetch'.${NC}"
        RESOLVED_CMD="omnifetch"
        return 0
    fi

    local abs_bin
    abs_bin=$(realpath -m "$user_bin" 2>/dev/null || echo "$user_bin")

    if [ ! -f "$abs_bin" ]; then
        echo -e "${RED}Предупреждение: файл '$abs_bin' не существует на диске.${NC}"
        read -r -p "Всё равно использовать этот путь? [y/N]: " confirm_nf
        if [[ ! "$confirm_nf" =~ ^[YyДд] ]]; then
            prompt_custom_binary_path
            return 0
        fi
    elif [ ! -x "$abs_bin" ]; then
        echo -e "${YELLOW}Файл '$abs_bin' не исполняемый. Добавляем права на исполнение (chmod +x)...${NC}"
        chmod +x "$abs_bin" 2>/dev/null || true
    fi

    RESOLVED_CMD="$abs_bin"
}

install_omnifetch_system() {
    echo ""
    echo -e "${CYAN}${BOLD}==> Сборка и установка omnifetch...${NC}"
    if [ -f "$SCRIPT_DIR/Makefile" ]; then
        if (cd "$SCRIPT_DIR" && make build && sudo make install); then
            echo ""
            echo -e "${GREEN}${BOLD}✔ omnifetch успешно установлен в систему!${NC}"
            RESOLVED_CMD="omnifetch"
            return 0
        else
            echo ""
            echo -e "${RED}Ошибка установки. Проверьте права sudo или вывод make.${NC}"
            read -r -p "Продолжить с указанием пути к файлу вручную? [Y/n]: " retry_choice
            if [[ ! "$retry_choice" =~ ^[Nn] ]]; then
                prompt_custom_binary_path
            else
                RESOLVED_CMD="omnifetch"
            fi
        fi
    else
        echo -e "${RED}Makefile не найден в директории $SCRIPT_DIR.${NC}"
        prompt_custom_binary_path
    fi
}

normalize_target_command() {
    local cmd="$1"
    cmd=$(echo "$cmd" | xargs)
    if [ -z "$cmd" ]; then
        echo "omnifetch"
        return 0
    fi

    local first_word="${cmd%% *}"
    local rest_args=""
    if [[ "$cmd" == *" "* ]]; then
        rest_args="${cmd#* }"
    fi

    first_word="${first_word/#\~/$HOME}"

    # Если указан путь к файлу (содержит / или существует как файл)
    if [[ "$first_word" == *"/"* ]] || [ -f "$first_word" ]; then
        local abs_first
        abs_first=$(realpath -m "$first_word" 2>/dev/null || echo "$first_word")
        if [ -f "$abs_first" ] && [ ! -x "$abs_first" ]; then
            chmod +x "$abs_first" 2>/dev/null || true
        fi
        if [ -n "$rest_args" ]; then
            echo "$abs_first $rest_args"
        else
            echo "$abs_first"
        fi
    else
        echo "$cmd"
    fi
}

add_aliases_flow() {
    resolve_omnifetch_executable

    echo ""
    select_shell_and_config

    echo ""
    echo -e "${BOLD}2. Выберите алиасы для добавления:${NC}"
    echo -e "   ${GREEN}1)${NC} fastfetch, neofetch"
    echo -e "   ${GREEN}2)${NC} fastfetch, neofetch, fetch, sysinfo, of"
    echo -e "   ${GREEN}3)${NC} Только fastfetch"
    echo -e "   ${GREEN}4)${NC} Только neofetch"
    echo -e "   ${GREEN}5)${NC} Ввести свои алиасы вручную (через запятую)"
    echo ""
    read -r -p "Выберите вариант [1-5, по умолчанию 1]: " alias_choice

    local aliases=()
    case "$alias_choice" in
        2)
            aliases=("fastfetch" "neofetch" "fetch" "sysinfo" "of")
            ;;
        3)
            aliases=("fastfetch")
            ;;
        4)
            aliases=("neofetch")
            ;;
        5)
            echo ""
            echo -e "${GRAY}Пример: ff, nf, ofetch, myfetch, info${NC}"
            read -r -p "Введите алиасы через запятую: " custom_aliases_str
            IFS=',' read -ra ADDR <<< "$custom_aliases_str"
            for a in "${ADDR[@]}"; do
                local trimmed
                trimmed=$(echo "$a" | xargs)
                if [ -n "$trimmed" ]; then
                    aliases+=("$trimmed")
                fi
            done
            ;;
        1|*)
            aliases=("fastfetch" "neofetch")
            ;;
    esac

    if [ ${#aliases[@]} -eq 0 ]; then
        echo -e "${RED}Ошибка: Список алиасов пуст!${NC}"
        return 1
    fi

    local base_cmd="${RESOLVED_CMD:-omnifetch}"

    echo ""
    echo -e "${BOLD}3. Команда вызова Omnifetch:${NC}"
    echo -e "По умолчанию: ${CYAN}${BOLD}$base_cmd${NC}"
    echo -e "${GRAY}(Вы можете нажать Enter, чтобы использовать её, либо указать свой путь / флаги, например: $base_cmd --fast)${NC}"
    read -r -p "Введите команду [Enter для '$base_cmd']: " target_cmd
    if [ -z "$target_cmd" ]; then
        target_cmd="$base_cmd"
    fi

    # Автоматически нормализуем путь (раскрываем ~, относительные пути переводим в абсолютные)
    target_cmd=$(normalize_target_command "$target_cmd")

    mkdir -p "$(dirname "$SELECTED_CONFIG")"
    touch "$SELECTED_CONFIG"

    local backup_file="${SELECTED_CONFIG}.omnifetch.bak"
    cp "$SELECTED_CONFIG" "$backup_file"
    echo ""
    echo -e "${GRAY}Создана резервная копия: $backup_file${NC}"

    local alias_lines=()
    for name in "${aliases[@]}"; do
        if [ "$SELECTED_SHELL" = "fish" ]; then
            alias_lines+=("alias $name '$target_cmd'")
        else
            alias_lines+=("alias $name='$target_cmd'")
        fi
    done

    if grep -qF "$MARKER_START" "$SELECTED_CONFIG"; then
        local tmp_file
        tmp_file=$(mktemp)
        awk -v start="$MARKER_START" -v end="$MARKER_END" '
            $0 == start { skip=1; next }
            $0 == end { skip=0; next }
            !skip { print }
        ' "$SELECTED_CONFIG" > "$tmp_file"
        mv "$tmp_file" "$SELECTED_CONFIG"
    fi

    {
        echo ""
        echo "$MARKER_START"
        for line in "${alias_lines[@]}"; do
            echo "$line"
        done
        echo "$MARKER_END"
    } >> "$SELECTED_CONFIG"

    echo ""
    echo -e "${GREEN}${BOLD}✔ Алиасы успешно добавлены в ${SELECTED_CONFIG}!${NC}"
    echo ""
    echo -e "Добавленные алиасы:"
    for line in "${alias_lines[@]}"; do
        echo -e "  ${CYAN}$line${NC}"
    done
    echo ""
    echo -e "${YELLOW}${BOLD}Чтобы применить изменения прямо сейчас, выполните:${NC}"
    if [ "$SELECTED_SHELL" = "fish" ]; then
        echo -e "  ${BOLD}source $SELECTED_CONFIG${NC}"
    else
        echo -e "  ${BOLD}source $SELECTED_CONFIG${NC} ${GRAY}(или перезапустите терминал)${NC}"
    fi
}

remove_aliases_flow() {
    select_shell_and_config

    if [ ! -f "$SELECTED_CONFIG" ]; then
        echo -e "${RED}Файл $SELECTED_CONFIG не найден!${NC}"
        return 1
    fi

    local has_marker=0
    local has_standalone=0

    if grep -qF "$MARKER_START" "$SELECTED_CONFIG"; then
        has_marker=1
    fi

    if grep -E -q 'alias [a-zA-Z0-9_-]+.*omnifetch' "$SELECTED_CONFIG"; then
        has_standalone=1
    fi

    if [ $has_marker -eq 0 ] && [ $has_standalone -eq 0 ]; then
        echo ""
        echo -e "${YELLOW}В файле ${SELECTED_CONFIG} не обнаружено алиасов Omnifetch.${NC}"
        return 0
    fi

    echo ""
    echo -e "${BOLD}Найденные записи Omnifetch в ${SELECTED_CONFIG}:${NC}"
    grep -E 'alias [a-zA-Z0-9_-]+.*omnifetch' "$SELECTED_CONFIG" | while read -r line; do
        echo -e "  ${RED}- $line${NC}"
    done
    echo ""

    read -r -p "Вы действительно хотите удалить эти алиасы? [y/N]: " confirm_del
    if [[ ! "$confirm_del" =~ ^[YyДд] ]]; then
        echo -e "${GRAY}Удаление отменено пользователем.${NC}"
        return 0
    fi

    local backup_file="${SELECTED_CONFIG}.omnifetch.bak"
    cp "$SELECTED_CONFIG" "$backup_file"
    echo -e "${GRAY}Создана резервная копия: $backup_file${NC}"

    local tmp_file
    tmp_file=$(mktemp)

    awk -v start="$MARKER_START" -v end="$MARKER_END" '
        $0 == start { skip=1; next }
        $0 == end { skip=0; next }
        !skip { print }
    ' "$SELECTED_CONFIG" > "$tmp_file"

    grep -v -E 'alias [a-zA-Z0-9_-]+.*omnifetch' "$tmp_file" > "${tmp_file}.2" || true
    mv "${tmp_file}.2" "$SELECTED_CONFIG"
    rm -f "$tmp_file"

    echo ""
    echo -e "${GREEN}${BOLD}✔ Все алиасы Omnifetch были успешно удалены из ${SELECTED_CONFIG}!${NC}"
    echo -e "${GRAY}(Резервная копия сохранена в: $backup_file)${NC}"
    echo ""
    echo -e "${YELLOW}${BOLD}Чтобы изменения вступили в силу:${NC}"
    echo -e "  ${BOLD}source $SELECTED_CONFIG${NC} ${GRAY}(или перезапустите терминал)${NC}"
}

main_menu() {
    while true; do
        banner
        echo -e "${BOLD}Главное меню:${NC}"
        echo -e "  ${GREEN}1)${NC} Добавить алиасы"
        echo -e "  ${RED}2)${NC} Убрать алиасы"
        echo -e "  ${GRAY}0)${NC} Выход"
        echo ""
        read -r -p "Выберите действие [0-2]: " action

        case "$action" in
            1)
                echo ""
                add_aliases_flow
                echo ""
                read -r -p "Нажмите Enter, чтобы вернуться в меню..." _
                ;;
            2)
                echo ""
                remove_aliases_flow
                echo ""
                read -r -p "Нажмите Enter, чтобы вернуться в меню..." _
                ;;
            0|q|Q|"")
                echo -e "\n${CYAN}До встречи!${NC}\n"
                exit 0
                ;;
            *)
                echo -e "${RED}Неверный пункт меню! Попробуйте снова.${NC}"
                sleep 1
                ;;
        esac
    done
}

main_menu
