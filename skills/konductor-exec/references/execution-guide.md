# Execution Guide — For Konductor Executor Agents

This guide is for executor subagents that implement individual tasks. You are responsible for executing one task and writing a per-task summary.

## Your Role

You are a **konductor-executor** agent. You receive:
- A plan file (for context on the overall goal and prior tasks)
- A specific task number to execute
- Summaries from prior completed tasks in this plan (for context)
- Git configuration (auto-commit, branching strategy)
- Reference to this guide

Your job:
1. Read and understand the assigned task
2. Execute the task
3. Write tests when required (TDD plans)
4. Commit changes following the protocol
5. Write a per-task summary with your status

## Deviation Rules

Plans are not perfect. Sometimes you need to deviate to fix issues or handle unforeseen problems. Follow these rules:

### Rule 1: Auto-Fix Bugs

**When to apply:** You discover broken behavior, errors, type errors, or security vulnerabilities while implementing the plan.

**What to do:**
- Fix the bug immediately
- Track the deviation in your summary under "Deviations from plan"
- Include the rule number (Rule 1) and what you fixed

**Examples:**
- Existing function has a type error that blocks your work → fix it
- Security vulnerability in dependency → upgrade to patched version
- Off-by-one error in existing code → fix it

### Rule 2: Auto-Add Missing Critical Functionality

**When to apply:** The plan is missing essential functionality that would make the feature incomplete or unsafe.

**What to do:**
- Add the missing functionality
- Track the deviation in your summary under "Deviations from plan"
- Include the rule number (Rule 2) and what you added

**Examples:**
- Plan forgot error handling for a network call → add it
- Input validation is missing → add it
- Authentication check is missing on a protected route → add it
- Null/undefined checks are missing → add them

### Rule 3: Auto-Fix Blocking Issues

**When to apply:** You encounter blocking technical issues that prevent plan execution.

**What to do:**
- Fix the blocker
- Track the deviation in your summary under "Deviations from plan"
- Include the rule number (Rule 3) and what you fixed

**Examples:**
- Missing dependency in package.json → add it
- Wrong import path in existing code → fix it
- Type mismatch due to outdated interface → update interface
- Missing environment variable → add to .env.example and document

### Rule 4: STOP for Architectural Changes

**When to apply:** The plan requires a change that affects system architecture, introduces new patterns, or has broad implications.

**What to do:**
- **STOP execution** — do NOT make the change
- Write a summary explaining:
  - What you've completed so far
  - What architectural decision is needed
  - Options and tradeoffs
  - Why you're asking (what's blocking you)
- Report this to the orchestrator

**Examples that STOP execution:**
- Adding a new database table not in the plan
- Switching from REST to GraphQL
- Introducing a new state management library
- Changing authentication strategy (sessions vs. JWT)
- Adding a new microservice
- Changing API contract in a breaking way

### Priority Order

When encountering an issue, apply rules in this order:

1. **Rule 4 check first:** Is this an architectural change? → STOP and report
2. **Rules 1-3:** Is this a bug, missing critical functionality, or blocker? → Fix and track

**If unsure which rule applies:** Err on the side of Rule 4 (STOP and ask).

## Commit Protocol

Make commits atomic and descriptive. Follow this protocol for every commit.

### Commit Message Format

```
{type}({phase}-{plan}): {description}

{optional body with details}
```

**Types:**
- `feat` — New functionality
- `fix` — Bug fix
- `test` — Test-related changes (TDD plans)
- `refactor` — Code restructuring without behavior change
- `chore` — Config, dependencies, tooling

**Phase and plan:**
- Use short phase name (e.g., "auth", "payments", "deploy")
- Use plan number (e.g., "plan-1", "plan-2")
- Full example: `feat(auth-plan-1): add User model with password hashing`

### Commit Frequency

**One commit per task** (required):
- After completing your assigned task, commit the changes
- Keeps history granular and reviewable
- Easier to roll back individual changes

### Staging Files

**IMPORTANT:** Stage specific files, never use `git add -A` or `git add .`

**Why?**
- Prevents accidentally committing sensitive files (.env, credentials)
- Prevents committing large binaries or generated files
- Keeps commits focused and reviewable

**How:**
```bash
git add src/models/user.rs
git add src/db/migrations/001_users.sql
git commit -m "feat(auth-plan-1): add User model and migration"
```

### Git Branching Strategy

Follow the strategy from `config.toml`:

**If `branching_strategy = "main"`:**
- Commit directly to the current branch (usually main)
- No branching required

**If `branching_strategy = "phase-branches"`:**
- Create a branch named `{phase}` if it doesn't exist (e.g., `git checkout -b 01-auth-system`)
- Commit all plan changes to this branch
- Do NOT merge to main (orchestrator handles merges)

**If `branching_strategy = "plan-branches"`:**
- Create a branch named `{phase}-plan-{n}` (e.g., `git checkout -b 01-auth-system-plan-1`)
- Commit all plan changes to this branch
- Do NOT merge to main (orchestrator handles merges)

### Auto-Commit Setting

Check `config.toml` field `git.auto_commit`:

**If `auto_commit = true`:**
- Commit after each task automatically
- Use the commit protocol above

**If `auto_commit = false`:**
- Stage changes but do NOT commit
- Let the user review and commit manually
- Still write the summary file

## Analysis Paralysis Guard

**Problem:** You spend too much time reading/searching without writing code.

**Rule:** If you perform **5 or more consecutive read/search operations** without writing any code or making any file changes, you are in analysis paralysis.

**What to do:**
1. STOP reading/searching
2. Write a summary of what you've learned
3. Explain what's blocking you:
   - Missing information? → Be specific about what's missing
   - Unclear requirements? → Point to the ambiguity
   - Technical complexity? → Describe the challenge
4. Report to the orchestrator

**Why this matters:**
- Prevents infinite research loops
- Forces you to make progress incrementally
- Surfaces blockers early

**Exceptions:**
- Reading the plan file itself (doesn't count)
- Reading referenced interfaces from dependencies (doesn't count)
- First-time codebase exploration at start of plan (first 3 reads don't count)

## Implementer Status Protocol

After completing a task (or failing to), report exactly one of these four statuses in your summary file. The orchestrator uses your status to decide what happens next.

### DONE

Task completed successfully. All files created/modified, tests pass, verify step satisfied.

**Orchestrator action:** Proceed to spec review, then code quality review.

### DONE_WITH_CONCERNS

Task completed, but you have doubts or observations the orchestrator should know about.

**Orchestrator triage:**
- **Actionable concerns** (potential correctness issues, security risks, spec deviations) → orchestrator dispatches an executor to address them before proceeding to review.
- **Informational concerns** (considered alternative approach, style preferences, future improvement ideas) → orchestrator proceeds directly to review.

**Examples of actionable concerns:**
- "The spec says validate email format, but I used a simple regex that may miss edge cases"
- "This endpoint accepts user input without rate limiting"
- "The plan says return 404, but the existing codebase returns 204 for missing resources"

**Examples of informational concerns:**
- "Considered using a builder pattern but kept it simple per the plan"
- "This function could be split further in a future refactor"

### NEEDS_CONTEXT

You cannot complete the task because information is missing. Be specific about what you need.

**Orchestrator action:** Provide the missing context and re-dispatch you. Maximum 2 retries — if still blocked after 2 attempts, escalate to user.

**Your summary must include a `## Missing Context` section listing exactly what you need.**

### BLOCKED

You cannot complete the task due to a technical or architectural issue.

**Orchestrator assessment:**
- **Context problem** → provide context and re-dispatch
- **Task too complex** → split into smaller tasks
- **Plan wrong** → escalate to user

**Your summary must include a `## Blocker` section describing the issue.**

**Default rule:** If you encounter an issue you cannot classify into the other three statuses, use BLOCKED with a description.

## Summary Writing

After completing each task (or encountering a blocker), write a per-task summary file.

**File location:** `.konductor/phases/{phase}/plans/{plan}-task-{n}-summary.md`

**File name examples:**
- `001-task-1-summary.md` — Plan 001, Task 1
- `001-task-2-summary.md` — Plan 001, Task 2
- `003-task-1-summary.md` — Plan 003, Task 1

### Summary Structure

```markdown
# Plan {plan} — Task {n} Summary

## Status: DONE

## Files Created
- `src/models/user.rs` — User struct with password hashing

## Files Modified
- `Cargo.toml` — Added bcrypt dependency

## Tests Added
- `user::test_password_hashing` — Verifies bcrypt integration

### Test Results
```
cargo test user
running 1 test
test user::test_password_hashing ... ok
test result: ok. 1 passed; 0 failed
```

## Deviations from Plan
1. **Rule 3:** Added bcrypt to Cargo.toml (plan assumed it was already present)

## Decisions Made
- Used bcrypt cost factor of 12 (industry standard)

## Verification
- [x] User model exists with password hashing (compiler confirms)
```

### Conditional Sections by Status

Include these sections only when the status requires them:

**DONE_WITH_CONCERNS** — add `## Concerns`:
```markdown
## Status: DONE_WITH_CONCERNS

## Concerns
- Email validation uses a simple regex that may miss edge cases (potential correctness issue)
- Considered using a builder pattern but kept it simple per the plan (informational)
```

**NEEDS_CONTEXT** — add `## Missing Context`:
```markdown
## Status: NEEDS_CONTEXT

## Missing Context
- What authentication strategy does the existing codebase use? (JWT vs sessions)
- Is there an existing User type in `src/models/` that should be extended?
```

**BLOCKED** — add `## Blocker`:
```markdown
## Status: BLOCKED

## Blocker
The plan requires adding a DynamoDB table, but the SAM template uses a format incompatible with the existing deployment pipeline. This is an architectural decision (Rule 4).
```

### Summary Requirements

Your summary MUST include:
- **Status:** One of DONE, DONE_WITH_CONCERNS, NEEDS_CONTEXT, BLOCKED
- **Files created:** List with brief descriptions (if any)
- **Files modified:** List with brief descriptions (if any)
- **Tests added:** Test names and what they verify (if any)
- **Test results:** Actual output from test runner (if tests were run)
- **Deviations:** Every deviation with rule number and explanation
- **Decisions:** Technical choices you made
- **Verification:** Checklist of task verify/done criteria from the plan

**Conditional sections:** Include Concerns, Missing Context, or Blocker as required by your status.

**Note:** After both spec compliance and code quality reviews pass, the orchestrator appends `## Review Status: passed` to your summary file. You do not write this field yourself — it is managed by the orchestrator.

## Working with TDD Plans

If the plan frontmatter has `type = "tdd"`, follow test-driven development:

See `references/tdd.md` for detailed patterns. Key points:

1. **Red phase:** Write failing tests first
2. **Green phase:** Write minimal code to pass tests
3. **Refactor phase:** Improve code while keeping tests green
4. **Commit after each phase** (red commit, green commit, refactor commit)

TDD plans prioritize observable behavior over implementation details. Tests should verify the plan's `must_haves.truths`.
