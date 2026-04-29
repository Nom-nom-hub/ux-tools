# ux — Better Python Tool Runner

A faster, smarter Python tool runner that beats uvx with pre-warmed caches, native venvs, multiple sources, and offline support.

## Why ux?

| Feature | uvx | ux |
|---------|-----|-----|
| Pre-warm cache | ❌ | ✅ |
| Multiple sources | ❌ | ✅ |
| Offline mode | ❌ | ✅ |
| No uv dependency | ❌ | ✅ |
| Simpler caching | ❌ | ✅ |
| Native venv creation | ❌ | ✅ |

uv is great, but it doesn't let you pre-warm environments, run offline, or use GitHub/Gist sources.

## Installation

### One-liner (recommended)

```bash
curl -LsSf https://raw.githubusercontent.com/nom-nom-hub/ux-tools/main/install.sh | sh
```

### Direct Download

```bash
# macOS (Apple Silicon)
curl -LsSf https://github.com/nom-nom-hub/ux-tools/releases/download/v0.1.0/ux-aarch64-apple-darwin.tar.gz | tar -xzf - -C ~/.local/bin

# Linux
curl -LsSf https://github.com/nom-nom-hub/ux-tools/releases/download/v0.1.0/ux-x86_64-unknown-linux-gnu.tar.gz | tar -xzf - -C ~/.local/bin

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
```

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

In addition to PyPI packages:

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