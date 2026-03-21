# Phase 01 Research: Core CLI Hardening

## Current State Analysis

### Codebase Metrics
- **Total Rust LOC:** 718 (hook.rs: 133, main.rs: 29, mcp.rs: 354, state.rs: 202)
- **Existing tests:** 5 (3 state transition validation, 2 hook destructive command detection)
- **Clippy warnings:** 0
- **Cargo test:** All 5 pass

### Module Analysis

#### state.rs — State Machine
**What exists:**
- `KonductorState` struct with nested `Project`, `Current`, `Progress`, `Metrics`, `Blocker`, `Release`
- All fields are `Option<T>` — very permissive, no validation on read
- `read_state()` / `write_state()` — basic file I/O with error mapping
- `validate_transition()` — checks against a static `TRANSITIONS` table
- `transition()` — reads state, validates, updates step/status/phase, writes back
- `add_blocker()` / `resolve_blocker()` — append/resolve blockers in the vec

**Gaps identified:**
1. No test for `read_state()` / `write_state()` (requires filesystem mocking or temp dirs)
2. No test for `transition()` end-to-end (reads/writes state.toml)
3. No test for `add_blocker()` / `resolve_blocker()`
4. `transition()` doesn't update `metrics.total_agent_sessions`
5. No validation that state.toml has required fields after deserialization
6. Missing transitions: `"discussed" → "discussed"` (re-discuss), `"complete" → "initialized"` (next phase reset)
7. `resolve_blocker()` resolves the first matching blocker but doesn't validate the step transition from "blocked"

#### mcp.rs — MCP Server
**What exists:**
- `KonductorMcp` struct with tool and prompt routers
- 9 prompts (k-init, k-plan, k-exec, k-verify, k-ship, k-next, k-status, k-discuss, k-map)
- 5 tools (state_get, state_transition, state_add_blocker, state_resolve_blocker, plans_list, status)
- `parse_plan_frontmatter()` — only extracts `wave` and `title` fields

**Gaps identified:**
1. `parse_plan_frontmatter()` only parses `wave` and `title` — missing: `phase`, `plan`, `depends_on`, `type`, `autonomous`, `requirements`, `files_modified`, `must_haves`
2. No tests for any MCP tool handler
3. No tests for `parse_plan_frontmatter()`
4. Tool handlers return raw strings — no structured error format
5. `plans_list` doesn't parse full frontmatter (only wave/title)
6. `status` tool constructs JSON manually — fragile

#### hook.rs — Hook Processor
**What exists:**
- `HookEvent` deserialization from stdin JSON
- `PostToolUse` handler: tracks file writes to `.konductor/.tracking/modified-files.log`
- `PreToolUse` handler: blocks destructive shell commands
- `is_destructive()` — pattern matching against hardcoded list
- 2 tests for destructive command detection

**Gaps identified:**
1. No test for `handle_post_tool_use()` (file tracking)
2. No test for `handle_pre_tool_use()` (command blocking)
3. No test for `HookEvent` deserialization
4. Destructive patterns are hardcoded — not configurable
5. No test for edge cases: empty input, malformed JSON, missing fields

#### main.rs — CLI Entry Point
**What exists:**
- Clap-based CLI with `mcp` and `hook` subcommands
- No `--version` flag

**Gaps identified:**
1. No `--version` flag (REQ-06)
2. No integration test for CLI argument parsing

### Config Management (REQ-05)
**Current state:** config.toml is referenced in skill definitions but the CLI never reads or validates it. There is no `config.rs` module. Skills tell the orchestrator to "read config.toml" but the MCP tools don't expose config values.

**What's needed:**
- A `config.rs` module with `Config` struct and `read_config()` function
- Validation of required fields and sensible defaults
- An MCP tool to expose config values (or include in `status` output)

## Libraries and Patterns

### Testing Strategy
- **Unit tests:** Use `#[cfg(test)]` modules in each source file
- **Filesystem tests:** Use `tempfile` crate for temp directories to test state read/write
- **MCP integration tests:** The rmcp crate doesn't easily support in-process testing of tool handlers. Test the underlying functions directly instead.
- **Snapshot testing:** Consider `insta` crate for JSON output validation

### Recommended Crates
- `tempfile` — temp directories for filesystem tests (already common in Rust ecosystem)
- `toml` — already in use, sufficient for frontmatter parsing
- No additional crates needed for core hardening

### Frontmatter Parsing
The current `parse_plan_frontmatter()` does manual line-by-line parsing. For full TOML frontmatter support (including nested tables like `[must_haves]`), extract the content between `+++` delimiters and parse with the `toml` crate directly into a typed struct.

## Risk Assessment

1. **Low risk:** Adding tests — no behavioral changes, purely additive
2. **Low risk:** Config module — new module, no changes to existing code
3. **Medium risk:** Frontmatter parser rewrite — changes `parse_plan_frontmatter()` return type, affects `plans_list` tool
4. **Low risk:** Error handling improvements — wrapping existing returns in structured format
5. **Low risk:** Version flag — trivial clap addition

## Recommendations

1. Start with tests for existing code (no behavioral changes)
2. Add config module as independent new code
3. Rewrite frontmatter parser with proper TOML parsing
4. Improve error handling in MCP tools
5. Add version flag last (trivial)
