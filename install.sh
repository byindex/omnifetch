#!/usr/bin/env bash

# ==============================================================================
# Omnifetch Installer
# https://github.com/byindex/omnifetch
# ==============================================================================

set -e

REPO="byindex/omnifetch"
BINARY_NAME="omnifetch"

BOLD='\033[1m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[0;90m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo ".")"

PREFIX="/usr/local"
USER_INSTALL=false
UNINSTALL=false
FORCE_YES=false

show_help() {
    cat <<EOF
Usage: install.sh [OPTIONS]

Options:
  --user            Install to ~/.local/bin (no root/sudo required)
  --system          Install system-wide to /usr/local/bin (requires root/sudo)
  --prefix <DIR>    Custom installation prefix (default: /usr/local)
  --uninstall       Uninstall omnifetch and shell completions
  -y, --yes         Non-interactive mode, accept defaults
  -h, --help        Show this help message

Examples:
  sudo ./install.sh              # System-wide installation
  ./install.sh --user            # User installation to ~/.local/bin
  curl -fsSL https://raw.githubusercontent.com/byindex/omnifetch/main/install.sh | bash
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --user)
            USER_INSTALL=true
            shift
            ;;
        --system)
            USER_INSTALL=false
            shift
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        -y|--yes)
            FORCE_YES=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            echo "$arch"
            ;;
    esac
}

find_local_binary() {
    local candidates=(
        "$SCRIPT_DIR/omnifetch"
        "$SCRIPT_DIR/target/release/omnifetch"
        "$SCRIPT_DIR/target/$(uname -m)-unknown-linux-gnu/release/omnifetch"
        "$SCRIPT_DIR/target/x86_64-unknown-linux-gnu/release/omnifetch"
    )
    for c in "${candidates[@]}"; do
        if [[ -f "$c" && -x "$c" ]]; then
            echo "$c"
            return 0
        fi
    done
    return 1
}

# Set target paths based on permissions and flags
setup_paths() {
    if [[ "$USER_INSTALL" == true ]] || [[ "$EUID" -ne 0 && "$PREFIX" == "/usr/local" && "$USER_INSTALL" != false ]]; then
        # Default to user install if not root and no explicit prefix given
        USER_INSTALL=true
        BIN_DIR="$HOME/.local/bin"
        BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
        ZSH_COMP_DIR="$HOME/.local/share/zsh/site-functions"
        FISH_COMP_DIR="$HOME/.config/fish/completions"
    else
        BIN_DIR="${PREFIX}/bin"
        BASH_COMP_DIR="${PREFIX}/share/bash-completion/completions"
        ZSH_COMP_DIR="${PREFIX}/share/zsh/site-functions"
        FISH_COMP_DIR="${PREFIX}/share/fish/vendor_completions.d"
    fi
}

do_uninstall() {
    setup_paths
    echo -e "${CYAN}${BOLD}Uninstalling ${BINARY_NAME}...${NC}"

    local targets=(
        "$BIN_DIR/$BINARY_NAME"
        "$BASH_COMP_DIR/$BINARY_NAME"
        "$ZSH_COMP_DIR/_$BINARY_NAME"
        "$FISH_COMP_DIR/$BINARY_NAME.fish"
    )

    if [[ "$EUID" -eq 0 ]]; then
        targets+=(
            "/usr/share/bash-completion/completions/$BINARY_NAME"
            "/usr/share/zsh/site-functions/_$BINARY_NAME"
            "/usr/share/fish/vendor_completions.d/$BINARY_NAME.fish"
        )
    fi

    local removed=0
    for target in "${targets[@]}"; do
        if [[ -f "$target" ]]; then
            if rm -f "$target" 2>/dev/null; then
                echo -e "  ${GREEN}✔${NC} Removed $target"
                removed=$((removed + 1))
            fi
        fi
    done

    if [[ $removed -eq 0 ]]; then
        echo -e "${YELLOW}No installed files were found.${NC}"
    else
        echo -e "${GREEN}${BOLD}Omnifetch was successfully uninstalled.${NC}"
    fi
    exit 0
}

if [[ "$UNINSTALL" == true ]]; then
    do_uninstall
fi

# Banner
echo -e "${CYAN}${BOLD}"
echo "  ╔═══════════════════════════════════════════════════════════╗"
echo "  ║             ⚡ OMNIFETCH INSTALLER                        ║"
echo "  ╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Detect if we should use user install or prompt
if [[ "$EUID" -ne 0 && "$USER_INSTALL" == false && "$PREFIX" == "/usr/local" ]]; then
    if [[ "$FORCE_YES" == false && -t 0 ]]; then
        echo -e "${YELLOW}Running without root privileges.${NC}"
        echo "Choose installation location:"
        echo "  [1] System-wide with sudo (/usr/local/bin) (Recommended)"
        echo "  [2] Current user only (~/.local/bin)"
        read -r -p "Select [1/2] (default: 1): " choice
        case "$choice" in
            2)
                USER_INSTALL=true
                ;;
            *)
                echo -e "${GRAY}Elevating privileges using sudo...${NC}"
                exec sudo bash "$0" "$@"
                ;;
        esac
    else
        # Non-interactive without root: default to user install
        USER_INSTALL=true
    fi
fi

setup_paths

echo -e "Target directory: ${BOLD}$BIN_DIR${NC}"
echo ""

WORK_DIR=""
CLEANUP_WORK_DIR=false

cleanup() {
    if [[ "$CLEANUP_WORK_DIR" == true && -n "$WORK_DIR" && -d "$WORK_DIR" ]]; then
        rm -rf "$WORK_DIR"
    fi
}
trap cleanup EXIT

# 1. Locate or download binary
SRC_BIN=""
if SRC_BIN=$(find_local_binary); then
    echo -e "${GREEN}✔${NC} Using local binary: ${GRAY}$SRC_BIN${NC}"
else
    ARCH=$(detect_arch)
    echo -e "${CYAN}→${NC} Downloading latest release for Linux ${BOLD}$ARCH${NC}..."

    WORK_DIR=$(mktemp -d -t omnifetch-install-XXXXXX)
    CLEANUP_WORK_DIR=true

    LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)
    if [[ -z "$LATEST_TAG" ]]; then
        LATEST_TAG="v0.1.1"
    fi

    TAR_NAME="omnifetch-${LATEST_TAG}-linux-${ARCH}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${TAR_NAME}"

    echo -e "  Downloading ${BOLD}$LATEST_TAG${NC} from $DOWNLOAD_URL"
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$WORK_DIR/$TAR_NAME"; then
        echo -e "${RED}✖ Failed to download release archive.${NC}"
        echo "Please check your internet connection or install manually from source."
        exit 1
    fi

    tar -xzf "$WORK_DIR/$TAR_NAME" -C "$WORK_DIR"
    EXTRACTED_DIR=$(find "$WORK_DIR" -mindepth 1 -maxdepth 1 -type d | head -n 1)

    if [[ -f "$EXTRACTED_DIR/omnifetch" ]]; then
        SRC_BIN="$EXTRACTED_DIR/omnifetch"
        SCRIPT_DIR="$EXTRACTED_DIR"
    elif [[ -f "$WORK_DIR/omnifetch" ]]; then
        SRC_BIN="$WORK_DIR/omnifetch"
        SCRIPT_DIR="$WORK_DIR"
    else
        echo -e "${RED}✖ Extracted archive does not contain omnifetch binary.${NC}"
        exit 1
    fi
fi

# 2. Install binary
echo -e "${CYAN}→${NC} Installing binary to ${BOLD}$BIN_DIR/$BINARY_NAME${NC}..."
mkdir -p "$BIN_DIR"
install -m 755 "$SRC_BIN" "$BIN_DIR/$BINARY_NAME"
echo -e "  ${GREEN}✔${NC} Binary installed successfully"

# 3. Install shell completions
echo -e "${CYAN}→${NC} Installing shell completions..."

# Bash
mkdir -p "$BASH_COMP_DIR"
if [[ -f "$SCRIPT_DIR/completions/omnifetch.bash" ]]; then
    install -m 644 "$SCRIPT_DIR/completions/omnifetch.bash" "$BASH_COMP_DIR/$BINARY_NAME"
elif "$BIN_DIR/$BINARY_NAME" --completion bash >/dev/null 2>&1; then
    "$BIN_DIR/$BINARY_NAME" --completion bash > "$BASH_COMP_DIR/$BINARY_NAME"
fi
echo -e "  ${GREEN}✔${NC} bash: $BASH_COMP_DIR/$BINARY_NAME"

# Zsh
mkdir -p "$ZSH_COMP_DIR"
if [[ -f "$SCRIPT_DIR/completions/_omnifetch" ]]; then
    install -m 644 "$SCRIPT_DIR/completions/_omnifetch" "$ZSH_COMP_DIR/_$BINARY_NAME"
elif "$BIN_DIR/$BINARY_NAME" --completion zsh >/dev/null 2>&1; then
    "$BIN_DIR/$BINARY_NAME" --completion zsh > "$ZSH_COMP_DIR/_$BINARY_NAME"
fi
echo -e "  ${GREEN}✔${NC} zsh: $ZSH_COMP_DIR/_$BINARY_NAME"

# Fish
mkdir -p "$FISH_COMP_DIR"
if [[ -f "$SCRIPT_DIR/completions/omnifetch.fish" ]]; then
    install -m 644 "$SCRIPT_DIR/completions/omnifetch.fish" "$FISH_COMP_DIR/$BINARY_NAME.fish"
elif "$BIN_DIR/$BINARY_NAME" --completion fish >/dev/null 2>&1; then
    "$BIN_DIR/$BINARY_NAME" --completion fish > "$FISH_COMP_DIR/$BINARY_NAME.fish"
fi
echo -e "  ${GREEN}✔${NC} fish: $FISH_COMP_DIR/$BINARY_NAME.fish"

# 4. PATH check for user install
if [[ "$USER_INSTALL" == true ]]; then
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo ""
        echo -e "${YELLOW}Notice: $HOME/.local/bin is not in your current PATH.${NC}"
        echo -e "Add it to your shell configuration (e.g. ~/.bashrc or ~/.zshrc):"
        echo -e "  ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    fi
fi

# 5. Success summary
echo ""
echo -e "${GREEN}${BOLD}✔ Omnifetch was installed successfully!${NC}"
if "$BIN_DIR/$BINARY_NAME" --version >/dev/null 2>&1; then
    echo -e "Version: ${CYAN}$("$BIN_DIR/$BINARY_NAME" --version)${NC}"
fi

echo ""
echo -e "To configure shell aliases (e.g. replace neofetch/fastfetch with omnifetch):"
if [[ -f "$SCRIPT_DIR/setup-aliases.sh" ]]; then
    echo -e "  ${BOLD}./setup-aliases.sh${NC}"
else
    echo -e "  Run the alias setup wizard if bundled with your download."
fi
echo -e "To run omnifetch right now: ${BOLD}omnifetch${NC}"
