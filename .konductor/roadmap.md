# Roadmap

## Phase 01: MCP Protocol Integration Tests

**Goal:** Build a test harness that validates all MCP tools and prompts by communicating with `konductor mcp` over stdio JSON-RPC.

**Success Criteria:**
- Test harness can spawn `konductor mcp`, send JSON-RPC requests, and parse responses
- All 7 MCP tools are tested (valid inputs, error cases, state mutations)
- All 9 MCP prompts are tested (correct message content, argument handling)
- Tests run in CI via `cargo test` or a dedicated test script
- Tests are isolated (each test starts with fresh .konductor/ state)

**Requirements:** REQ-01, REQ-08

## Phase 02: E2E Pipeline Tests and Synthetic Project

**Goal:** Create a synthetic test project and run the full konductor pipeline against it, validating all artifacts.

**Success Criteria:**
- Synthetic project exists with clear spec (e.g., CLI calculator)
- E2E test runs init→plan→execute→verify→ship and validates all artifacts
- Test is reproducible and runs in CI
- Benchmark harness collects timing, quality, and correctness metrics
- Baseline metrics are established and stored

**Requirements:** REQ-02, REQ-03, REQ-04, REQ-05, REQ-07, REQ-08

## Phase 03: Self-Improvement Loop

**Goal:** Build an autonomous agent that analyzes benchmark results and improves konductor's skills and prompts.

**Success Criteria:**
- Agent reads benchmark results and identifies underperforming skills
- Agent generates improved skill/prompt versions
- Agent re-runs benchmarks to validate improvements
- Improvement loop is bounded (max 3 iterations)
- Improvements that regress metrics are automatically reverted
- CI workflow runs the full test + benchmark + improve cycle

**Requirements:** REQ-06, REQ-07
