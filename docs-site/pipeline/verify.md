# Verify

Runs tests and validates that all acceptance criteria pass.

## What Happens

1. The `konductor-verifier` agent runs the test suite
2. Checks each task's acceptance criteria
3. Validates cross-phase regression gates
4. Reports pass/fail status with details

## Outputs

Results stored in `.konductor/.results/`:

```
.results/
├── phase-01.json
└── tests/
```

## Usage

```
> verify
```

!!! warning
    Verification must pass before shipping. If tests fail, go back to execute to fix issues.
