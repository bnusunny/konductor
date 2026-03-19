# Execution Guide — For Konductor Executor Agents

This guide is for executor subagents that implement individual plans. You are responsible for executing one plan from start to finish.

## Your Role

You are a **konductor-executor** agent. You receive:
- A plan file with tasks to complete
- Git configuration (auto-commit, branching strategy)
- Reference to this guide

Your job:
1. Read and understand the plan
2. Execute each task in order
3. Write tests when required (TDD plans)
4. Commit changes following the protocol
5. Write a summary when done

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

**One commit per task** (preferred):
- After completing each task, commit the changes
- Keeps history granular and reviewable
- Easier to roll back individual changes

**Exceptions:**
- If tasks are tightly coupled and splitting commits would break functionality, combine them
- Always explain in the commit body why tasks were combined

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

## Summary Writing

After completing all tasks (or encountering a blocker), write a summary file.

**File location:** `.konductor/phases/{phase}/plans/{plan-number}-summary.md`

**File name examples:**
- `001-summary.md`
- `002-summary.md`
- `010-summary.md`

### Summary Structure

```markdown
# Plan {plan-number} Summary

## Status
[Completed | Blocked | Partial]

## Files Created
- `src/models/user.rs` — User struct with password hashing
- `src/db/migrations/001_users.sql` — Users table migration

## Files Modified
- `src/routes/auth.rs` — Added registration endpoint
- `Cargo.toml` — Added bcrypt dependency

## Tests Added
- `user::test_password_hashing` — Verifies bcrypt integration
- `auth::test_registration_endpoint` — Verifies POST /auth/register

### Test Results
```
cargo test user
   Compiling auth-system v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 2.3s
     Running unittests (target/debug/deps/auth_system-abc123)
running 2 tests
test user::test_password_hashing ... ok
test auth::test_registration_endpoint ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Deviations from Plan
1. **Rule 1:** Fixed type error in existing auth routes that prevented compilation
2. **Rule 2:** Added email validation to registration endpoint (plan didn't specify)
3. **Rule 3:** Added bcrypt to Cargo.toml (plan assumed it was already present)

## Decisions Made
- Used bcrypt cost factor of 12 (industry standard for password hashing)
- Made password_hash field private to prevent accidental exposure
- Added index on users.email for faster lookups during login

## Blockers Encountered
None. All tasks completed successfully.

## Verification
All must_haves from plan frontmatter verified:
- [x] Users can register with email and password (POST /auth/register returns 201)
- [x] Passwords are hashed with bcrypt (verified in test)
- [x] User model imported by auth routes (compiler confirms)
```

### Summary Requirements

Your summary MUST include:
- **Status:** One of [Completed, Blocked, Partial]
- **Files created:** List with brief descriptions
- **Files modified:** List with brief descriptions
- **Tests added:** Test names and what they verify
- **Test results:** Actual output from test runner (paste full output)
- **Deviations:** Every deviation with rule number and explanation
- **Decisions:** Technical choices you made
- **Blockers:** Any issues that stopped you (or "None")
- **Verification:** Checklist of must_haves from plan frontmatter

**If blocked:** Explain clearly what stopped you and what information or decision is needed to proceed.

## Working with TDD Plans

If the plan frontmatter has `type = "tdd"`, follow test-driven development:

See `references/tdd.md` for detailed patterns. Key points:

1. **Red phase:** Write failing tests first
2. **Green phase:** Write minimal code to pass tests
3. **Refactor phase:** Improve code while keeping tests green
4. **Commit after each phase** (red commit, green commit, refactor commit)

TDD plans prioritize observable behavior over implementation details. Tests should verify the plan's `must_haves.truths`.
