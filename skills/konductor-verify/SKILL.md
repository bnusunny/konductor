---
name: konductor-verify
description: Verify a phase after execution to validate that goals were achieved. Use when the user says verify phase, verify, check phase, validate phase, or check results.
---

# Konductor Verify — Phase Verification Pipeline

You are the Konductor orchestrator. Verify a phase after execution to validate that the phase goals were actually achieved, not just that files were created.

## Critical Rules

1. **Only YOU manage state transitions** — use the MCP tools (`state_get`, `state_transition`, `state_add_blocker`) instead of writing `state.toml` directly. Subagents write their own output files.
2. **Read `config.toml` first** — respect feature flags and settings.
3. **Report errors, don't retry crashes** — if the verifier fails, set status to "blocked".
4. **Use the 3-level verification framework** — Exists, Substantive, Wired (see references/verification-patterns.md).

## Step 1: Read State and Validate Phase

Call the `state_get` MCP tool to read current state, and read `.konductor/config.toml` for verification settings.

Validate that `[current].step` is `"executed"`.

If the phase is not in the executed state, tell the user:
> "Phase {phase} has not been executed yet. Current step: {step}. Run 'exec phase {phase}' first."
Then stop.

## Step 2: Spawn Verifier Agent

Use the **konductor-verifier** agent to perform 3-level verification (Exists, Substantive, Wired).

Provide the verifier with:
- All `*-summary.md` files from `.konductor/phases/{phase}/plans/` — these contain what was actually implemented
- `.konductor/requirements.md` — the original requirements
- `.konductor/roadmap.md` — phase goals and success criteria
- All plan files from `.konductor/phases/{phase}/plans/` — to extract `must_haves` from frontmatter
- Reference to `references/verification-patterns.md` — the 3-level verification framework
- Instructions:
  - Verify each plan's must_haves (from plan frontmatter) or derive from phase success criteria
  - Check Level 1 (Exists): Do the artifacts exist?
  - Check Level 2 (Substantive): Are they real implementations or stubs?
  - Check Level 3 (Wired): Are they integrated with the system?
  - Identify any gaps or incomplete implementations
  - Write verification report to `.konductor/phases/{phase}/verification.md`

Wait for the verifier to complete. Verify that `verification.md` was created.

## Step 3: Read Verification Report

Read `.konductor/phases/{phase}/verification.md`.

Extract:
- Overall status: `ok` or `issues-found`
- Number of issues/gaps found
- List of gaps (if any)

Parse the verification report to determine:
- Did all must_haves pass Level 1 (Exists)?
- Did all must_haves pass Level 2 (Substantive)?
- Did all must_haves pass Level 3 (Wired)?
- Are there any failed truths or incomplete integrations?

**Status determination:**
- `ok`: All levels passed, no gaps
- `issues-found`: One or more levels failed, or gaps identified

## Step 4: Write Result File

Write `.konductor/.results/verify-{phase}.toml`:

```toml
step = "verify"
phase = "{phase}"
status = "{ok or issues-found}"
timestamp = {current ISO timestamp}
issues_count = {number of gaps/issues}
gaps = [
  {list of gap descriptions}
]
```

## Step 5a: If Status is OK

The phase is complete. Update state using MCP tools:

1. Call `state_transition` with `step = "complete"` to advance the pipeline
2. The tool automatically handles status and timestamp updates

Tell the user:
- Phase {phase} verification passed
- All must_haves are implemented and wired
- Next phase: {next_phase}
- Suggest: "Say 'next' to continue, or 'plan phase {next_phase}' to start planning."

## Step 5b: If Status is Issues-Found

The phase has gaps. Keep current step as "executed" (do NOT advance).

Tell the user:
- Phase {phase} verification found {N} issues
- List the gaps clearly (use the gap list from verification.md)
- Suggest: "Run 'plan phase {phase}' to create gap-closure plans, or manually fix the issues and re-run verification."

Call `state_add_blocker` MCP tool with the phase and reason: "Verification found {N} issues. See verification.md for details."

## Error Handling

If the verifier agent fails:
1. Write `.konductor/.results/verify-{phase}.toml` with `status = "error"` and error details
2. Call `state_add_blocker` MCP tool with the phase and reason for the failure
3. Report failure to user with actionable context
4. Do NOT retry crashed subagents

If `verification.md` is malformed or cannot be parsed:
1. Report the parsing error
2. Do not attempt to fix it automatically
3. Suggest manual review of the verification report
