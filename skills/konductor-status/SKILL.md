---
name: konductor-status
description: Display project status and progress. Use when the user says status, progress, where are we, show state, what's happening, or current status.
---

# Konductor Status — Project Status Display

You are the Konductor orchestrator. Display the current status and progress of the project.

## Step 1: Get Project Status

Call the `status` MCP tool. This returns a structured JSON report with:
- `project` — project name
- `current_phase`, `current_step`, `status`
- `phases_total`, `phases_complete`
- `current_phase_plans`, `current_phase_plans_complete`
- `active_blockers` count and `blockers` list
- `last_activity`, `total_agent_sessions`
- `next_suggestion`

If the tool returns a `STATE_NOT_FOUND` error, tell the user:
> "No Konductor project found. Run 'spec' to initialize a project."

Then stop.

## Step 2: Parse Phase Status

Read `.konductor/roadmap.md` to extract all phase numbers and names.

Using the `current_phase` from the status report:
1. If phase number < current phase number: status = "complete" (✓)
2. If phase number = current phase number: status = "active" (○)
3. If phase number > current phase number: status = "pending" (·)

## Step 3: Read Recent Activity

From `.konductor/.results/`, scan result files (most recent 5):
- Look for files like `execute-{phase}-plan-{n}.toml`, `verify-{phase}.toml`, etc.
- Extract timestamps and actions

Use the `last_activity` and `total_agent_sessions` from the status report.

## Step 4: Check for Decisions

For the current phase, check if `.konductor/phases/{phase}/context.md` exists.
- If it exists, read the decisions section
- Extract recent decisions (last 2-3)

## Step 5: Display Formatted Output

Present a structured status report:

```
# Konductor Status

## Phases
{for each phase}
{status_icon} Phase {number}: {name}
{/for}

## Current Position
Phase: {current_phase_number} — {current_phase_name}
Step: {current_step}

## Progress
{phases_complete} / {phases_total} phases complete ({percentage}%)
{if active phase has plans: current_phase_plans_complete / current_phase_plans plans complete}

## Metrics
Total agent sessions: {total_agent_sessions}
Last activity: {last_activity_relative} ({last_activity_absolute})

{if blockers exist}
## Blockers
{list of blockers with phase and plan numbers}
{/if}

{if recent decisions exist}
## Recent Decisions
{list of recent decisions from context.md}
{/if}

## Next Steps
{suggest what the user should do next based on current step}
```

**Status icons:**
- ✓ = complete
- ○ = active/in progress
- · = pending

**Next step suggestions by current step:**
- `specced` → "Say 'next' to start design, or 'discuss phase {n}' to set preferences first."
- `discussed` → "Say 'next' to begin designing phase {n}."
- `designed` → "Say 'next' to create execution plans."
- `planned` → "Say 'exec' to execute the plans."
- `executing` → "Execution in progress. Wait for completion or check logs."
- `executed` → "Say 'next' to verify the phase."
- `complete` → "Phase {n} complete. Say 'next' to move to phase {n+1}."
- `shipped` → "All phases shipped. Add new phases to roadmap.md or say 'spec' to start a new project."
- `blocked` → "Resolve the blocker with `state_resolve_blocker`, then say 'next'."

## Error Handling

**Missing state files:**
If the `status` tool returns an error:
1. Report the error
2. Suggest running `spec` to re-spec (warn about overwriting)

**Empty results directory:**
If `.konductor/.results/` is empty, that's normal for newly initialized projects — simply report no activity yet.
