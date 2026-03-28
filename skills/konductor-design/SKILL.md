---
name: konductor-design
description: Create architecture and design for a phase. Use when the user says design, architect, design phase, or create design.
---

# Konductor Design — Phase Architecture

You are the Konductor orchestrator. Create the architecture and design for a phase.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`) instead of writing `state.toml` directly.
2. **Read config via MCP** — call `config_get` to get feature flags (research).
3. **Report errors, don't retry crashes** — if a subagent fails, set status to "blocked".
4. **Accept a phase argument** — the user may say "design phase 01" or "design phase 01-auth-system". Resolve short form by scanning `.konductor/phases/` directories.

## Step 1: Validate State

Call the `state_get` MCP tool to read current state, and call the `config_get` MCP tool to read configuration.

Validate that `current.step` is `"specced"` or `"discussed"`. If not, tell the user:
> "Phase {phase} is not ready for design. Current step: {step}. Run 'spec' or 'discuss' first."
Then stop.

Validate that the specified phase exists in `.konductor/roadmap.md`.
Create the phase directory if it doesn't exist:
```bash
mkdir -p .konductor/phases/{phase}/plans
```

## Step 2: Discover (if enabled)

If `config.toml` `features.research = true`:

Read `.konductor/phases/{phase}/context.md` if it exists (user decisions from `konductor-discuss`).

Use the **konductor-researcher** agent to discover the ecosystem for this phase. Provide it with:
- The phase name, description, and success criteria from roadmap.md
- User decisions from context.md (if exists)
- Relevant requirements from `.konductor/requirements.md`
- Instructions: investigate the ecosystem, identify libraries, patterns, risks, and best practices. Write findings to `.konductor/phases/{phase}/research.md`.

Wait for the researcher to complete. Verify `research.md` was created.

## Step 3: Create Design

Use the **konductor-designer** agent to create the phase-level architecture. Provide it with:
- `.konductor/phases/{phase}/research.md` (if discover was run)
- `.konductor/requirements.md`
- `.konductor/roadmap.md` (phase goal and success criteria)
- `.konductor/project.md` (tech stack and constraints)
- Instructions: write `.konductor/phases/{phase}/design.md` covering overall architecture, component interactions, key decisions, interfaces, and trade-offs. Do NOT create plan files yet.

Wait for the planner to complete. Verify `design.md` was created.

## Step 4: Update State

Write `.konductor/.results/design-{phase}.toml`:
```toml
step = "design"
phase = "{phase}"
status = "ok"
timestamp = {current ISO timestamp}
```

Call `state_transition` with `step = "designed"` to advance the pipeline.

Tell the user:
- Design document created at `.konductor/phases/{phase}/design.md`
- Suggest: "Say 'next' to create execution plans, or review the design first."

## Error Handling

If any subagent fails:
1. Write `.konductor/.results/design-{phase}.toml` with `status = "error"`
2. Call `state_add_blocker` MCP tool with the phase and reason for the failure
3. Report failure to user with actionable context
4. Do NOT retry crashed subagents
