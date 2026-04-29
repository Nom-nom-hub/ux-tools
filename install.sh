#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
VERSION="v0.1.0"

get_os() {
    case "$(uname -s)" in
        Linux*)  echo "unknown-linux-gnu" ;;
        Darwin*)
            case "$(uname -m)" in
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                *)          echo "x86_64-apple-darwin" ;;
            esac
            ;;
        *)      echo "unknown" ;;
    esac
}

install() {
    local os
    os="$(get_os)"
    
    if [ "$os" = "unknown" ]; then
        echo "Error: Unsupported OS" >&2
        exit 1
    fi

    local url="https://github.com/nom-nom-hub/ux-tools/releases/download/v0.1.0/ux"
    
    echo "Installing ux for ${os}..."
    
    mkdir -p "$INSTALL_DIR"
    
    if command -v curl >/dev/null 2>&1; then
        curl -LsSf "$url" -o "${INSTALL_DIR}/ux"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "${INSTALL_DIR}/ux"
    else
        echo "Error: Neither curl nor wget found" >&2
        exit 1
    fi
    
    chmod +x "${INSTALL_DIR}/ux"
    
    echo "Installed ux to ${INSTALL_DIR}/ux"
    
    # Try to add to PATH
    local shell_rc=""
    if [ -n "$ZSH_VERSION" ]; then
        shell_rc="${HOME}/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        shell_rc="${HOME}/.bashrc"
    fi
    
    if [ -n "$shell_rc" ] && [ -f "$shell_rc" ]; then
        if ! grep -q '.local/bin' "$shell_rc"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
            echo "Added to ${shell_rc}"
            echo "Run: source ${shell_rc}"
        fi
    fi
    
    echo ""
    echo "Usage: ux <tool> [args...]"
    echo "Example: ux ruff -- --version"
}

install