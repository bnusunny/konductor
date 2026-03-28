# State Machine Reference

Konductor manages pipeline progress through a state machine stored in `.konductor/state.toml`. The orchestrator uses MCP tools (`state_transition`, `state_get`) to manage state — never editing the file directly.

## Steps

| Step | Description |
|------|-------------|
| `specced` | Project spec complete, requirements generated. Ready for design. |
| `discussed` | User preferences captured for the current phase. Ready for design. |
| `designed` | Phase architecture created. Ready for task planning. |
| `planned` | Execution plans created and validated. Ready for review and execution. |
| `executing` | Plans are being executed by subagents. |
| `executed` | All plans completed. Ready for verification. |
| `complete` | Phase verified and done. Ready to advance to next phase. |
| `shipped` | All phases complete, project finalized and tagged. |
| `blocked` | A blocker is preventing progress. Requires resolution. |

## Valid Transitions

```
specced ──→ discussed ──→ designed ──→ planned ──→ executing ──→ executed ──→ complete ──→ shipped
   │                         ↑
   └─────────────────────────┘
          (skip discuss)

blocked ──→ specced
blocked ──→ designed
blocked ──→ planned
blocked ──→ executed
```

| From | To | When |
|------|----|------|
| `specced` | `discussed` | User runs `/k-discuss` to set preferences |
| `specced` | `designed` | User runs `/k-design` (skipping discuss) |
| `discussed` | `designed` | Design completes after discuss |
| `designed` | `planned` | Task planning completes |
| `planned` | `executing` | Execution begins |
| `executing` | `executed` | All plans in all waves complete |
| `executed` | `complete` | Verification passes |
| `complete` | `shipped` | Project finalized |
| `blocked` | `specced` | Blocker resolved, restart from spec |
| `blocked` | `designed` | Blocker resolved, restart from designed |
| `blocked` | `planned` | Blocker resolved, restart from planned |
| `blocked` | `executed` | Blocker resolved, restart from executed |

!!! warning
    Invalid transitions (e.g., `specced` → `executed`) are rejected by the MCP tool with a structured error.

## Blockers

Blockers pause the pipeline when something goes wrong:

```toml
[[blockers]]
phase = "01"
step = "executing"
reason = "Executor subagent failed: compilation error in src/main.rs"
timestamp = "2026-03-21T18:00:00Z"
resolved = false
```

- **Adding:** `state_add_blocker` sets `status = "blocked"` and appends to the blockers list
- **Resolving:** `state_resolve_blocker` marks the blocker as resolved. When no active blockers remain, status returns to `"idle"`

## State File Structure

```toml
[project]
name = "my-project"
initialized = "2026-03-21T18:00:00Z"

[current]
phase = "01"          # Current phase number
step = "executing"    # Current pipeline step
status = "busy"       # idle, busy, or blocked
plan = ""             # Current plan being executed
wave = 1              # Current wave number

[progress]
phases_total = 3
phases_complete = 0
current_phase_plans = 5
current_phase_plans_complete = 2

[metrics]
last_activity = "2026-03-21T18:00:00Z"
total_agent_sessions = 3

[[blockers]]
phase = "01"
step = "executing"
reason = "Build failed"
timestamp = "2026-03-21T18:00:00Z"
resolved = false
```
