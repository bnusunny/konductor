# Hook System

Konductor includes a hook system that tracks file modifications and blocks destructive commands. The same `konductor` binary handles hooks via the `hook` subcommand.

## Configuration

Hooks are configured in `.kiro/hooks/konductor-hooks.json`:

```json
{
  "hooks": [
    {
      "event": "PostToolUse",
      "matcher": "write",
      "command": "~/.kiro/bin/konductor hook",
      "timeout_ms": 2000
    },
    {
      "event": "PreToolUse",
      "matcher": "shell",
      "command": "~/.kiro/bin/konductor hook",
      "timeout_ms": 1000
    }
  ]
}
```

## File Tracking (PostToolUse)

Every time a file is written by an agent, the hook appends the file path to a tracking log:

```
.konductor/.tracking/modified-files.log
```

This log records all files modified during execution, useful for:

- Reviewing what changed during a phase
- Debugging unexpected modifications
- Generating commit scopes

The tracking directory is created automatically if it doesn't exist.

## Destructive Command Blocking (PreToolUse)

Before any shell command executes, the hook checks it against a list of destructive patterns. If a match is found, the command is **blocked** and the hook exits with code 2.

### Blocked Patterns

| Pattern | Reason |
|---------|--------|
| `rm -rf /` | Filesystem destruction |
| `rm -rf /*` | Filesystem destruction |
| `rm -rf ~` | Home directory destruction |
| `drop table` | Database destruction |
| `drop database` | Database destruction |
| `truncate table` | Data loss |
| `git push --force origin main` | Force push to main |
| `git push --force origin master` | Force push to master |
| `git push -f origin main` | Force push to main |
| `git push -f origin master` | Force push to master |
| `git reset --hard origin` | History destruction |
| `mkfs.` | Disk formatting |
| `dd if=/dev/zero` | Disk overwrite |
| `> /dev/sda` | Disk overwrite |

!!! note
    Pattern matching is case-insensitive. `DROP TABLE users` and `drop table users` are both blocked.

### What's Allowed

These commands are **not** blocked:

- `rm src/temp.rs` — removing specific files is fine
- `git push origin feature-branch` — pushing to non-main branches is fine
- `git push --force origin feature-branch` — force push to feature branches is fine
- `git reset --hard HEAD~1` — local resets are fine

## How It Works

```
Agent wants to run shell command
        │
        ▼
  PreToolUse hook fires
        │
        ▼
  konductor hook reads event from stdin
        │
        ▼
  Is it a shell command?
   ├── No → exit 0 (allow)
   └── Yes → check against destructive patterns
        ├── Not destructive → exit 0 (allow)
        └── Destructive → stderr message + exit 2 (block)
```

## Checking the Tracking Log

To see what files were modified during execution:

```bash
cat .konductor/.tracking/modified-files.log
```

To see unique files:

```bash
sort -u .konductor/.tracking/modified-files.log
```
