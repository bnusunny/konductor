# Commands

## Prompt Commands

All commands are available as Tab-completable prompts. Type `@k-` then press Tab.

### `@k-spec` — Define Project Spec

Discovers project goals, generates spec documents (`project.md`, `requirements.md`, `roadmap.md`), and sets up the pipeline.

```
> @k-spec
```

Creates the `.konductor/` directory with initial state. `@k-init` works as an alias.

After all phases are shipped, run `@k-spec` again to add new phases to the existing project or re-spec from scratch.

### `@k-design` — Create Phase Architecture

Creates the phase-level architecture with component interactions, key decisions, and interfaces. Runs ecosystem discovery if enabled.

```
> @k-design 01
```

Takes a phase number as argument. Creates `design.md` in `.konductor/phases/{phase}/`.

### `@k-plan` — Create Execution Plans

Breaks the phase design into execution plans with tasks, acceptance criteria, and wave ordering. Validates plans.

```
> @k-plan 01
```

Takes a phase number as argument. Creates plan files in `.konductor/phases/{phase}/plans/`.

### `@k-review` — Review Design and Plans

Reviews the architecture and plans before execution. Evaluates soundness, feasibility, consistency, and requirement coverage.

```
> @k-review 01
```

Takes a phase number as argument. Creates `review.md` with verdict and asks for user approval.

### `@k-exec` — Execute Current Phase

Dispatches fresh executor per task following TDD workflow, then runs two-stage review (spec compliance + code quality).

```
> @k-exec
```

Executes plans wave by wave with per-task dispatch.

### `@k-verify` — Verify Current Phase

Validates that phase goals were achieved using the 3-level verification framework (Exists, Substantive, Wired).

```
> @k-verify
```

Writes verification report to `.konductor/phases/{phase}/verification.md`.

### `@k-ship` — Ship Project

Verifies all phases are complete, runs cross-phase integration checks, creates git tag, and archives project state.

```
> @k-ship
```

### `@k-next` — Advance Pipeline

Determines what needs to happen next based on current state and executes it automatically.

```
> @k-next
```

This is the most common command — it reads the state and runs the appropriate pipeline step.

### `@k-status` — Show Status

Displays phase progress, current position, recent activity, blockers, and next steps.

```
> @k-status
```

### `@k-discuss` — Discuss Phase

Sets context for a phase before design — approach preferences, library choices, architectural trade-offs.

```
> @k-discuss 01
```

Takes a phase number as argument. Writes decisions to `.konductor/phases/{phase}/context.md`.

### `@k-map` — Map Codebase

Analyzes existing code structure, tech stack, architecture patterns, testing setup, and integrations.

```
> @k-map
```

Used for brownfield projects during spec.

## MCP Tools

These tools are used by the orchestrator agent for deterministic state management. They're available via the built-in MCP server.

| Tool | Description |
|------|-------------|
| `state_get` | Read current pipeline state from `state.toml` |
| `state_transition` | Advance to a new step (validates the transition is allowed) |
| `state_add_blocker` | Add a blocker to the current phase |
| `state_resolve_blocker` | Resolve a blocker for a phase |
| `state_advance_phase` | Advance a shipped project to a new phase |
| `plans_list` | List plans for a phase with wave, type, dependencies, and completion status |
| `status` | Get a structured status report with progress, blockers, and config |
| `config_get` | Read current configuration from `config.toml` (with defaults applied) |
| `config_init` | Create `config.toml` with defaults (idempotent) |

### Error Responses

All MCP tools return structured JSON errors:

```json
{
  "error": true,
  "code": "INVALID_TRANSITION",
  "message": "Invalid transition: 'specced' → 'executed'"
}
```

Error codes: `STATE_NOT_FOUND`, `INVALID_TRANSITION`, `NO_PLANS`, `IO_ERROR`, `INVALID_CONFIG`, `SERIALIZE_ERROR`.
