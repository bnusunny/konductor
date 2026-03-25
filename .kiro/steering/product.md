# Product — Konductor v0.8.x

## Purpose
Spec-driven development orchestrator for Kiro CLI. Manages project initialization, phase planning, execution, verification, and shipping through an MCP server and hook processor. Includes self-improvement capabilities via synthetic project benchmarking and AWS deployment verification.

## Target Users
- Developers using Kiro CLI who want structured, spec-driven project execution
- Konductor maintainers validating skill quality via benchmarks
- CI systems gating releases on benchmark results and deployment verification
- The self-improvement agent using per-project metrics

## Key Features
- MCP server with deterministic state management tools (state_init, state_get, state_transition, state_add_blocker, state_resolve_blocker, plans_list, status)
- MCP prompts for pipeline stages (k-init, k-plan, k-exec, k-verify, k-ship, k-next, k-status, k-discuss, k-map)
- Hook processor for file tracking, safety guardrails, stuck detection, tool call ledger, and workflow validation (pre/post tool use)
- Skill-based pipeline: init → discuss → plan → exec → verify → ship
- Multiple synthetic test projects (CLI calculator, REST API SAM, API + DynamoDB SAM)
- Per-project granular benchmark metrics (timing, test pass rates, lint, artifact quality)
- SAM deployment pipeline (deploy → verify → teardown)
- Self-improvement agent with per-project analysis
- npm distribution (`npm install -g konductor`)
- Installer script with checksum verification and idempotent operation
- MkDocs documentation site
- Rust-based E2E test harness using Agent Client Protocol (ACP)
- Statically linked Linux binaries (musl)
