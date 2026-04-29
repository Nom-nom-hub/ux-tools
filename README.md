# ux — Better Python Tool Runner

<p align="center">
  <a href="https://github.com/nom-nom-hub/ux-tools/actions/workflows/release.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/nom-nom-hub/ux-tools/release.yml?style=flat-square" alt="CI">
  </a>
  <a href="https://github.com/nom-nom-hub/ux-tools/releases">
    <img src="https://img.shields.io/github/v/release/nom-nom-hub/ux-tools?style=flat-square" alt="Version">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/nom-nom-hub/ux-tools?style=flat-square" alt="License">
  </a>
</p>

<p align="center">
  A faster, smarter Python tool runner that improves on uvx with pre-warmed caches, native venvs, and multiple sources.
</p>

## Why ux?

| Feature | uvx | ux |
|---------|-----|-----|
| Pre-warm cache | ❌ | ✅ |
| Native venv creation | ❌ | ✅ |
| PyPI | ✅ | ✅ |
| GitHub/Gist/URL | ❌ | ✅ |
| Offline mode | ❌ | ✅ |
| Single binary | ❌ | ✅ |

## Installation

### One-liner (recommended)

```bash
curl -LsSf https://raw.githubusercontent.com/nom-nom-hub/ux-tools/main/install.sh | sh
```

### Direct Download

```bash
# macOS (Apple Silicon)
curl -LsSf https://github.com/nom-nom-hub/ux-tools/releases/download/v0.1.0/ux -o ~/.local/bin/ux
chmod +x ~/.local/bin/ux

# Add to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Build from Source

```bash
git clone https://github.com/nom-nom-hub/ux-tools
cd ux-tools
cargo build --release
cp target/release/ux ~/.local/bin/ux

## Quick Start

```bash
# Run a tool (native execution)
ux ruff -- --version
ux black --check main.py
ux httpie GET https://example.com

# Pre-warm environment (faster on next run)
ux warm ruff
ux warm --all

# Cache management
ux cache ls
ux cache rm ruff
ux cache clean

# Run from cache only (offline)
ux --offline ruff
```

## Usage

```
ux <tool> [args...]        # Run a tool
ux warm <tool>            # Pre-warm environment
ux warm --all             # Pre-warm all cached tools
ux --use-uv <tool>       # Run via uv (faster installs)
ux --offline <tool>      # Run from cache only

# Cache commands
ux cache ls              # List cached tools
ux cache rm <tool>       # Remove cached tool
ux cache clean           # Clear all cache
```

## Multiple Sources

In addition to PyPI packages, ux supports:

```bash
# PyPI (default)
ux ruff
ux black@latest

# GitHub raw file
ux github:owner/repo/path/to/script.py
ux github:owner/repo/path@v1.0.0

# Gist
ux gist:gist_id
ux gist:gist_id:filename.py

# Raw URL
ux https://example.com/script.py

# Local file
ux ./local/script.py
```

## How It Works

1. **Native Mode** (default): Creates its own virtual environment via Python's venv module, installs the package via pip, then executes directly.

2. **UV Mode** (`--use-uv`): Delegates to uv for package resolution and installation. Faster for first runs, but requires uv installed.

3. **Cache Warming** (`ux warm`): Pre-builds the virtual environment so subsequent runs are instant.

## Cache Location

- macOS: `~/Library/Caches/com.ux.ux/venvs/`
- Linux: `~/.cache/com.ux.ux/venvs/`

## Requirements

- Python 3.8+ (for venv module)
- pip

## Development

```bash
# Clone
git clone https://github.com/nom-nom-hub/ux-tools
cd ux-tools

# Build
cargo build --release

# Test
cargo run --release -- ruff -- --version

# Add binary to PATH
cp target/release/ux ~/.local/bin/
```

## License

Apache-2.0 — See [LICENSE](LICENSE)

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/nom-nom-hub">@nom-nom-hub</a>
</p>