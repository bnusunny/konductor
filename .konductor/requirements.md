# Requirements

## REQ-01: Robust State Machine
The konductor-cli state machine must handle all valid transitions, reject invalid ones, and recover gracefully from corrupted state. Edge cases like re-initialization, interrupted execution resume, and concurrent blocker management must be handled.

## REQ-02: Comprehensive Test Coverage
The konductor-cli Rust binary must have unit tests for all modules (state, mcp, hook) and integration tests for the MCP server protocol. Current test coverage is minimal (only state transitions and hook destructive command detection).

## REQ-03: Error Handling and Resilience
All MCP tool handlers must return structured error responses. File I/O errors, TOML parsing failures, and missing directory conditions must be handled gracefully with actionable error messages.

## REQ-04: Plan Frontmatter Parsing
The plan frontmatter parser must handle all documented fields (phase, plan, wave, depends_on, type, autonomous, requirements, files_modified, must_haves). Currently only wave and title are parsed.

## REQ-05: Config Management
A config.toml management system must exist — reading defaults, validating fields, and providing config values to MCP tool consumers. Currently config.toml is referenced in skills but not managed by the CLI.

## REQ-06: CLI Usability
The konductor binary must provide helpful error messages, usage hints, and version information. The `--version` flag should report the current version.

## REQ-07: Install Script Reliability
The install.sh script must handle edge cases: existing installations, permission errors, unsupported platforms, network failures during binary download, and checksum verification.

## REQ-08: Documentation Completeness
The docs-site must document all commands, configuration options, the state machine, plan file format, and troubleshooting guides. Architecture docs must reflect the current MCP-based design.

## REQ-09: CI/CD Pipeline Robustness
GitHub Actions workflows must handle: release-please version bumping, cross-platform binary builds, docs deployment, and conventional commit enforcement. Build failures must not produce partial releases.

## REQ-10: Hook System Completeness
The hook system must track all file modifications accurately and block all categories of destructive commands. The destructive command list should be configurable or extensible.
