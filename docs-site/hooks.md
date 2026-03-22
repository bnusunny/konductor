# Hook System

Konductor includes a steering hook system inspired by [Strands-style just-in-time guidance](https://strandsagents.com/blog/steering-accuracy-beats-prompts-workflows/). Instead of front-loading all instructions into prompts, hooks observe what the agent is doing and provide targeted feedback at decision points. The same `konductor` binary handles hooks via the `hook` subcommand.

## Configuration

Hooks are configured inline in each agent's JSON config (e.g. `.kiro/agents/konductor.json`):

```json
{
  "hooks": {
    "preToolUse": [
      {
        "matcher": "*",
        "command": "konductor hook",
        "timeout_ms": 1000
      }
    ],
    "postToolUse": [
      {
        "matcher": "*",
        "command": "konductor hook",
        "timeout_ms": 2000
      }
    ]
  }
}
```

Both hooks use the `"*"` matcher to intercept all tool calls (built-in and MCP).

## Tool Call Ledger (PostToolUse)

Every tool call is recorded to an append-only JSONL ledger:

```
.konductor/.tracking/ledger.jsonl
```

Each line is a JSON object with:

- `tool_name` — the tool that was called
- `input_summary` — compact summary (file path for read/write, command for shell)
- `success` — whether the tool succeeded
- `output_summary` — last few lines of shell output (shell only)

The ledger uses `O_APPEND` writes which are atomic on POSIX, making it safe for concurrent hook processes (e.g. parallel subagents).

## File Tracking (PostToolUse)

In addition to the ledger, file writes are tracked in:

```
.konductor/.tracking/modified-files.log
```

## Steering Hooks (PreToolUse)

PreToolUse hooks can block tool execution (exit code 2) and return feedback to the LLM via STDERR. Konductor uses this for four types of steering:

### 1. Destructive Command Blocking

Shell commands are checked against destructive patterns (case-insensitive):

| Pattern | Reason |
|---------|--------|
| `rm -rf /`, `rm -rf ~` | Filesystem destruction |
| `drop table`, `drop database` | Database destruction |
| `git push --force origin main` | Force push to main/master |
| `git reset --hard origin` | History destruction |
| `mkfs.`, `dd if=/dev/zero` | Disk destruction |

### 2. State Protection

Direct writes to `.konductor/state.toml` (via `write` tool or shell commands like `sed`, `echo >`) are blocked. Agents must use MCP tools (`state_get`, `state_transition`, etc.) instead.

### 3. Stuck Detection

Tracks consecutive writes to the same file. After 5 consecutive writes without any shell command in between, the write is blocked with guidance:

> "You may be stuck in a loop. Stop and reconsider: read the error output, try a different approach, or ask the user."

The counter resets when the agent runs a shell command or writes to a different file.

### 4. Workflow Validation

Reads the tool call ledger to enforce pipeline rules:

- **Plan file gating**: Writing to `.konductor/phases/*/plans/*.md` is blocked unless the ledger shows a prior `state_get`, `state_transition`, or `state_init` call.
- **Test-before-complete**: Calling `state_transition` with `step: "complete"` is blocked unless the ledger shows a prior shell command matching a test/lint pattern.

Test patterns are configurable in `.konductor/config.toml`:

```toml
[hooks]
test_patterns = ["test", "pytest", "cargo test", "npm test", "go test", "mvn test", "gradle test", "jest", "vitest", "ruff", "lint", "eslint", "clippy", "golangci-lint"]
```

## How It Works

```
Agent calls any tool
        │
        ▼
  PreToolUse hook fires
        │
        ├── Destructive command? → BLOCK + feedback
        ├── Direct state.toml write? → BLOCK + feedback
        ├── Stuck (5+ writes to same file)? → BLOCK + feedback
        ├── Workflow violation? → BLOCK + feedback
        └── All checks pass → exit 0 (allow)

        ... tool executes ...

  PostToolUse hook fires
        │
        ├── Record tool call to ledger.jsonl
        └── If write tool → append path to modified-files.log
```
