# Installation

## One-Line Installer

```bash
curl -fsSL https://konductor.cloud/install | bash
```

This downloads the latest release and installs to `~/.kiro/` (global).

### Options

| Flag | Description |
|------|-------------|
| `--global`, `-g` | Install to `~/.kiro/` (default) |
| `--local`, `-l` | Install to `./.kiro/` in current project |

```bash
# Install locally to current project
curl -fsSL https://konductor.cloud/install | bash -s -- --local
```

## From Source

```bash
git clone https://github.com/bnusunny/konductor.git
cd konductor
./install.sh
```

### Build the Hook Binary

The hook binary provides file tracking and safety guardrails. To build from source:

```bash
cd konductor-hook
cargo build --release
cd ..
./install.sh
```

## Requirements

- [Kiro CLI](https://kiro.dev) installed and configured
- `bash`, `curl` or `wget` for the one-line installer
- Rust toolchain (only if building from source)
