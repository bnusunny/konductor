# Verification Patterns — 3-Level Verification Framework

This guide defines how to verify that phase execution actually achieved its goals, not just created files.

## The Three Levels

Verification proceeds in three increasingly rigorous levels:

### Level 1: Exists
**Question:** Is the artifact present?

**Checks:**
- File exists at expected path
- Directory structure is correct
- Database tables exist
- API endpoints are defined

**Commands:**
```bash
# File existence
[ -f src/models/user.rs ] && echo "OK" || echo "MISSING"

# Directory structure
[ -d src/routes ] && [ -d src/models ] && echo "OK"

# Database table (PostgreSQL)
psql -d mydb -c "\dt users" | grep -q users && echo "OK"

# API endpoint defined (grep for route definition)
grep -r "POST.*auth/register" src/routes/ && echo "OK"
```

**When Level 1 fails:** The most basic requirement is unmet. Execution did not complete.

### Level 2: Substantive
**Question:** Is the artifact a real implementation, not a stub?

**Checks:**
- File has meaningful content (>minimum line count)
- Not just comments or placeholders
- No TODO/FIXME/PLACEHOLDER patterns
- Contains expected code structures (functions, classes, exports)

**Thresholds:**
- Model files: >20 lines
- Route handlers: >15 lines
- Component files: >25 lines
- Test files: >30 lines
- Migration files: >10 lines

**Anti-patterns to detect:**
```rust
// STUB: Not substantive
pub struct User {
    // TODO: implement fields
}

// REAL: Substantive
pub struct User {
    pub id: Uuid,
    pub email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}
```

**Commands:**
```bash
# Line count check (exclude comments)
lines=$(grep -v '^[[:space:]]*#' src/models/user.rs | grep -v '^[[:space:]]*$' | wc -l)
[ "$lines" -gt 20 ] && echo "OK" || echo "STUB"

# TODO/FIXME detection
grep -i 'TODO\|FIXME\|PLACEHOLDER\|STUB' src/models/user.rs && echo "INCOMPLETE"

# Meaningful structure (has at least one pub fn)
grep -q 'pub fn' src/models/user.rs && echo "OK"
```

**When Level 2 fails:** Execution created files but didn't implement them properly. This is common with autonomous executors that hit errors.

### Level 3: Wired
**Question:** Is the artifact connected to the rest of the system?

**Checks:**
- Imported by other files
- Actually used (not just imported)
- Part of the call graph
- No orphaned code

**Common wiring patterns:**

**Component → API fetch:**
```javascript
// In component file
import { fetchUsers } from '../api/users'
// And uses it
const users = await fetchUsers()
```

**API → Database query:**
```python
# In API handler
from models.user import User
# And uses it
user = User.query.filter_by(email=email).first()
```

**Form → Handler:**
```rust
// In form component
<form action="/auth/register" method="POST">
// And in routes
pub async fn register(form: Form<RegisterData>) -> Response
```

**State → Render:**
```jsx
// In state file
export const userSlice = createSlice({ ... })
// And in component
import { selectUser } from '../store/userSlice'
const user = useSelector(selectUser)
```

**Commands:**
```bash
# Check if User model is imported anywhere
grep -r "use.*models::user" src/ --exclude-dir=models | wc -l

# Check if User is actually used (not just imported)
# Look for User:: or User.new or similar
grep -r "User::\|User\.new\|User\.find" src/ | wc -l

# Check bidirectional wiring (component imports API, API returns component data)
grep -q "fetchUsers" src/components/UserList.tsx && \
grep -q "export.*fetchUsers" src/api/users.ts && echo "WIRED"
```

**When Level 3 fails:** Execution implemented features but they're isolated. Integration work is missing.

## Must-Haves Derivation

Every plan should have a `must_haves` section in its frontmatter:

```toml
[must_haves]
truths = [...]
artifacts = [...]
key_links = [...]
```

Verification uses these to determine what to check.

### Option A: From Plan Frontmatter

If the plan has a `must_haves` section, use it directly.

**Example plan frontmatter:**
```toml
[must_haves]
truths = ["Users can register with email", "Passwords are hashed"]
artifacts = ["src/models/user.rs", "src/routes/auth.rs"]
key_links = ["User imported by auth routes", "bcrypt used in user.rs"]
```

**Verification:**
1. For each truth, derive a test (manual or automated)
2. For each artifact, check Levels 1-3
3. For each key_link, verify the connection exists

### Option B: From Roadmap Success Criteria

If the plan doesn't have `must_haves`, fall back to the phase's success criteria from `roadmap.md`.

**Example roadmap:**
```markdown
## Phase 01: Authentication System

Success criteria:
- Users can register with email and password
- Users can log in and receive a session token
- Passwords are securely hashed
```

**Derive must_haves:**
- **truths:** Each success criterion becomes a truth
- **artifacts:** Infer from common patterns (User model, auth routes, migrations)
- **key_links:** Infer from dependencies (auth routes use User model)

### Option C: Goal-Backward Derivation

If neither plan frontmatter nor roadmap success criteria are available, work backward from the phase goal.

**Steps:**
1. Read the phase goal from roadmap.md
2. Ask: "What must be true for this goal to be achieved?"
3. Ask: "What files must exist?"
4. Ask: "How must those files be connected?"

**Example:**
- **Goal:** "Implement user authentication"
- **Truths:** Users can register, Users can log in, Sessions are managed
- **Artifacts:** User model, auth routes, session middleware
- **Key links:** Auth routes import User, Middleware validates sessions

## Gap Structuring

When verification finds issues, structure them as "gaps" for the next planning cycle.

**Gap format:**
```toml
[[gaps]]
truth = "Users can register with email"
status = "failed"
reason = "Registration endpoint returns 500"
artifacts = ["src/routes/auth.rs"]
missing = ["Error handling", "Email validation"]
```

**Fields:**
- `truth`: Which must_have truth failed
- `status`: "failed" or "partial" or "incomplete"
- `reason`: Specific error or issue
- `artifacts`: Which files are involved
- `missing`: What needs to be added/fixed

**Write gaps to:** `.konductor/phases/{phase}/gaps.toml`

The next planning cycle can read this file and create gap-closure plans.

## Verification Commands by Language

### Rust
```bash
# Compilation check
cargo check

# Test execution
cargo test

# Import check
grep -r "use crate::models::User" src/

# Usage check
grep -r "User::" src/ | grep -v "^src/models/user.rs"
```

### Python
```bash
# Import check
python -c "from models.user import User; print('OK')"

# Test execution
pytest tests/

# Import usage
grep -r "from models.user import" src/ | wc -l

# Usage check
grep -r "User(" src/ | grep -v "models/user.py"
```

### JavaScript/TypeScript
```bash
# Compilation check (TypeScript)
npx tsc --noEmit

# Test execution
npm test

# Import check
grep -r "import.*from.*models/user" src/

# Usage check
grep -r "new User\|User\.find\|User\.create" src/
```

### Go
```bash
# Compilation check
go build ./...

# Test execution
go test ./...

# Import check
grep -r "\"myapp/models\"" . | wc -l

# Usage check
grep -r "models\.User" . | grep -v "models/user.go"
```

## Example Verification Report

A verification report should follow this structure:

```markdown
# Verification Report: Phase 01 — Authentication System

**Status:** Issues Found
**Date:** 2026-03-19
**Plans Verified:** 3

## Level 1: Exists ✓

All expected artifacts are present:
- ✓ src/models/user.rs
- ✓ src/routes/auth.rs
- ✓ src/db/migrations/001_users.sql
- ✓ tests/auth_test.rs

## Level 2: Substantive ✓

All files contain real implementations:
- ✓ User model has 45 lines, includes fields and methods
- ✓ Auth routes have 67 lines, no TODOs
- ✓ Migration file creates users table with all columns
- ✓ Tests have 52 lines with 6 test cases

## Level 3: Wired ⚠

Issues found:
- ⚠ User model is imported by auth routes (OK)
- ⚠ Auth routes use User::new (OK)
- ✗ Registration endpoint returns 500 error
- ✗ No middleware validates session tokens

## Gaps

```toml
[[gaps]]
truth = "Users can register with email"
status = "failed"
reason = "POST /auth/register returns 500: database connection error"
artifacts = ["src/routes/auth.rs", "src/db/connection.rs"]
missing = ["Database connection pool initialization", "Error handling"]

[[gaps]]
truth = "Sessions are validated on protected routes"
status = "incomplete"
reason = "Session middleware exists but is not applied to routes"
artifacts = ["src/middleware/session.rs", "src/main.rs"]
missing = ["Apply middleware to protected routes"]
```

## Next Steps

1. Run `konductor plan phase 01 --gaps` to create gap-closure plans
2. Or manually fix: Initialize DB connection pool in main.rs
3. Re-run verification after fixes
```

## Verification Automation

Verification should be automated where possible.

**Pattern: Verification script per plan**

Create `.konductor/phases/{phase}/verify-plan-{n}.sh`:
```bash
#!/bin/bash
set -e

# Level 1: Exists
[ -f src/models/user.rs ] || { echo "FAIL: user.rs missing"; exit 1; }

# Level 2: Substantive
lines=$(grep -v '^[[:space:]]*//' src/models/user.rs | grep -v '^[[:space:]]*$' | wc -l)
[ "$lines" -gt 20 ] || { echo "FAIL: user.rs is stub"; exit 1; }

# Level 3: Wired
grep -q "User::" src/routes/auth.rs || { echo "FAIL: User not used in auth"; exit 1; }

# Truth verification
cargo test auth::test_register || { echo "FAIL: registration test failed"; exit 1; }

echo "PASS: Plan 1 verified"
```

Then run: `bash .konductor/phases/{phase}/verify-plan-{n}.sh`

## Verification vs. Testing

**Testing** checks that code is correct (unit tests, integration tests).
**Verification** checks that execution achieved the phase goal (end-to-end validation).

**Testing:**
- Runs during execution (per task)
- Checks individual functions
- Passes/fails per test case
- Developer-focused

**Verification:**
- Runs after all plans execute
- Checks the entire phase
- Validates against must_haves
- User-focused (does it deliver the feature?)

**Both are necessary.** Tests catch bugs. Verification catches gaps.
