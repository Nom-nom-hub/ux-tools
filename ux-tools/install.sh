#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="ux"

if [ -d "$INSTALL_DIR" ]; then
    echo "Installing ux to ${INSTALL_DIR}..."
    cp target/release/ux "${INSTALL_DIR}/ux"
    chmod +x "${INSTALL_DIR}/ux"
    echo "Installed ux to ${INSTALL_DIR}/ux"
    echo ""
    echo "Add to your PATH:"
    echo "  source ${HOME}/.local/bin/env"
else
    echo "Creating ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR"
    cp target/release/ux "${INSTALL_DIR}/ux"
    chmod +x "${INSTALL_DIR}/ux"
    echo "Installed ux to ${INSTALL_DIR}/ux"
    echo ""
    echo "Add to your PATH:"
    echo "  source ${HOME}/.local/bin/env"
fi

echo ""
echo "Usage:"
echo "  ux ruff -- --version"
echo "  ux warm ruff"
echo "  ux --help"