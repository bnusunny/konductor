---
name: konductor-status
description: Display project status and progress. Use when the user says status, progress, where are we, show state, what's happening, or current status.
---

# Konductor Status — Project Status Display

You are the Konductor orchestrator. Display the current status and progress of the project.

## Step 1: Read Project State

Read these files:
- `.konductor/state.toml` — current phase, step, progress counters
- `.konductor/roadmap.md` — all phases with goals
- Scan `.konductor/.results/` directory for result files

If `.konductor/state.toml` doesn't exist, tell the user:
> "No Konductor project found. Run 'init' to initialize a project."

Then stop.

## Step 2: Parse Phase Status

From `roadmap.md`, extract all phase numbers and names.

From `state.toml`, read:
- `[current].phase` — active phase
- `[current].step` — current step within phase (initialized, discussed, planned, executing, executed, verified, complete)
- `[progress].phases_total` — total phases
- `[progress].phases_complete` — completed phases

For each phase in the roadmap:
1. If phase number < current phase number: status = "complete" (✓)
2. If phase number = current phase number: status = "active" (○)
3. If phase number > current phase number: status = "pending" (·)

## Step 3: Read Recent Activity

From `.konductor/.results/`, scan result files (most recent 5):
- Look for files like `execute-{phase}-plan-{n}.toml`, `verify-{phase}.toml`, etc.
- Extract timestamps and actions
- Identify blockers (any result files with `status = "error"`)

From `state.toml`, read:
- `[metrics].last_activity` — timestamp of last activity
- `[metrics].total_agent_sessions` — number of agent invocations

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
- `initialized` → "Say 'next' to start planning, or 'discuss phase {n}' to set preferences first."
- `discussed` → "Say 'next' to begin planning phase {n}."
- `planned` → "Say 'exec' to execute the plans."
- `executing` → "Execution in progress. Wait for completion or check logs."
- `executed` → "Say 'next' to verify the phase."
- `verified` → "Say 'next' to mark phase complete and advance."
- `complete` → "Phase {n} complete. Say 'next' to move to phase {n+1}."

## Error Handling

**Missing state files:**
If `state.toml` or `roadmap.md` is missing or malformed:
1. Report which file is missing or corrupted
2. Suggest running `init` to reinitialize (warn about overwriting)

**Empty results directory:**
If `.konductor/.results/` is empty, that's normal for newly initialized projects — simply report no activity yet.
