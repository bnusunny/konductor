# TDD Patterns — Test-Driven Development for Executors

This guide covers test-driven development (TDD) practices for executor agents implementing plans with `type = "tdd"`.

## When to Use TDD

Use TDD when the plan frontmatter specifies `type = "tdd"`.

**TDD is appropriate for:**
- Pure functions with clear input/output contracts
- Business logic and validation rules
- Data transformations and parsing
- Algorithmic code (sorting, filtering, calculations)
- API endpoints with well-defined contracts

**TDD is NOT appropriate for:**
- UI components with complex interactions
- Infrastructure setup (databases, servers)
- Exploratory work with unclear requirements
- Simple CRUD operations with no business logic

## The Red-Green-Refactor Cycle

TDD follows a three-phase cycle:

### 1. Red Phase: Write Failing Tests

Write tests that express the desired behavior from the plan's `must_haves.truths`.

**Rules:**
- Tests should fail initially (no implementation exists yet)
- Tests should be specific and focused (one behavior per test)
- Test names should describe the behavior, not the implementation

**Example (Rust):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validator_requires_minimum_length() {
        let result = validate_password("short");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must be at least 8 characters");
    }

    #[test]
    fn test_password_validator_requires_uppercase() {
        let result = validate_password("nouppercase123!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password must contain at least one uppercase letter");
    }

    #[test]
    fn test_password_validator_accepts_valid_password() {
        let result = validate_password("ValidPass123!");
        assert!(result.is_ok());
    }
}
```

**Commit after red phase:**
```
test(auth-plan-2): add password validation tests (red phase)

Tests verify:
- Minimum 8 characters required
- At least one uppercase letter required
- Valid passwords accepted
```

### 2. Green Phase: Write Minimal Implementation

Write the simplest code that makes the tests pass.

**Rules:**
- Focus on making tests pass, not on perfection
- Avoid premature optimization
- Don't add features not covered by tests

**Example (Rust):**
```rust
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    Ok(())
}
```

**Verify:**
```bash
cargo test password
```

**Commit after green phase:**
```
feat(auth-plan-2): implement password validation (green phase)

Validates:
- Minimum length of 8 characters
- At least one uppercase letter
```

### 3. Refactor Phase: Improve Code Quality

Improve the code structure while keeping tests green.

**Rules:**
- Tests must continue to pass
- Improve readability, not functionality
- Extract common patterns
- Remove duplication

**Example (Rust):**
```rust
pub fn validate_password(password: &str) -> Result<(), String> {
    let validators = [
        (|p: &str| p.len() >= 8, "Password must be at least 8 characters"),
        (|p: &str| p.chars().any(|c| c.is_uppercase()), "Password must contain at least one uppercase letter"),
    ];

    for (validator, error_msg) in validators {
        if !validator(password) {
            return Err(error_msg.to_string());
        }
    }

    Ok(())
}
```

**Commit after refactor phase:**
```
refactor(auth-plan-2): extract password validators into array

No behavior change. Improves readability and makes adding new validators easier.
```

## Test Structure

### Arrange-Act-Assert (AAA) Pattern

Structure tests in three parts:

```rust
#[test]
fn test_calculate_discount_for_vip_customer() {
    // Arrange: Set up test data
    let customer = Customer::new("vip-123", CustomerTier::VIP);
    let order = Order::new(100.0);

    // Act: Execute the behavior
    let discount = calculate_discount(&customer, &order);

    // Assert: Verify the result
    assert_eq!(discount, 20.0); // VIP gets 20% discount
}
```

### Test Naming Conventions

Use descriptive names that express intent:

**Good:**
- `test_validates_email_format`
- `test_rejects_duplicate_usernames`
- `test_calculates_total_with_tax`

**Bad:**
- `test1`
- `test_user`
- `test_works`

### Test Data

Use meaningful test data that represents real scenarios:

**Good:**
```javascript
test('parses ISO date strings', () => {
  const result = parseDate('2024-03-19T10:30:00Z');
  expect(result.year).toBe(2024);
  expect(result.month).toBe(3);
  expect(result.day).toBe(19);
});
```

**Bad:**
```javascript
test('parses dates', () => {
  const result = parseDate('foo');
  expect(result).toBeDefined();
});
```

## Test-First for Observable Behaviors

Write tests for the plan's `must_haves.truths` before implementing.

**Example plan truth:** "Users cannot register with duplicate email addresses"

**Test first:**
```python
def test_registration_rejects_duplicate_email():
    # Arrange: Create first user
    user1 = User.create(email="test@example.com", password="Pass123!")

    # Act: Try to create second user with same email
    with pytest.raises(DuplicateEmailError):
        user2 = User.create(email="test@example.com", password="Pass456!")

    # Assert: Only one user exists
    assert User.count_by_email("test@example.com") == 1
```

Then implement the logic to make it pass.

## Verification Commands

Tests must be automated and run quickly (< 60 seconds).

**By language/framework:**

### Rust (cargo test)
```bash
cargo test                    # Run all tests
cargo test user::             # Run tests in user module
cargo test --lib              # Run library tests only
cargo test -- --nocapture     # Show println! output
```

### JavaScript/TypeScript (Jest/Vitest)
```bash
npm test                      # Run all tests
npm test user.test.ts         # Run specific test file
npm test -- --watch           # Run in watch mode
npm test -- --coverage        # Generate coverage report
```

### Python (pytest)
```bash
pytest                        # Run all tests
pytest tests/test_user.py     # Run specific test file
pytest -k "email"             # Run tests matching "email"
pytest -v                     # Verbose output
```

### Go (go test)
```bash
go test ./...                 # Run all tests in all packages
go test -v                    # Verbose output
go test -run TestUser         # Run specific test
go test -cover                # Show coverage
```

## Test Patterns by Stack

### REST API Testing

```javascript
// Jest + Supertest
test('POST /auth/register returns 201 with user JSON', async () => {
  const response = await request(app)
    .post('/auth/register')
    .send({ email: 'test@example.com', password: 'Pass123!' })
    .expect(201);

  expect(response.body).toHaveProperty('id');
  expect(response.body.email).toBe('test@example.com');
  expect(response.body).not.toHaveProperty('password');
});
```

### Database Testing

```python
# pytest with fixtures
@pytest.fixture
def db():
    """Create a fresh database for each test"""
    connection = create_test_db()
    yield connection
    connection.drop_all()

def test_user_saved_to_database(db):
    user = User(email="test@example.com", password_hash="hashed")
    db.save(user)

    retrieved = db.query(User).filter_by(email="test@example.com").first()
    assert retrieved is not None
    assert retrieved.email == "test@example.com"
```

### Business Logic Testing

```rust
#[test]
fn test_discount_calculation_for_bulk_orders() {
    let order = Order {
        items: vec![
            Item { price: 10.0, quantity: 100 },
            Item { price: 5.0, quantity: 200 },
        ],
    };

    let discount = calculate_bulk_discount(&order);

    assert_eq!(discount, 0.15); // 15% discount for orders > $1000
}
```

## Anti-Patterns to Avoid

### 1. Testing Implementation Details

**Bad:**
```javascript
test('uses Array.map internally', () => {
  const spy = jest.spyOn(Array.prototype, 'map');
  transform([1, 2, 3]);
  expect(spy).toHaveBeenCalled();
});
```

**Good:**
```javascript
test('transforms array elements', () => {
  const result = transform([1, 2, 3]);
  expect(result).toEqual([2, 4, 6]);
});
```

### 2. Brittle Mocks

**Bad:**
```python
def test_sends_welcome_email():
    mock_mailer = Mock()
    mock_mailer.send.return_value = True
    mock_mailer.template_id = "welcome-123"
    mock_mailer.from_address = "noreply@example.com"
    # ... 10 more mock configurations

    user.send_welcome_email(mock_mailer)
    assert mock_mailer.send.called
```

**Good:**
```python
def test_sends_welcome_email():
    mock_mailer = Mock()
    user.send_welcome_email(mock_mailer)

    # Verify behavior, not every internal detail
    mock_mailer.send.assert_called_once()
    args = mock_mailer.send.call_args[0]
    assert "Welcome" in args[0]  # Email subject
    assert user.email in args[1]  # Recipient
```

### 3. Testing Getters and Setters

**Bad:**
```java
@Test
public void testSetName() {
    User user = new User();
    user.setName("John");
    assertEquals("John", user.getName());
}
```

**Why bad:** This tests language features, not your logic. Skip trivial getters/setters.

### 4. One Giant Test

**Bad:**
```javascript
test('user registration system', () => {
  // Tests 10 different things in one test
  // If it fails, you don't know which part broke
});
```

**Good:**
```javascript
test('validates email format during registration', () => { /* ... */ });
test('hashes password during registration', () => { /* ... */ });
test('creates user record in database', () => { /* ... */ });
test('sends welcome email after registration', () => { /* ... */ });
```

## Coverage Goals

Aim for high coverage of critical paths, not 100% coverage of all code.

**Prioritize testing:**
- Business logic and validation rules
- Error handling and edge cases
- Security-critical code (auth, permissions)
- Data transformations with complex rules

**Lower priority:**
- Simple CRUD operations
- Framework glue code
- Configuration files
- Trivial getters/setters

## Summary Writing for TDD Plans

When writing your plan summary, include:

**Test Results:**
Paste the full output from your test runner showing all tests passing.

**Coverage:**
If the plan specifies coverage requirements, include coverage report.

**Red-Green-Refactor commits:**
List the commits you made for each phase of the TDD cycle.

**Example:**
```markdown
## Test Results
```
npm test

 PASS  src/validation/password.test.ts
  ✓ validates minimum length (3 ms)
  ✓ requires uppercase letter (1 ms)
  ✓ requires number (1 ms)
  ✓ requires special character (2 ms)
  ✓ accepts valid password (1 ms)

Test Suites: 1 passed, 1 total
Tests:       5 passed, 5 total
```

## Commits
1. test(auth-plan-2): add password validation tests (red phase)
2. feat(auth-plan-2): implement password validation (green phase)
3. refactor(auth-plan-2): extract validators into reusable functions
```
