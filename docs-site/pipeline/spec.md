# Spec

The first step discovers project goals and generates spec documents.

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
> /k-spec
```

`/k-init` also works as an alias.

## Re-Speccing After Shipping

After all phases are shipped, you can run `/k-spec` again to extend the project. Konductor will ask whether you want to:

- **(a) Add new phases** — appends to the existing `roadmap.md` and `requirements.md` without overwriting prior work
- **(b) Re-spec from scratch** — overwrites `project.md`, `requirements.md`, and `roadmap.md`

If you run `/k-spec` while a project is in progress (not shipped), Konductor warns that re-speccing will overwrite the spec documents and asks for confirmation.

!!! tip
    Be specific about your requirements during the interview. The quality of generated specs directly impacts design and execution quality.
