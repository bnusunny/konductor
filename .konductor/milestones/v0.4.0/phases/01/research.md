# Phase 01 Research: MCP Protocol Integration Tests

## Approach

The rmcp crate provides two testing patterns:

1. **In-process duplex** — `tokio::io::duplex()` creates a bidirectional channel. Server and client run in the same process. Fast, no binary needed. Good for unit-level tool/prompt testing.

2. **Child process** — `TokioChildProcess::new(Command::new("konductor").arg("mcp"))` spawns the actual binary. Client connects over stdio. Tests the real binary end-to-end. Requires `cargo build` first.

**Recommendation:** Use approach #2 (child process) for integration tests. This tests the actual binary, including CLI argument parsing, module wiring, and MCP server initialization. It's what we need for CI confidence.

## Key rmcp Client APIs

```rust
// Spawn server as child process
let transport = TokioChildProcess::new(
    tokio::process::Command::new("path/to/konductor").configure(|cmd| { cmd.arg("mcp"); })
)?;

// Connect client
let client = ().serve(transport).await?;

// List and call tools
let tools = client.list_all_tools().await?;
let result = client.call_tool(CallToolRequestParams { name: "state_get".into(), arguments: None }).await?;

// List and get prompts
let prompts = client.list_all_prompts().await?;
let result = client.get_prompt(GetPromptRequestParams { name: "k-status".into(), arguments: None }).await?;

// Cleanup
client.cancel().await?;
```

## Dependencies Needed

```toml
[dev-dependencies]
tempfile = "3"          # already present
rmcp = { version = "1.2", features = ["client", "transport-child-process"] }
anyhow = "1"
```

## Test Structure

Integration tests go in `konductor-cli/tests/` as separate test files (Rust integration test convention). Each test:
1. Creates a temp directory
2. Sets up `.konductor/state.toml` and `.konductor/config.toml` in it
3. Spawns `konductor mcp` with `current_dir` set to the temp dir
4. Connects rmcp client
5. Calls tools/prompts and validates responses
6. Cleans up

## State Isolation

Each test creates its own temp directory with fresh `.konductor/` state. The `konductor mcp` process runs with `current_dir` set to the temp dir, so `STATE_FILE = ".konductor/state.toml"` resolves relative to the temp dir.

## What to Test

### Tools (7)
- `state_get` — returns state JSON, errors when no state.toml
- `state_transition` — valid transitions succeed, invalid rejected
- `state_add_blocker` — adds blocker, sets status to blocked
- `state_resolve_blocker` — resolves blocker, clears blocked status
- `plans_list` — lists plans with frontmatter data, handles missing dir
- `status` — returns structured status report
- `config_get` — returns config with defaults

### Prompts (9)
- `k-init`, `k-exec`, `k-verify`, `k-ship`, `k-next`, `k-status`, `k-map` — no args, return correct message text
- `k-plan`, `k-discuss` — take phase arg, return message with phase number
