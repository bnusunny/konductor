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

Each plan contains 2-5 tasks. Each task is broken into **bite-sized steps that take 2-5 minutes each**. Every step that involves code must include the actual code block — no prose descriptions of what to write.

**If a plan would need more than 5 tasks:** Split it into multiple plans in the same wave.

**Task structure:**
- **files:** Which files to modify/create
- **action:** What to do — broken into numbered steps, each 2-5 minutes
- **verify:** How to check success (command to run)
- **done:** What "done" looks like (observable outcome)

**Each step within a task must:**
1. Be completable in 2-5 minutes
2. Include the exact code to write (for code steps)
3. Include the command to run (for verification steps)
4. Be independently verifiable

**Example task with bite-sized steps (TDD):**
```markdown
### Task 2: Add password hashing to User model

- **files:** `src/models/user.rs`
- **action:**
  1. Write failing test:
     ```rust
     #[cfg(test)]
     mod tests {
         use super::*;

         #[test]
         fn test_password_is_hashed() {
             let user = User::new("test@example.com".into(), "Secret123!".into()).unwrap();
             assert_ne!(user.password_hash(), "Secret123!");
             assert!(user.verify_password("Secret123!"));
         }
     }
     ```
  2. Run `cargo test user::tests::test_password_is_hashed` — confirm it fails (method not found)
  3. Implement:
     ```rust
     use bcrypt::{hash, verify, DEFAULT_COST};

     impl User {
         pub fn new(email: String, password: String) -> Result<Self, AuthError> {
             let password_hash = hash(&password, DEFAULT_COST)
                 .map_err(|_| AuthError::HashError)?;
             Ok(Self { id: Uuid::new_v4(), email, password_hash, created_at: Utc::now() })
         }

         pub fn password_hash(&self) -> &str { &self.password_hash }

         pub fn verify_password(&self, password: &str) -> bool {
             verify(password, &self.password_hash).unwrap_or(false)
         }
     }
     ```
  4. Run `cargo test user::tests::test_password_is_hashed` — confirm it passes
- **verify:** `cargo test user::tests::test_password_is_hashed`
- **done:** Passwords are hashed with bcrypt before storage, verified by test
```

## No Placeholders

Every step must contain the actual content an engineer needs to execute it. The **konductor-plan-checker** agent enforces this rule and will reject plans that violate it.

**Banned patterns:**
- `"TBD"`, `"TODO"`, `"implement later"`, `"fill in details"`
- `"Add appropriate error handling"` / `"add validation"` / `"handle edge cases"` (show the actual error handling, validation, or edge case code)
- `"Write tests for the above"` without actual test code (include the test code)
- `"Similar to Task N"` (repeat the code — the engineer may be reading tasks out of order)
- Steps that describe what to do without showing how (code blocks required for code steps)
- References to types, functions, or methods not defined in any prior or current task

**Rule:** If a step involves writing code, the step must include the code block. If a step involves running a command, the step must include the command. No exceptions.

## Plan File Format

Each plan is a markdown file with TOML frontmatter and a structured body.

**Frontmatter fields:**
- `phase`: Phase identifier (e.g., "01-auth-system")
- `plan`: Plan number within the phase (1, 2, 3...)
- `wave`: Execution wave (1, 2, 3...)
- `depends_on`: List of plan numbers this plan depends on (e.g., `[1, 2]`)
- `type`: Either "tdd" (test-driven, default) or "execute" (standard implementation). Use `type = "execute"` to opt out of TDD for infrastructure, configuration, or documentation tasks. The planner must always emit an explicit `type` field.
- `autonomous`: Boolean, true if executor can proceed without human input
- `requirements`: List of REQ-XX identifiers this plan addresses
- `files_modified`: List of files this plan will touch (helps with merge conflict prediction)
- `must_haves.truths`: Observable truths that must be true when done
- `must_haves.artifacts`: Files that must exist
- `must_haves.key_links`: Wiring checks (A imports B, C calls D)

**Body sections (in order):**
- `## Goal`: What this plan delivers
- `## Design`: Technical design for this plan's implementation
  - `### Approach`: Why this approach, what alternatives were considered
  - `### Key Interfaces`: Function signatures, type definitions, API contracts introduced or modified by this plan
  - `### Error Handling`: How errors are handled within this plan's scope
  - `### Trade-offs`: What was traded off and why
- `## Dependencies` (optional): Interface context from plans this one depends on
- `## Tasks`: Ordered list of implementation tasks

**Complete example:**
```markdown
+++
phase = "01-auth-system"
plan = 1
wave = 1
depends_on = []
type = "tdd"
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

## Design

### Approach
Use a single User struct with private password_hash field. Bcrypt for hashing (industry standard, built-in salt). Migration creates the users table directly — no ORM migration framework needed at this stage.

### Key Interfaces
```rust
pub struct User {
    pub id: Uuid,
    pub email: String,
    password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, password: String) -> Result<Self, AuthError>;
    pub fn verify_password(&self, password: &str) -> bool;
}
```

### Error Handling
`User::new` returns `AuthError::WeakPassword` if password doesn't meet requirements, `AuthError::HashError` if bcrypt fails. Callers handle these via the Result type.

### Trade-offs
- Bcrypt over argon2: simpler dependency, sufficient for this use case, slightly slower but acceptable
- No email uniqueness enforcement at struct level — handled by DB unique constraint

## Tasks

### Task 1: Create User struct with tests

- **files:** `src/models/user.rs`
- **action:**
  1. Write failing test:
     ```rust
     #[cfg(test)]
     mod tests {
         use super::*;

         #[test]
         fn test_new_user_has_correct_email() {
             let user = User::new("test@example.com".into(), "Secret123!".into()).unwrap();
             assert_eq!(user.email, "test@example.com");
         }

         #[test]
         fn test_new_user_has_uuid() {
             let user = User::new("test@example.com".into(), "Secret123!".into()).unwrap();
             assert!(!user.id.is_nil());
         }
     }
     ```
  2. Run `cargo test models::user::tests` — confirm it fails (User not defined)
  3. Implement:
     ```rust
     use chrono::{DateTime, Utc};
     use uuid::Uuid;

     pub struct User {
         pub id: Uuid,
         pub email: String,
         password_hash: String,
         pub created_at: DateTime<Utc>,
     }

     impl User {
         pub fn new(email: String, password: String) -> Result<Self, AuthError> {
             Ok(Self {
                 id: Uuid::new_v4(),
                 email,
                 password_hash: password, // placeholder — next task adds hashing
                 created_at: Utc::now(),
             })
         }
     }
     ```
  4. Run `cargo test models::user::tests` — confirm both tests pass
- **verify:** `cargo test models::user::tests`
- **done:** User struct compiles and passes basic tests

### Task 2: Add password hashing

- **files:** `src/models/user.rs`
- **action:**
  1. Add failing test:
     ```rust
     #[test]
     fn test_password_is_hashed_not_plaintext() {
         let user = User::new("test@example.com".into(), "Secret123!".into()).unwrap();
         assert_ne!(user.password_hash, "Secret123!");
     }

     #[test]
     fn test_verify_correct_password() {
         let user = User::new("test@example.com".into(), "Secret123!".into()).unwrap();
         assert!(user.verify_password("Secret123!"));
     }

     #[test]
     fn test_verify_wrong_password() {
         let user = User::new("test@example.com".into(), "Secret123!".into()).unwrap();
         assert!(!user.verify_password("WrongPass1!"));
     }
     ```
  2. Run `cargo test models::user::tests` — confirm new tests fail
  3. Update `User::new` and add `verify_password`:
     ```rust
     use bcrypt::{hash, verify, DEFAULT_COST};

     impl User {
         pub fn new(email: String, password: String) -> Result<Self, AuthError> {
             let password_hash = hash(&password, DEFAULT_COST)
                 .map_err(|_| AuthError::HashError)?;
             Ok(Self { id: Uuid::new_v4(), email, password_hash, created_at: Utc::now() })
         }

         pub fn verify_password(&self, password: &str) -> bool {
             verify(password, &self.password_hash).unwrap_or(false)
         }
     }
     ```
  4. Run `cargo test models::user::tests` — confirm all 5 tests pass
- **verify:** `cargo test models::user::tests`
- **done:** Passwords are hashed with bcrypt, verified by 3 new tests

### Task 3: Create migration

- **files:** `src/db/migrations/001_users.sql`
- **action:**
  1. Write the migration:
     ```sql
     CREATE TABLE users (
         id UUID PRIMARY KEY,
         email VARCHAR(255) NOT NULL UNIQUE,
         password_hash VARCHAR(255) NOT NULL,
         created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
     );

     CREATE INDEX idx_users_email ON users (email);
     ```
  2. Run `sqlx migrate run` — confirm it succeeds
- **verify:** `sqlx migrate run`
- **done:** users table exists in database with email unique constraint
```

## Phase-Level Design Document

Before writing individual plans, the planner must produce `.konductor/phases/{phase}/design.md`. This document captures the overall technical design for the phase, giving reviewers and executors a holistic view.

**Contents:**
1. **Overview** — one paragraph summarizing the architectural approach for this phase
2. **Components** — which modules, files, or services are involved and their responsibilities
3. **Interactions** — how components communicate (function calls, API requests, event flows)
4. **Key Decisions** — architectural choices that span multiple plans, with rationale
5. **Shared Interfaces** — type definitions, function signatures, or API contracts used across multiple plans

**Template:**
```markdown
# Design: Phase {phase} — {name}

## Overview
{One paragraph: what is being built and the high-level approach}

## Components
- **{component}**: {responsibility}
- **{component}**: {responsibility}

## Interactions
{Describe how components interact — which calls which, data flow}

## Key Decisions

### {Decision title}
**Choice:** {what was decided}
**Rationale:** {why}
**Alternatives considered:** {what else was evaluated}

## Shared Interfaces
{Type definitions, function signatures, or API contracts referenced by multiple plans}
```

Individual plan `## Design` sections should reference this document for shared context rather than duplicating it. For example: "See design.md Key Decisions for the choice of bcrypt over argon2."

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

## TDD as Default

TDD is the default execution mode for all plans. The planner must always emit `type = "tdd"` unless the plan explicitly opts out.

**Backward compatibility:** Existing plans without an explicit `type` field are treated as `"execute"`. The planner must always emit an explicit `type` field going forward, making the default moot for well-formed plans.

**Opt-out with `type = "execute"`:** Use `type = "execute"` for tasks where TDD doesn't apply:
- Infrastructure plans (SAM templates, Terraform, CI/CD configs)
- Configuration files (TOML, YAML, JSON configs)
- Documentation-only plans (README, guides, specs)
- Refactoring plans where existing tests already cover the behavior

**TDD task structure (RED → GREEN → REFACTOR):**
1. **RED:** Write a failing test with the exact test code
2. **Verify RED:** Run the test command — confirm it fails
3. **GREEN:** Write the minimal implementation to pass the test
4. **Verify GREEN:** Run the test command — confirm it passes
5. **REFACTOR** (optional): Clean up while keeping tests green

**Example TDD task:**
```markdown
### Task 1: Write password validation tests and implement

- **files:** `src/validation/password.rs`, `src/validation/password_test.rs`
- **action:**
  1. Write failing tests:
     ```rust
     #[cfg(test)]
     mod tests {
         use super::validate_password;

         #[test]
         fn rejects_short_password() {
             assert!(validate_password("Ab1!").is_err());
         }

         #[test]
         fn rejects_no_uppercase() {
             assert!(validate_password("abcdefg1!").is_err());
         }

         #[test]
         fn rejects_no_number() {
             assert!(validate_password("Abcdefgh!").is_err());
         }

         #[test]
         fn accepts_valid_password() {
             assert!(validate_password("Secret123!").is_ok());
         }
     }
     ```
  2. Run `cargo test validation::password` — confirm all 4 tests fail
  3. Implement:
     ```rust
     pub fn validate_password(password: &str) -> Result<(), &'static str> {
         if password.len() < 8 { return Err("too short"); }
         if !password.chars().any(|c| c.is_uppercase()) { return Err("no uppercase"); }
         if !password.chars().any(|c| c.is_numeric()) { return Err("no number"); }
         Ok(())
     }
     ```
  4. Run `cargo test validation::password` — confirm all 4 tests pass
- **verify:** `cargo test validation::password`
- **done:** Password validation passes all 4 test cases
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

- [ ] Phase-level `design.md` exists with overview, components, interactions, key decisions, and shared interfaces
- [ ] Every plan has valid TOML frontmatter
- [ ] Every plan has an explicit `type` field (`"tdd"` or `"execute"`)
- [ ] Plans default to TDD unless explicitly opted out (infra, config, docs)
- [ ] Every plan has a `## Design` section with Approach, Key Interfaces, Error Handling, and Trade-offs
- [ ] Wave numbers form a valid DAG (no cycles)
- [ ] Each plan has 2-5 tasks
- [ ] Each task has bite-sized steps (2-5 minutes each) with code blocks for code steps
- [ ] No placeholder patterns (TBD, TODO, implement later, etc.)
- [ ] Each task has files, action, verify, and done
- [ ] Every requirement from requirements.md is covered
- [ ] `must_haves` section is complete and observable
- [ ] Dependent plans include interface context
- [ ] File paths in `files_modified` are realistic
- [ ] Verification commands are concrete (not "manual testing")
- [ ] Plans prefer vertical slices over horizontal layers
