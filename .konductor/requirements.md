# Requirements

## REQ-01: MCP Protocol Integration Tests
The test suite must validate all MCP tools (state_get, state_transition, state_add_blocker, state_resolve_blocker, plans_list, status, config_get) and all MCP prompts (k-init, k-plan, k-exec, k-verify, k-ship, k-next, k-status, k-discuss, k-map) by sending JSON-RPC requests over stdio to `konductor mcp` and validating responses match expected schemas.

## REQ-02: E2E Pipeline Tests
The test suite must run a complete init→plan→execute→verify→ship pipeline on a synthetic toy project and validate that all expected artifacts are created (.konductor/state.toml, plans, summaries, verification reports). The synthetic project must be self-contained and reproducible.

## REQ-03: Synthetic Test Project
A small, deterministic test project must exist that konductor can plan and execute against. It should be simple enough to complete in a single pipeline run (e.g., a CLI calculator in Rust or Python) but complex enough to exercise all pipeline phases.

## REQ-04: Benchmark Harness
A benchmark harness must collect metrics for each pipeline run: per-phase wall-clock time, number of plans generated, verification pass/fail rate, gap count, and whether generated code compiles and passes tests. Results must be stored in a structured format (TOML or JSON) for comparison.

## REQ-05: Baseline Metrics
An initial baseline benchmark must be established by running the pipeline on the synthetic project. Future runs are compared against this baseline to detect regressions or improvements.

## REQ-06: Self-Improvement Loop
An autonomous agent must analyze benchmark results, identify skills or prompts that produce poor outcomes (high gap count, verification failures, slow execution), generate improved versions of those skills/prompts, and re-run benchmarks to validate the improvements. The loop must be bounded (max iterations) to prevent runaway execution.

## REQ-07: CI Integration
A GitHub Actions workflow must run MCP protocol tests and E2E pipeline tests on pull requests. Benchmark results must be reported as PR comments or artifacts. Regressions against baseline must fail the CI check.

## REQ-08: Test Reproducibility
All tests must be deterministic and isolated. Each test run must start from a clean state (fresh .konductor/ directory). Tests must not depend on network access or external services beyond what's available in CI.
