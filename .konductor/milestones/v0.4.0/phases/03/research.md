# Phase 03 Research: Self-Improvement Loop

## How the Loop Works

1. Run benchmark → get results.toml
2. Analyze results → identify worst-performing step (slowest, most gaps, verification failures)
3. Read the skill file for that step → identify what could be improved
4. Generate an improved version of the skill → write to a staging area
5. Re-run benchmark with the improved skill → get new results.toml
6. Compare → if improved, keep; if regressed, revert
7. Repeat up to max iterations

## Architecture

The improvement loop is a bash script that:
- Drives `kiro-cli chat` to analyze benchmark results and suggest improvements
- Uses git to manage skill versions (branch per iteration, revert on regression)
- Calls the existing benchmark.sh and compare.sh scripts

### Key Design Decisions

**Script-based, not Rust:** The improvement loop is orchestration logic — bash scripts calling kiro-cli. No need for compiled code.

**Git-based versioning:** Each improvement iteration creates a git stash or branch. Revert is just `git checkout -- skills/`. Simple and reliable.

**Bounded:** Max 3 iterations hardcoded. Each iteration has a timeout.

**Local-first:** Runs locally like the E2E tests. CI workflow is a thin wrapper.

## Improvement Strategy

The agent analyzes benchmark results and focuses on:
- **Slow steps:** If plan_time or exec_time is high, improve the corresponding skill's instructions to be more concise/focused
- **High gap count:** If verification finds gaps, improve the exec skill to be more thorough
- **Verification failures:** If verification status is issues-found, improve the verify skill's criteria

The agent reads the current SKILL.md, the benchmark results, and generates a revised version with specific changes.

## CI Workflow

A GitHub Actions workflow that:
1. Runs `cargo test` (unit + MCP integration)
2. If KONDUCTOR_E2E secret is set, runs E2E + benchmark
3. Uploads benchmark results as artifact
4. Optionally runs improvement loop (gated behind separate flag)
