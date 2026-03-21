# Plan File Format

Execution plans are markdown files with TOML frontmatter, stored in `.konductor/phases/{phase}/plans/`.

## Frontmatter Fields

Plans use `+++` delimited TOML frontmatter:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `phase` | string | yes | Phase identifier (e.g., `"01"`) |
| `plan` | integer | yes | Plan number within the phase |
| `wave` | integer | yes | Execution wave (1, 2, 3...). Plans in the same wave can run in parallel. |
| `title` | string | yes | Human-readable plan title |
| `depends_on` | integer[] | no | Plan numbers this plan depends on (e.g., `[1, 2]`) |
| `type` | string | no | `"execute"` (default) or `"tdd"` (test-driven) |
| `autonomous` | boolean | no | `true` if executor can proceed without human input |
| `requirements` | string[] | no | Requirement IDs this plan addresses (e.g., `["REQ-01"]`) |
| `files_modified` | string[] | no | Files this plan will create or modify |

### `[must_haves]` Section

| Field | Type | Description |
|-------|------|-------------|
| `truths` | string[] | Observable truths that must be true when done |
| `artifacts` | string[] | Files that must exist after execution |
| `key_links` | string[] | Wiring checks (e.g., "A imports B", "C calls D") |

## Complete Example

```markdown
+++
phase = "01"
plan = 1
wave = 1
title = "User Model and Migrations"
depends_on = []
type = "tdd"
autonomous = true
requirements = ["REQ-01", "REQ-02"]
files_modified = ["src/models/user.rs", "src/db/migrations/001_users.sql"]

[must_haves]
truths = ["Users can register with email", "Passwords are hashed with bcrypt"]
artifacts = ["src/models/user.rs", "src/db/migrations/001_users.sql"]
key_links = ["User model imported by auth routes"]
+++

# Plan 01: User Model and Migrations

## Goal
Create the User data model and database schema.

## Tasks

### Task 1: Create User struct

- **files:** `src/models/user.rs`
- **action:** Define User struct with id, email, password_hash, created_at
- **verify:** `cargo check` passes
- **done:** User struct compiles

### Task 2: Add password hashing

- **files:** `src/models/user.rs`
- **action:** Import bcrypt, add hash_password method
- **verify:** `cargo test user::test_password_hashing`
- **done:** Passwords are hashed before storage
```

## Task Structure

Each task within a plan has four fields:

| Field | Description |
|-------|-------------|
| **files** | Which files to create or modify |
| **action** | What to do (be specific) |
| **verify** | Command to check success |
| **done** | Observable outcome when complete |

## Wave Ordering

Plans execute in wave order. Wave 1 runs first, then wave 2, etc. Plans within a wave can run in parallel (up to `max_wave_parallelism`).

Rules:

- Plans with no dependencies → Wave 1
- Plans depending on Wave 1 outputs → Wave 2
- A plan's wave = max(dependency waves) + 1

## Summary Files

When a plan completes execution, a summary file is created at `{plan-number}-summary.md` (e.g., `001-summary.md`). The presence of a summary file indicates the plan is complete.
