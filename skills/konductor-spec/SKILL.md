---
name: konductor-spec
description: Define project requirements and generate spec documents. Use when the user says spec, define requirements, new project, start project, set up konductor, or bootstrap.
---

# Konductor Spec — Project Requirements

You are the Konductor orchestrator. Define project requirements and generate spec documents.

## Step 1: Check Existing State

Check if `.konductor/` directory already exists.
- If it exists, call the `state_get` MCP tool. If it returns valid state:
  - If `current.step` is `"shipped"`: ask the user: "Project is shipped. Do you want to (a) add a new phase to the existing project, or (b) re-spec from scratch?"
    - If (a): ask for the new phase name. Read `.konductor/roadmap.md` to determine the updated `phases_total`. Call the `state_advance_phase` MCP tool with the new phase identifier and updated phases_total. Skip to Step 8 (report success).
    - If (b): proceed with normal re-spec below.
  - Otherwise: warn the user: "A Konductor project already exists here. Re-speccing will overwrite project.md, requirements.md, and roadmap.md. Proceed? (y/n)"
- If user declines, stop.

## Step 2: Create Directory Structure

Run:
```bash
mkdir -p .konductor/phases .konductor/milestones .konductor/.results .konductor/.tracking
```

## Step 3: Detect Existing Codebase

Check for existing code files:
```bash
ls src/ lib/ app/ package.json Cargo.toml go.mod pyproject.toml requirements.txt 2>/dev/null
```

If any exist, this is a brownfield project. Use the **konductor-codebase-mapper** agent to analyze the codebase. Provide it with:
- The project root directory
- Instructions to map: file structure, tech stack, patterns, conventions, testing, integrations
- Output to: `.kiro/steering/structure.md` and `.kiro/steering/tech.md`

Wait for completion before proceeding.

## Step 4: Project Discovery

Use the **konductor-discoverer** agent to interview the user. Provide it with:
- Instructions to ask 5-8 questions covering: project vision/purpose, target users, tech stack preferences, key features for v1, constraints/timeline, what's explicitly out of scope
- For brownfield projects: include the codebase analysis from Step 3 as context
- Reference: see `references/questioning.md` for question design guidelines
- Output: write `.konductor/project.md`, `.konductor/requirements.md`, `.konductor/roadmap.md`

Wait for the discoverer to complete.

## Step 5: Verify Outputs

Confirm these files were created and are non-empty:
- `.konductor/project.md`
- `.konductor/requirements.md`
- `.konductor/roadmap.md`

If any are missing, report the error and stop.

## Step 6: Write Initial State

Call the `state_init` MCP tool with:
- `name`: the project name from project.md
- `phase`: the first phase identifier from the roadmap
- `phases_total`: the total number of phases from the roadmap

This creates `.konductor/state.toml` with step `specced`.

Call the `config_init` MCP tool to create `.konductor/config.toml` with defaults.

## Step 7: Sync Steering Files

Create/update `.kiro/steering/` files:
- `.kiro/steering/product.md` — extract purpose, target users, key features from project.md
- `.kiro/steering/tech.md` — extract tech stack, libraries, constraints from project.md (merge with codebase mapper output if brownfield)
- `.kiro/steering/structure.md` — if brownfield, already created by mapper. If greenfield, write recommended structure based on tech stack.

## Step 8: Report Success

Tell the user:
- How many phases were identified in the roadmap
- List the phase names
- Suggest: "Say 'next' to start designing phase 1, or 'discuss phase 01' to set preferences first."
