# Installation

## Via npm

```bash
npm install -g konductor
```

Requires Node.js 16+. The npm package automatically downloads the correct platform binary and installs agents and skills to `~/.kiro/`.

## From Source

```bash
git clone https://github.com/bnusunny/konductor.git
cd konductor/konductor-cli
cargo build --release
```

## What Gets Installed

| Component | Path | Description |
|-----------|------|-------------|
| Agents | `~/.kiro/agents/*.json` | Orchestrator and subagent configs (includes hook configuration) |
| Skills | `~/.kiro/skills/konductor-*/` | Skill instructions for each command |
| Binary | `konductor` | Unified binary (MCP server + hook processor) |

## Requirements

- [Kiro CLI](https://kiro.dev) installed and configured
- Node.js 16+ (for npm install)
- Rust toolchain (only if building from source)
