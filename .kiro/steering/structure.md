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
│   ├── konductor-discuss/
│   ├── konductor-plan/
│   ├── konductor-exec/
│   ├── konductor-verify/
│   ├── konductor-ship/
│   ├── konductor-next/
│   ├── konductor-status/
│   └── konductor-map-codebase/
├── agents/                 # Agent configs (JSON)
│   ├── konductor.json      # Main orchestrator agent
│   ├── konductor-planner.json
│   ├── konductor-executor.json
│   ├── konductor-verifier.json
│   ├── konductor-researcher.json
│   ├── konductor-discoverer.json
│   ├── konductor-plan-checker.json
│   └── konductor-codebase-mapper.json
├── npm/                    # npm package wrapper
│   ├── package.json
│   ├── bin/konductor       # Shell wrapper for binary
│   ├── scripts/
│   │   ├── postinstall.js  # Downloads platform binary from GitHub releases
│   │   └── sync-version.js # Syncs version from version.txt
│   └── README.md
├── tests/
│   ├── acp-harness/        # Rust ACP test harness (replaces bash E2E)
│   │   ├── Cargo.toml      # Test crate with agent-client-protocol dep
│   │   ├── src/
│   │   │   └── lib.rs      # Shared test helpers (ACP client setup)
│   │   └── tests/
│   │       ├── integration/ # MCP tool tests via ACP
│   │       └── e2e/        # Full pipeline tests via ACP
│   ├── e2e/                # Legacy bash tests (benchmark/deploy only)
│   │   ├── compare.sh      # Baseline comparison
│   │   ├── improve.sh      # Self-improvement loop
│   │   ├── deploy.sh       # SAM deployment
│   │   ├── verify-deployment.sh  # Deployed resource verification
│   │   ├── teardown.sh     # SAM teardown
│   │   ├── lib.sh          # Test helpers
│   │   ├── baselines/      # Per-project baseline metrics
│   │   ├── results/        # Benchmark run results
│   │   └── synthetic-projects/   # Synthetic project specs
│   │       ├── cli-calculator/
│   │       ├── rest-api-sam/
│   │       └── api-dynamodb-sam/
│   └── test_installer.sh
├── .konductor/             # Project state (per-milestone)
├── .kiro/steering/         # Steering docs (product, tech, structure)
├── .github/workflows/      # CI workflows
│   ├── build-release.yml   # Build + release binaries
│   ├── test.yml            # Run tests
│   ├── npm-publish.yml     # Publish npm package
│   ├── docs.yml            # Deploy MkDocs site
│   ├── release-please.yml  # Automated releases
│   └── conventional-commits.yml
├── docs-site/              # MkDocs documentation
├── docs/                   # GitHub Pages (CNAME, one-line installer)
└── version.txt             # Single source of truth for version
```

## Key Patterns
- MCP server exposes tools for deterministic state management (no LLM-generated TOML)
- Skills are markdown files with step-by-step instructions
- Agents are JSON configs that reference skills and tools
- Hooks use `"*"` matcher to intercept all tool calls for steering
- Tool call ledger (append-only JSONL) enables workflow validation rules
- Steering hooks provide just-in-time guidance via PreToolUse exit code 2 → STDERR feedback to LLM
- Benchmark harness uses temp directories for isolation
