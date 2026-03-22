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
| `--force`, `-f` | Overwrite existing files (default: skip) |

```bash
# Install locally to current project
curl -fsSL https://konductor.cloud/install | bash -s -- --local

# Force reinstall (overwrites customizations)
curl -fsSL https://konductor.cloud/install | bash -s -- --force
```

!!! note
    By default, the installer skips files that already exist to preserve your customizations. Use `--force` to overwrite everything.

## From Source

```bash
git clone https://github.com/bnusunny/konductor.git
cd konductor
```

### Build the Konductor Binary

The `konductor` binary provides the MCP server and hook processor:

```bash
cd konductor-cli
cargo build --release
cd ..
./install.sh
```

## What Gets Installed

| Component | Path | Description |
|-----------|------|-------------|
| Agents | `agents/*.json` | Orchestrator and subagent configs |
| Skills | `skills/konductor-*/` | Skill instructions for each command |
| Hooks | `hooks/konductor-hooks.json` | Hook configuration |
| Binary | `bin/konductor` | Unified binary (MCP server + hook processor) |

## Requirements

- [Kiro CLI](https://kiro.dev) installed and configured
- `bash`, `curl` or `wget` for the one-line installer
- Rust toolchain (only if building from source)
