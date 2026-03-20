# Commands

| Command | Prompt Shortcut | Description |
|---------|----------------|-------------|
| `initialize` | `@k-init` | Start a new project, discover requirements, generate spec docs |
| `next` | `@k-next` | Advance to the next pipeline phase automatically |
| `plan` | `@k-plan` | Generate phase plans with tasks and acceptance criteria |
| `exec` / `execute` | `@k-exec` | Execute tasks for current phase (TDD workflow) |
| `verify` | `@k-verify` | Run tests and validate acceptance criteria |
| `status` | `@k-status` | Show current phase, progress, and next steps |
| `ship` | `@k-ship` | Commit work, create release notes, open PR |
| `discuss` | `@k-discuss` | Ask questions about the project, codebase, or pipeline |
| `map-codebase` | `@k-map` | Analyze existing code structure, patterns, and conventions |

!!! tip
    Type `@k-` then press Tab to autocomplete any Konductor command. Prompts with arguments (like `@k-plan`) will prompt you for the phase number.

## MCP Tools

These tools are available to the orchestrator agent via the built-in MCP server for deterministic state management:

| Tool | Description |
|------|-------------|
| `state_get` | Read current pipeline state from `state.toml` |
| `state_transition` | Advance to a new step (with validation) |
| `state_add_blocker` | Add a blocker to the current phase |
| `state_resolve_blocker` | Resolve a blocker for a phase |
| `plans_list` | List plans for a phase with wave and completion status |
| `status` | Get a structured status report of the entire project |
