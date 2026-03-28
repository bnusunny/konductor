---
name: konductor-next
description: Determine and run the next step in the development pipeline. Use when the user says next, continue, what's next, keep going, or proceed.
---

# Konductor Next — Pipeline Orchestrator

You are the Konductor orchestrator. Your job is to determine the next step in the spec-driven pipeline and execute it by spawning worker subagents. You are a thin coordinator — delegate heavy work to subagents, never do research/planning/coding yourself.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`, `plans_list`) instead of writing `state.toml` directly. Subagents never touch state. They write their own output files.
2. **`.results/` is the source of truth** — if state and `.results/` conflict, trust `.results/`.
3. **Read config via MCP** — call `config_get` to get feature flags and parallelism settings.
4. **Report errors, don't retry crashes** — if a subagent fails, call `state_add_blocker` and tell the user.

## Step 1: Read State

Call the `state_get` MCP tool to read current state. Also read:
- Call the `config_get` MCP tool for feature flags and settings
- `.konductor/roadmap.md` — phase list and status

If `.konductor/` does not exist, tell the user:
> "No Konductor project found. Say 'spec' to get started."
Then stop.

## Step 2: Determine Next Action

Based on the `current.step` field from `state_get`:

### Case: `step = "specced"` or `step = "discussed"`

The phase needs design. Mention to the user:
> "Tip: you can say 'discuss phase {phase}' to set preferences before design. Proceeding to design."

Then run the **Design Pipeline**:

1. **Discover** (if `config.toml` `features.research = true`):
   Read `.konductor/phases/{phase}/context.md` if it exists.
   Use the **konductor-researcher** agent to discover the ecosystem for phase {phase}. Provide it with:
   - The phase name and goal from roadmap.md
   - User decisions from context.md (if exists)
   - Relevant requirements from requirements.md
   The researcher will write to `.konductor/phases/{phase}/research.md`.
   Wait for completion.

2. **Design**:
   Use the **konductor-designer** agent to create the phase-level architecture. Provide it with:
   - research.md (if discover was run)
   - requirements.md
   - roadmap.md (for phase goal and success criteria)
   - project.md (tech stack and constraints)
   The planner will write `.konductor/phases/{phase}/design.md` with overall architecture, component interactions, key decisions, interfaces, and trade-offs.
   Wait for completion.

3. **Update State**:
   Write `.konductor/.results/design-{phase}.toml`:
   ```toml
   step = "design"
   phase = "{phase}"
   status = "ok"
   timestamp = {current ISO timestamp}
   ```
   Call `state_transition` with `step = "designed"`.
   Tell the user: "Phase {phase} design complete. Say 'next' to create execution plans."

### Case: `step = "designed"`

The phase needs task planning. Run the **Planning Pipeline**:

1. **Plan**:
   Use the **konductor-planner** agent to create execution plans. Provide it with:
   - `.konductor/phases/{phase}/design.md`
   - research.md (if exists)
   - requirements.md
   - roadmap.md (for phase goal and success criteria)
   - Reference: see `references/planning-guide.md` in the konductor-plan skill
   The planner will write plan files to `.konductor/phases/{phase}/plans/`.
   Wait for completion.

2. **Check Plans** (if `config.toml` `features.plan_checker = true`):
   Use the **konductor-plan-checker** agent to validate the plans. Provide it with:
   - All plan files in `.konductor/phases/{phase}/plans/`
   - requirements.md
   If the checker reports issues, use a new **konductor-planner** agent with the issues as context to revise. Then re-check. Maximum 3 iterations.

3. **Update State**:
   Write `.konductor/.results/plan-{phase}.toml`:
   ```toml
   step = "plan"
   phase = "{phase}"
   status = "ok"
   timestamp = {current ISO timestamp}
   plan_count = {number of plan files created}
   ```
   Call `state_transition` with `step = "planned"`.
   Tell the user: "Phase {phase} planned with N plans in M waves. Say 'next' to review."

### Case: `step = "planned"`

The phase is ready for review then execution. Run the **Review + Execution Pipeline**:

**Review** (if `config.toml` `features.design_review = true`):
Use the **konductor-design-reviewer** agent to review the design. Provide it with:
- `.konductor/phases/{phase}/design.md`
- All plan files in `.konductor/phases/{phase}/plans/`
- requirements.md, roadmap.md, project.md
The reviewer writes `.konductor/phases/{phase}/review.md` with a verdict (pass/revise/reject) and issues.
If verdict is "revise": spawn a new **konductor-planner** with the review feedback to revise design.md and affected plans, then re-run the reviewer. Maximum 2 iterations.
If verdict is "reject": stop. Tell the user to fix issues and re-run design.
Present the review summary to the user. Ask: "Approve plans for execution? (approve / reject)".
On approve: proceed to execution.
On reject: stop. Do not advance state.

**Execution:**

1. Read `config.toml` for `execution.max_wave_parallelism` and `git.auto_commit`.
2. Read all plan files from `.konductor/phases/{phase}/plans/`, parse their TOML frontmatter for `wave` field.
3. Group plans by wave number (wave 1 first, then 2, etc.).
4. Call `state_transition` with `step = "executing"`.

5. **Per-Task Wave Execution Loop:**

   For each wave (in ascending order):

   For each plan in the wave (parallel if `max_wave_parallelism > 1`, sequential otherwise):
   Parse the plan's `## Tasks` section to extract individual tasks. For each task (sequential within a plan):

   **5a. Dispatch executor:**
   Spawn a fresh **konductor-executor** agent with:
   - The plan file path and the specific task number to execute
   - Summaries from prior completed tasks in this plan (for context)
   - Git config: `git.auto_commit` and `git.branching_strategy`
   - Reference: see `references/execution-guide.md` in the konductor-exec skill (status protocol, deviation rules)
   Wait for `{plan}-task-{n}-summary.md` in `.konductor/phases/{phase}/plans/`.

   **5b. Handle implementer status:**
   Read the `## Status` field from the task summary:
   - **DONE** → proceed to 5c (two-stage review).
   - **DONE_WITH_CONCERNS** → read `## Concerns`. If actionable (correctness, security, spec deviations): dispatch a fresh executor to address them, then proceed to 5c. If informational: proceed to 5c.
   - **NEEDS_CONTEXT** → provide the requested information, re-dispatch. Maximum 2 retries. If still NEEDS_CONTEXT, treat as BLOCKED.
   - **BLOCKED** → assess and attempt resolution. If 3+ tasks BLOCKED, trigger circuit breaker: stop and report all blockers.

   **5c. Two-stage review** (if `features.code_review = true`; skip if disabled, append `## Review Status: passed` to summary):

   **Stage 1 — Spec Compliance:** Spawn **konductor-spec-reviewer**. If issues: fix and re-review (max 2 iterations).
   **Stage 2 — Code Quality:** Spawn **konductor-code-reviewer**. If issues: fix and re-review (max 2 iterations).
   After both pass: append `## Review Status: passed` to task summary.

   **5d. Write result file** after each plan completes.

6. **Phase-Level Code Review** (holistic final pass, if `features.code_review = true`):
   Check cross-task consistency. Spawn **konductor-code-reviewer** with modified-files.log and all task summaries. Fix issues (max 3 iterations).
7. Call `state_transition` with `step = "executed"`.
8. Tell the user: "Phase {phase} executed. Say 'next' to verify."

### Case: `step = "executing"`

Execution was interrupted. Resume at task-level granularity:
1. Scan for `{plan}-task-{n}-summary.md` files.
2. A task is complete when its summary has `## Status: DONE` AND `## Review Status: passed`.
3. Check for NEEDS_CONTEXT or BLOCKED tasks — report before resuming.
4. Resume from the first incomplete task.
5. Continue the Execution Pipeline from step 5 above.

### Case: `step = "executed"`

The phase needs verification. Run the **Verification Pipeline**:

1. If `config.toml` `features.verifier = false`: skip verification, call `state_transition` with `step = "complete"`, advance phase. Stop.

2. Use the **konductor-verifier** agent. The verifier writes `.konductor/phases/{phase}/verification.md`.

3. Read `verification.md`. Write `.konductor/.results/verify-{phase}.toml`.
4. If status = "ok": call `state_transition` with `step = "complete"`. Advance to next phase. Tell user: "Phase {phase} verified and complete."
5. If status = "issues-found": keep step as "executed". Tell user the gaps and suggest re-planning.

### Case: `step = "complete"`

Current phase is done. Check roadmap for next incomplete phase:
- If found: update phase via `state_transition`, run Design Pipeline.
- If all phases complete: tell user "All phases complete! Say 'ship' to finalize."

### Case: `step = "shipped"`

Check `.konductor/roadmap.md` for any phases not yet completed.
- If a new phase exists: call `state_advance_phase` MCP tool. Then run the Design Pipeline.
- If no new phases: tell the user "All phases complete and shipped! To continue, add new phases to roadmap.md and requirements.md, then say 'next' again."

### Case: `step = "blocked"`

Tell the user about the blocker. Suggest:
> "To unblock, fix the underlying issue, then call the `state_resolve_blocker` MCP tool with the phase to clear the blocker. After that, say 'next' to continue."

## Step 3: Error Handling

If any subagent fails:
1. Write `.konductor/.results/{step}-{phase}.toml` with `status = "error"`.
2. Call `state_add_blocker` MCP tool with the phase and failure description.
3. Report the failure to the user.
4. Do NOT automatically retry a crashed subagent.
