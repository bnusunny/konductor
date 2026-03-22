---
name: konductor-plan
description: Plan a phase for execution. Research the ecosystem, create execution plans, and validate them. Use when the user says plan phase, plan, research and plan, or prepare phase.
---

# Konductor Plan — Phase Planning Pipeline

You are the Konductor orchestrator. Plan a phase by researching the ecosystem, creating execution plans, and validating them.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`) instead of writing `state.toml` directly. Subagents write their own output files.
2. **Read `config.toml` first** — respect feature flags (research, plan_checker).
3. **Report errors, don't retry crashes** — if a subagent fails, set status to "blocked".
4. **Accept a phase argument** — the user may say "plan phase 01" or "plan phase 01-auth-system". Resolve short form by scanning `.konductor/phases/` directories.

## Step 1: Validate State

Call the `state_get` MCP tool to read current state, and read `.konductor/config.toml`.
Validate that the specified phase exists in `.konductor/roadmap.md`.
Create the phase directory if it doesn't exist:
```bash
mkdir -p .konductor/phases/{phase}/plans
```

## Step 2: Research (if enabled)

If `config.toml` `features.research = true`:

Read `.konductor/phases/{phase}/context.md` if it exists (user decisions from `konductor-discuss`).

Use the **konductor-researcher** agent to research this phase. Provide it with:
- The phase name, description, and success criteria from roadmap.md
- User decisions from context.md (if exists)
- Relevant requirements from `.konductor/requirements.md`
- Instructions: investigate the ecosystem, identify libraries, patterns, risks, and best practices. Write findings to `.konductor/phases/{phase}/research.md`.

Wait for the researcher to complete. Verify `research.md` was created.

## Step 3: Create Plans

Use the **konductor-planner** agent to decompose the phase into execution plans. Provide it with:
- `.konductor/phases/{phase}/research.md` (if research was run)
- `.konductor/requirements.md`
- `.konductor/roadmap.md` (phase goal and success criteria)
- The planning guide: see `references/planning-guide.md`
- Instructions: create plan files in `.konductor/phases/{phase}/plans/` using TOML frontmatter format

Wait for the planner to complete. Verify at least one plan file was created.

## Step 4: Validate Plans (if enabled)

If `config.toml` `features.plan_checker = true`:

Use the **konductor-plan-checker** agent to validate the plans. Provide it with:
- All plan files in `.konductor/phases/{phase}/plans/`
- `.konductor/requirements.md`
- Instructions: check coverage (every requirement addressed), sizing (2-5 tasks per plan), dependencies (valid wave ordering), completeness (no gaps). Fix issues in-place.

If the checker reports issues that it could not fix:
- Spawn a new **konductor-planner** agent with the checker's feedback as additional context
- Re-run the **konductor-plan-checker**
- Maximum 3 iterations. If still unresolved, report to user.

## Step 5: Update State

Write `.konductor/.results/plan-{phase}.toml`:
```toml
step = "plan"
phase = "{phase}"
status = "ok"
timestamp = {current ISO timestamp}
plan_count = {number of plan files created}
```

Update state using MCP tools:
- Call `state_transition` with `step = "planned"` to advance the pipeline
- The tool automatically updates status and timestamps

Tell the user:
- How many plans were created
- How many waves
- Suggest: "Say 'next' to execute, or review the plans in `.konductor/phases/{phase}/plans/`"

## Error Handling

If any subagent fails:
1. Write `.konductor/.results/plan-{phase}.toml` with `status = "error"`
2. Call `state_add_blocker` MCP tool with the phase and reason for the failure
3. Report failure to user with actionable context
4. Do NOT retry crashed subagents
