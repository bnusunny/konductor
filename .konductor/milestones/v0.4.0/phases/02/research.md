# Phase 02 Research: E2E Pipeline Tests and Synthetic Project

## E2E Testing Strategy

### How kiro-cli + konductor works
- `kiro-cli chat --agent konductor --no-interactive --trust-all-tools "message"` sends a single message to the konductor agent
- The agent uses shell, read, write, and code tools (NOT MCP tools directly — those are for the orchestrator's internal use)
- The agent reads/writes `.konductor/` files to manage pipeline state
- Each prompt message triggers a pipeline step (init, plan, exec, verify, ship)

### E2E Test Approach
Bash test scripts that:
1. Set up a clean test directory with a synthetic project
2. Drive `kiro-cli chat --agent konductor` with pipeline commands
3. Validate artifacts on disk after each step
4. Collect timing metrics

### Two test tiers:
- **Tier 1: MCP protocol tests** (already done in phase 01) — fast, no LLM needed
- **Tier 2: Full agent E2E tests** — requires LLM access, drives kiro-cli, validates real pipeline output

### Synthetic Project
A minimal project that konductor can plan and execute:
- Simple enough to complete quickly (minimize LLM cost/time)
- Complex enough to exercise all pipeline phases
- Pre-defined spec so results are comparable across runs

### Benchmark Metrics
- Per-step wall-clock time (init, plan, exec, verify)
- Number of plans generated
- Verification pass/fail
- Gap count
- Total credits consumed (from kiro-cli output)

### Local-first
- Scripts run locally via `bash tests/e2e/run.sh`
- CI integration is a follow-up concern
- Tests gated behind `KONDUCTOR_E2E=1` env var since they require LLM access
