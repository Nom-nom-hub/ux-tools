# ux - Better Python Tool Runner

A faster, smarter Python tool runner that improves on uvx with pre-warmed caches, native venvs, and multiple sources.

## Installation

```bash
# Build
cd ux-tools
cargo build --release

# Install
cp target/release/ux ~/.local/bin/ux
source ~/.local/bin/env  # add to PATH
```

## Usage

```bash
# Run a tool (native execution)
ux ruff -- --version
ux black --check main.py

# Run faster via uv
ux --use-uv ruff -- --version

# Pre-warm environment
ux warm ruff
ux warm --all

# Cache management
ux cache ls
ux cache rm ruff
ux cache clean

# Run from cache only (offline)
ux --offline ruff
```

## Multiple Sources

```bash
# PyPI (default)
ux ruff

# GitHub raw file
ux github:owner/repo/path/to/script.py
ux github:owner/repo/path@ref  # specific branch/tag

# Gist
ux gist:gist_id
ux gist:gist_id:filename

# Raw URL
ux https://example.com/script.py

# Local file
ux ./local/script.py
```

## Features

- **Native Execution** - Creates its own virtual environments via pip
- **Smart Caching** - Pre-warm environments before you need them
- **Dual Mode** - Native (default) or delegate to uv (`--use-uv`)
- **Multiple Sources** - PyPI, GitHub, Gist, URLs, local files
- **Offline Mode** - Run from cache when offline (`--offline`)

## Cache Location

- macOS: `~/Library/Caches/com.ux.ux/venvs/`
- Linux: `~/.cache/com.ux.ux/venvs/`

## Why Better Than uvx?

| Feature | uvx | ux |
|---------|-----|-----|
| Pre-warm cache | ❌ | ✅ |
| Native venv | ❌ | ✅ |
| Multiple sources | PyPI only | +GitHub, Gist, URL |
| Offline mode | ❌ | ✅ |

## Binary Size

~4.8MB (single static binary)