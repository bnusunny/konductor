# Konductor Integration Testing & Self-Improvement

## Vision

Build a comprehensive integration test environment that validates konductor end-to-end with Kiro CLI, then use benchmark results to autonomously improve konductor's prompts, skills, and pipeline quality.

## Target Users

- Konductor maintainers who need confidence that changes don't break the pipeline
- CI systems that gate releases on integration test results
- The konductor agent itself, using benchmark data to self-improve

## Tech Stack

- **Language:** Rust (konductor-cli), Bash (test harness scripts)
- **MCP Testing:** Direct JSON-RPC over stdio to `konductor mcp`
- **E2E Testing:** Scripted Kiro CLI invocations with artifact validation
- **Benchmarking:** TOML-based metrics collection, comparison against baselines
- **CI:** GitHub Actions workflows
- **Test Projects:** Konductor itself (dogfood) + synthetic toy project

## Key Components

1. **MCP Protocol Tests** — Send JSON-RPC requests to `konductor mcp`, validate tool responses (state_get, state_transition, plans_list, etc.), prompt schemas, and error handling
2. **E2E Pipeline Tests** — Run full init→plan→exec→verify→ship pipeline on a synthetic project, validate all artifacts are created correctly
3. **Benchmark Harness** — Measure pipeline speed (per-phase timing), quality (test pass rate, verification pass rate, gap count), and correctness (does generated code compile and pass tests)
4. **Self-Improvement Agent** — Analyze benchmark results, identify underperforming skills/prompts, generate improved versions, re-run benchmarks to validate improvements
5. **CI Integration** — GitHub Actions workflow that runs integration tests and benchmarks on PRs

## Scope

- Integration tests for all MCP tools and prompts
- E2E tests for the full pipeline on a synthetic project
- Benchmark harness with speed, quality, and correctness metrics
- Autonomous skill/prompt improvement loop
- CI workflow for automated testing

## Out of Scope

- Token/cost efficiency metrics (requires API billing integration)
- UI dashboard for benchmark results
- Multi-model comparison (single model benchmarking only)
- Load testing / concurrent pipeline execution
