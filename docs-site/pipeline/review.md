# Review

Reviews the phase architecture and plans before execution begins.

## What Happens

1. The `konductor-design-reviewer` agent receives the phase `design.md`, all plan files, requirements, roadmap, and project constraints
2. It evaluates the design against five criteria:
   - **Architectural soundness** — do components and interactions make sense?
   - **Feasibility** — are proposed interfaces and approaches realistic given the codebase?
   - **Security** — hardcoded secrets, injection risks, missing input validation, unsafe patterns
   - **Cross-plan consistency** — do shared interfaces match across dependent plans?
   - **Requirement coverage** — every REQ-XX for this phase is addressed
3. Writes a structured review with a verdict

## Output

| File | Description |
|------|-------------|
| `.konductor/phases/{phase}/review.md` | Verdict (pass/revise/reject), issues with severity, and strengths |

### Verdicts

| Verdict | Meaning |
|---------|---------|
| `pass` | Design is sound, proceed to execution |
| `revise` | Issues found, plans are updated and re-reviewed |
| `reject` | Fundamental problems, requires redesign |

## Configuration

Review runs when enabled:

```toml
[features]
design_review = true  # default
```

Set `features.design_review = false` in `config.toml` to skip review.

## Usage

```
> /k-review 01
```
