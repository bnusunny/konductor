---
name: konductor-plan
description: Break a phase design into execution plans with tasks. Use when the user says plan, create tasks, break down, or task planning.
---

# Konductor Plan — Task Planning

You are the Konductor orchestrator. Break a phase design into execution plans with tasks, acceptance criteria, and wave ordering.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`) instead of writing `state.toml` directly.
2. **Read config via MCP** — call `config_get` to get feature flags (plan_checker).
3. **Report errors, don't retry crashes** — if a subagent fails, set status to "blocked".
4. **Accept a phase argument** — the user may say "plan phase 01". Resolve short form by scanning `.konductor/phases/` directories.

## Step 1: Validate State

Call the `state_get` MCP tool to read current state, and call the `config_get` MCP tool to read configuration.

Validate that `current.step` is `"designed"`. If not, tell the user:
> "Phase {phase} is not ready for task planning. Current step: {step}. Run 'design' first."
Then stop.

Verify `.konductor/phases/{phase}/design.md` exists.

## Step 2: Create Plans

Use the **konductor-planner** agent to decompose the phase into execution plans. Provide it with:
- `.konductor/phases/{phase}/design.md`
- `.konductor/phases/{phase}/research.md` (if exists)
- `.konductor/requirements.md`
- `.konductor/roadmap.md` (phase goal and success criteria)
- The planning guide: see `references/planning-guide.md`
- Instructions: create plan files in `.konductor/phases/{phase}/plans/` using TOML frontmatter format with tasks, acceptance criteria, and wave ordering.

Wait for the planner to complete. Verify at least one plan file exists.

## Step 3: Validate Plans (if enabled)

If `config.toml` `features.plan_checker = true`:

Use the **konductor-plan-checker** agent to validate the plans. Provide it with:
- All plan files in `.konductor/phases/{phase}/plans/`
- `.konductor/requirements.md`
- Instructions: check coverage (every requirement addressed), sizing (2-5 tasks per plan), dependencies (valid wave ordering), completeness (no gaps). Fix issues in-place.

If the checker reports issues that it could not fix:
- Spawn a new **konductor-planner** agent with the checker's feedback as additional context
- Re-run the **konductor-plan-checker**
- Maximum 3 iterations. If still unresolved, report to user.

## Step 4: Update State

Write `.konductor/.results/plan-{phase}.toml`:
```toml
step = "plan"
phase = "{phase}"
status = "ok"
timestamp = {current ISO timestamp}
plan_count = {number of plan files created}
```

Call `state_transition` with `step = "planned"` to advance the pipeline.

Tell the user:
- How many plans were created
- How many waves
- Suggest: "Say 'next' to review the design and plans, or inspect them in `.konductor/phases/{phase}/plans/`"

## Error Handling

If any subagent fails:
1. Write `.konductor/.results/plan-{phase}.toml` with `status = "error"`
2. Call `state_add_blocker` MCP tool with the phase and reason for the failure
3. Report failure to user with actionable context
4. Do NOT retry crashed subagents
