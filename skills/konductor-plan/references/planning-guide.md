# Planning Guide — Decomposing Phases into Execution Plans

This guide helps you transform a phase goal into concrete, executable plans.

## Core Philosophy: Vertical Slices

**Prefer vertical slices over horizontal layers.**

- **Bad (horizontal):** Plan 1 = data models, Plan 2 = API endpoints, Plan 3 = UI components
- **Good (vertical):** Plan 1 = user registration (model + API + UI), Plan 2 = user login (model + API + UI)

**Why vertical?**
- Each plan delivers a working feature end-to-end
- Plans can be executed in parallel (fewer dependencies)
- Users can test functionality incrementally
- Rollback is easier (one feature vs. broken layer)

**When to use horizontal:**
- Foundation work that everything depends on (database setup, auth middleware)
- Cross-cutting concerns (logging, error handling)
- Infrastructure (deployment, CI/CD)

## Wave Assignment

Plans execute in waves. Wave dependencies must form a DAG (directed acyclic graph).

**Rules:**
1. Plans with no dependencies → Wave 1
2. Plans depending only on Wave 1 outputs → Wave 2
3. Plans depending on Wave N outputs → Wave N+1
4. A plan can depend on multiple waves; use the max wave number + 1

**Goal:** Minimize wave count. More parallelism = faster execution.

**Example:**
- Wave 1: User model + migrations, Product model + migrations (parallel)
- Wave 2: Auth middleware (depends on User), Shopping cart (depends on User + Product)
- Wave 3: Checkout flow (depends on Shopping cart)

## Task Sizing

Each plan contains 2-5 tasks. Each task should take 15-60 minutes of execution time.

**If a plan would need more than 5 tasks:** Split it into multiple plans in the same wave.

**Task structure:**
- **files:** Which files to modify/create
- **action:** What to do (be specific)
- **verify:** How to check success (command to run)
- **done:** What "done" looks like (observable outcome)

**Example task:**
```markdown
### Task 2: Add password hashing to User model

- **files:** `src/models/user.rs`
- **action:** Import bcrypt crate, add `hash_password` method to User impl, call it in `new` constructor
- **verify:** `cargo test user::test_password_hashing`
- **done:** Passwords are hashed with bcrypt before storage
```

## Plan File Format

Each plan is a markdown file with TOML frontmatter and a structured body.

**Frontmatter fields:**
- `phase`: Phase identifier (e.g., "01-auth-system")
- `plan`: Plan number within the phase (1, 2, 3...)
- `wave`: Execution wave (1, 2, 3...)
- `depends_on`: List of plan numbers this plan depends on (e.g., `[1, 2]`)
- `type`: Either "execute" (standard implementation) or "tdd" (test-driven)
- `autonomous`: Boolean, true if executor can proceed without human input
- `requirements`: List of REQ-XX identifiers this plan addresses
- `files_modified`: List of files this plan will touch (helps with merge conflict prediction)
- `must_haves.truths`: Observable truths that must be true when done
- `must_haves.artifacts`: Files that must exist
- `must_haves.key_links`: Wiring checks (A imports B, C calls D)

**Complete example:**
```markdown
+++
phase = "01-auth-system"
plan = 1
wave = 1
depends_on = []
type = "execute"
autonomous = true
requirements = ["REQ-01", "REQ-02"]
files_modified = ["src/models/user.rs", "src/db/migrations/001_users.sql"]

[must_haves]
truths = ["Users can register with email and password", "Passwords are hashed with bcrypt"]
artifacts = ["src/models/user.rs", "src/db/migrations/001_users.sql"]
key_links = ["User model imported by auth routes", "bcrypt used in user.rs"]
+++

# Plan 01: User Model and Migrations

## Goal
Create the User data model and database schema to support user registration.

## Tasks

### Task 1: Create User struct

- **files:** `src/models/user.rs`
- **action:** Define User struct with fields: id (UUID), email (String), password_hash (String), created_at (DateTime)
- **verify:** `cargo check` passes
- **done:** User struct compiles

### Task 2: Add password hashing

- **files:** `src/models/user.rs`
- **action:** Import bcrypt, add `hash_password` method, call in constructor
- **verify:** `cargo test user::test_password_hashing`
- **done:** Passwords are hashed before storage

### Task 3: Create migration

- **files:** `src/db/migrations/001_users.sql`
- **action:** Write CREATE TABLE users with columns matching User struct
- **verify:** `sqlx migrate run` succeeds
- **done:** users table exists in database

### Task 4: Wire User model to auth routes

- **files:** `src/routes/auth.rs`
- **action:** Import User model, use it in registration handler
- **verify:** Compilation succeeds, User is referenced
- **done:** Registration route can create User instances
```

## Goal-Backward Methodology

Work backward from the phase goal to derive concrete requirements.

**Steps:**
1. **State the goal:** "Users can register with email and password"
2. **Derive observable truths:**
   - A user record exists in the database
   - The password is hashed, not plaintext
   - The registration endpoint returns success
3. **Derive required artifacts:**
   - `src/models/user.rs` (User struct)
   - `src/db/migrations/001_users.sql` (users table)
   - `src/routes/auth.rs` (registration endpoint)
4. **Derive required wiring:**
   - User model imported by auth routes
   - bcrypt library used in User model
   - Registration route calls User::new
5. **Identify key links:**
   - "User model imported by auth routes"
   - "bcrypt used in user.rs"
   - "POST /auth/register returns 201"

Use these to populate the `must_haves` section of the plan frontmatter.

## Requirement Coverage

Every requirement from `.konductor/requirements.md` must appear in at least one plan's `requirements` field.

**Check coverage:**
1. List all REQ-XX identifiers from requirements.md
2. Collect all `requirements` arrays from plan frontmatter
3. Ensure every REQ-XX appears at least once
4. If a requirement is missing, add a plan or extend an existing plan

**Multi-plan requirements:**
A requirement can span multiple plans. Example: "REQ-05: Users can manage their profile" might be split into:
- Plan 3: View profile (requirements = ["REQ-05"])
- Plan 4: Edit profile (requirements = ["REQ-05"])

## TDD Detection

If a task can be expressed as "expect(fn(input)).toBe(output)", make it a TDD plan.

**Indicators:**
- Pure functions (no I/O)
- Clear input/output contract
- Algorithmic logic (sorting, parsing, validation)
- Data transformations

**TDD plan differences:**
- `type = "tdd"` in frontmatter
- First task writes tests
- Remaining tasks implement to pass tests
- Verification is `cargo test` or equivalent

**Example TDD task:**
```markdown
### Task 1: Write password validation tests

- **files:** `src/validation/password_test.rs`
- **action:** Write tests for: min 8 chars, has uppercase, has number, has special char
- **verify:** Tests exist and fail
- **done:** 4 test cases written

### Task 2: Implement password validation

- **files:** `src/validation/password.rs`
- **action:** Write validate_password function to satisfy tests
- **verify:** `cargo test validation::password` passes
- **done:** All password validation tests pass
```

## Interface Context

When plans depend on each other, include interface definitions so executors don't need to explore the codebase.

**Include in dependent plans:**
- Type signatures
- Function signatures
- API contracts (request/response shapes)
- Event names and payloads

**Example:**
```markdown
## Dependencies

This plan depends on Plan 1 (User model). Expected interface:

```rust
pub struct User {
    pub id: Uuid,
    pub email: String,
    // password_hash is private
}

impl User {
    pub fn new(email: String, password: String) -> Result<Self, Error>;
    pub fn verify_password(&self, password: &str) -> bool;
}
```

Use this interface in Task 2 when implementing the login handler.
```

## Anti-Patterns to Avoid

**1. God plans:** A plan that touches 20+ files or has 10+ tasks. Split it.

**2. Circular dependencies:** Plan A depends on Plan B, Plan B depends on Plan A. Use waves to break cycles.

**3. Vague tasks:** "Set up authentication" is too vague. Be specific: "Add bcrypt to Cargo.toml", "Create User model", etc.

**4. Missing verification:** Every task needs a `verify` command. "Manual testing" is not sufficient.

**5. Underspecified must_haves:** "The feature works" is not an observable truth. Be concrete: "POST /auth/register returns 201 with user JSON".

**6. Ignoring research:** If research.md exists, use it. Don't re-discover libraries or patterns.

## Checklist for Completed Plans

Before finalizing plans, verify:

- [ ] Every plan has valid TOML frontmatter
- [ ] Wave numbers form a valid DAG (no cycles)
- [ ] Each plan has 2-5 tasks
- [ ] Each task has files, action, verify, and done
- [ ] Every requirement from requirements.md is covered
- [ ] `must_haves` section is complete and observable
- [ ] Dependent plans include interface context
- [ ] File paths in `files_modified` are realistic
- [ ] Verification commands are concrete (not "manual testing")
- [ ] Plans prefer vertical slices over horizontal layers
