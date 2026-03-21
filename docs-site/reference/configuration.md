# Configuration Reference

Konductor reads configuration from `.konductor/config.toml`. All fields have sensible defaults — the file is optional.

## `[general]`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_model` | string | `"claude-sonnet-4"` | The AI model used by subagents |

```toml
[general]
default_model = "claude-sonnet-4"
```

## `[execution]`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_wave_parallelism` | integer | `4` | Maximum number of plans to execute simultaneously within a wave. Set to `1` for sequential execution. |

```toml
[execution]
max_wave_parallelism = 4
```

## `[git]`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_commit` | boolean | `true` | Automatically commit after each task completes |
| `branching_strategy` | string | `"none"` | Branch strategy: `"none"` (commit to current branch) or `"feature"` (create feature branches per plan) |

```toml
[git]
auto_commit = true
branching_strategy = "none"
```

## `[features]`

Feature flags to enable or disable pipeline stages.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `research` | boolean | `true` | Run ecosystem research before planning |
| `plan_checker` | boolean | `true` | Validate plans after creation (coverage, sizing, dependencies) |
| `verifier` | boolean | `true` | Run 3-level verification after execution |

```toml
[features]
research = true
plan_checker = true
verifier = true
```

!!! tip
    Set `verifier = false` to skip verification and move directly from executed to complete. Useful for rapid iteration.

## Complete Example

```toml
[general]
default_model = "claude-sonnet-4"

[execution]
max_wave_parallelism = 4

[git]
auto_commit = true
branching_strategy = "none"

[features]
research = true
plan_checker = true
verifier = true
```

## Defaults

If `.konductor/config.toml` doesn't exist, all defaults are used. If the file exists but is missing sections, the missing sections use defaults. This means a minimal config like:

```toml
[execution]
max_wave_parallelism = 1
```

...will use defaults for everything except `max_wave_parallelism`.
