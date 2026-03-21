# Roadmap

## Phase 01: Core CLI Hardening

**Goal:** Harden the konductor-cli binary with comprehensive tests, robust error handling, full frontmatter parsing, and config management.

**Success Criteria:**
- All state transitions are tested (valid and invalid)
- MCP tool handlers return structured errors for all failure modes
- Plan frontmatter parser handles all documented fields
- Config.toml is read and validated by the CLI
- `cargo test` passes with no failures
- `cargo clippy` reports no warnings

**Requirements:** REQ-01, REQ-02, REQ-03, REQ-04, REQ-05, REQ-06

## Phase 02: Install and Distribution

**Goal:** Make the install script bulletproof and improve the release pipeline.

**Success Criteria:**
- Install script handles all edge cases (existing installs, permissions, unsupported platforms)
- Binary checksums are verified during installation
- CI builds produce consistent, tested artifacts
- Version reporting works end-to-end

**Requirements:** REQ-06, REQ-07, REQ-09

## Phase 03: Documentation and Polish

**Goal:** Complete the documentation site and ensure all features are documented.

**Success Criteria:**
- All commands documented with examples
- Configuration reference is complete
- Architecture docs reflect MCP-based design
- Troubleshooting guide covers common issues
- Hook system is documented

**Requirements:** REQ-08, REQ-10
