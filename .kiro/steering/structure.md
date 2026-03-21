# Structure — Konductor

## Repository Layout
```
konductor/
├── konductor-cli/          # Rust binary (MCP server + hook processor)
│   ├── src/
│   │   ├── main.rs         # CLI entry point (mcp/hook subcommands)
│   │   ├── mcp.rs          # MCP server (tools + prompts)
│   │   ├── state.rs        # State machine (read/write/transition)
│   │   ├── config.rs       # Config management
│   │   └── hook.rs         # Hook event processor
│   └── tests/              # Rust integration tests
├── skills/                 # Skill definitions (markdown)
│   ├── konductor-init/
│   ├── konductor-plan/
│   ├── konductor-exec/
│   ├── konductor-verify/
│   └── ...
├── agents/                 # Agent configs (JSON)
├── hooks/                  # Hook configs
├── tests/
│   ├── e2e/                # E2E pipeline tests + benchmarks
│   │   ├── run.sh          # Full pipeline test
│   │   ├── benchmark.sh    # Benchmark harness
│   │   ├── compare.sh      # Baseline comparison
│   │   ├── improve.sh      # Self-improvement loop
│   │   ├── lib.sh          # Test helpers
│   │   └── synthetic-project/  # Synthetic project specs
│   └── test_installer.sh
├── .konductor/             # Project state (per-milestone)
├── .kiro/steering/         # Steering docs (product, tech, structure)
├── docs-site/              # MkDocs documentation
└── install.sh              # Installer script
```

## Key Patterns
- MCP server exposes tools for deterministic state management (no LLM-generated TOML)
- Skills are markdown files with step-by-step instructions
- Agents are JSON configs that reference skills and tools
- Hooks intercept tool use for file tracking and safety guardrails
- Benchmark harness uses temp directories for isolation
