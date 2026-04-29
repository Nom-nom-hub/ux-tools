#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
REPO="nom-nom-hub/ux-tools"
TAG="v0.1.0"

get_os() {
    case "$(uname -s)" in
        Linux*)  echo "x86_64-unknown-linux-gnu" ;;
        Darwin*)
            case "$(uname -m)" in
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                *)          echo "x86_64-apple-darwin" ;;
            esac
            ;;
        *)      echo "" ;;
    esac
}

install() {
    local os
    os="$(get_os)"
    
    if [ -z "$os" ]; then
        echo "Error: Unsupported OS" >&2
        exit 1
    fi

    local url="https://github.com/${REPO}/releases/download/${TAG}/ux-${os}.tar.gz"
    
    echo "Downloading ux ${TAG} for ${os}..."
    
    mkdir -p "$INSTALL_DIR"
    
    if command -v curl >/dev/null 2>&1; then
        curl -LsSf "$url" | tar -xzf - -C "$INSTALL_DIR"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$url" | tar -xzf - -C "$INSTALL_DIR"
    else
        echo "Error: Neither curl nor wget found" >&2
        exit 1
    fi
    
    chmod +x "${INSTALL_DIR}/ux"
    
    echo "Installed ux to ${INSTALL_DIR}/ux"
    
    # Add to PATH
    if [ -n "$ZSH_VERSION" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc 2>/dev/null
    elif [ -n "$BASH_VERSION" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc 2>/dev/null
    fi
    
    echo ""
    echo "Usage:"
    echo "  ux ruff -- --version"
    echo "  ux warm ruff"
}

install