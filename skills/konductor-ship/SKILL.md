---
name: konductor-ship
description: Ship and finalize the project. Use when the user says ship, release, finalize, we're done, ready to ship, or complete project.
---

# Konductor Ship — Project Finalization and Release

You are the Konductor orchestrator. Finalize and ship the project after all phases are complete.

## Step 1: Read Project State

Call the `state_get` MCP tool to verify project readiness:
- Check `current.phase` — should be the last phase
- Check `current.step` — should be "complete"
- Check `progress.phases_complete` — should equal `phases_total`

If the project is not ready to ship, proceed to Step 2 to report the gap.

## Step 2: Verify All Phases Complete

Read `.konductor/roadmap.md` to get the full list of phases.

For each phase:
1. Check if `.konductor/phases/{phase}/` directory exists
2. Check if summary files exist for all plans
3. Verify that the phase has `step = "complete"` in state history

If any phases are incomplete:
- List which phases are not complete
- Show their current step
- Tell the user: "Cannot ship yet. Complete these phases first: {list}. Say 'status' to see current progress."
- Stop here

## Step 3: Cross-Phase Integration Verification

Spawn the **konductor-verifier** agent with instructions to perform a cross-phase integration check:
- Verify that outputs from earlier phases integrate correctly with later phases
- Check that API contracts are honored
- Test critical user flows that span multiple phases
- Look for integration gaps or inconsistencies between phase boundaries
- Reference: see `references/verification.md` for verification patterns

Provide the verifier with:
- All phase directories: `.konductor/phases/*/`
- All summary files from each phase
- The project roadmap for context

Wait for the verifier to complete and write its results to `.konductor/.results/verify-integration.toml`.

## Step 4: Check Verification Results

Read `.konductor/.results/verify-integration.toml`.

**If status = "ok":**
- Proceed to Step 5 (finalization)

**If status = "error" or issues found:**
- Display the issues to the user
- List affected phases and specific problems
- Tell the user: "Integration issues found. Fix these before shipping: {list of issues}. You can use 'exec' to re-run specific plans or fix manually."
- Stop here

## Step 5: Create Release Tag

Read `.konductor/project.md` to extract:
- Project name
- Version (if specified, or use "v1.0.0" as default)

Create a git tag:
```bash
git tag -a {version} -m "Release {version}: {project name} complete

All {phases_total} phases implemented and verified.

Generated with Konductor orchestrator"
```

Tell the user:
> "Created git tag: {version}"

## Step 6: Archive Project State

Create an archive of the current state for historical reference:

```bash
mkdir -p .konductor/milestones/{version}
cp -r .konductor/phases .konductor/milestones/{version}/
cp .konductor/state.toml .konductor/milestones/{version}/
cp .konductor/roadmap.md .konductor/milestones/{version}/
cp .konductor/project.md .konductor/milestones/{version}/
```

Write a milestone summary file at `.konductor/milestones/{version}/summary.md`:

```markdown
# {project name} — {version} Release

Released: {current date}

## Summary
{brief project description from project.md}

## Phases Completed
{for each phase}
- Phase {number}: {name} — {goal}
{/for}

## Metrics
- Total phases: {phases_total}
- Total agent sessions: {total_agent_sessions}
- Project duration: {initialized date} to {current date}

## Verification
Cross-phase integration verified on {current date}.

All phase outputs integrated successfully.
```

## Step 7: Update State to Shipped

Call `state_transition` with `step = "shipped"` to finalize the project state.

## Step 8: Report Success

Tell the user:

```
# Project Shipped! 🎉

{project name} {version} has been finalized.

## Summary
- {phases_total} phases completed
- {total_agent_sessions} agent sessions
- Verified: All integration checks passed
- Tagged: {version}
- Archived: .konductor/milestones/{version}/

## Next Steps
- Push the git tag: `git push origin {version}`
- Review the milestone summary: `.konductor/milestones/{version}/summary.md`
- Consider deploying your project

Great work!
```

## Error Handling

**Incomplete phases:**
If phases are incomplete, do NOT proceed with shipping. Report the gap and stop.

**Verification failures:**
If the verifier finds issues, do NOT create tags or archives. Report issues and stop.

**Git tag already exists:**
If the tag already exists, warn the user:
> "Tag {version} already exists. Use a different version or delete the existing tag first."

Then stop.

**Missing roadmap or state:**
If critical files are missing:
1. Report which files are missing
2. Suggest running diagnostic: "Say 'status' to check project state."
3. Stop
