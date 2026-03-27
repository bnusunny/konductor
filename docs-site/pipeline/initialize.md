# Initialize

The first phase discovers project goals and generates spec documents.

## What Happens

1. The `konductor-discoverer` agent interviews you about your project
2. It asks about goals, constraints, tech preferences, and scope
3. Generates three foundational documents

## Outputs

| File | Description |
|------|-------------|
| `project.md` | Project vision, scope, and constraints |
| `requirements.md` | Detailed functional and non-functional requirements |
| `roadmap.md` | High-level milestones and delivery phases |

## Usage

```
> initialize my project
```

## Re-Initializing After Shipping

After all phases are shipped, you can run `@k-init` again to extend the project. Konductor will ask whether you want to:

- **(a) Add new phases** — appends to the existing `roadmap.md` and `requirements.md` without overwriting prior work
- **(b) Reinitialize from scratch** — overwrites `project.md`, `requirements.md`, and `roadmap.md`

If you run `@k-init` while a project is in progress (not shipped), Konductor warns that reinitializing will overwrite the spec documents and asks for confirmation.

!!! tip
    Be specific about your requirements during the interview. The quality of generated specs directly impacts planning and execution quality.
