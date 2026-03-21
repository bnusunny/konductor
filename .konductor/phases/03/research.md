# Phase 03 Research: Documentation and Polish

## Current State

### Existing docs-site pages (10 pages, ~450 lines total):
- index.md (42 lines) — landing page, good
- getting-started/installation.md (61 lines) — covers clone + install.sh, one-liner
- getting-started/quickstart.md (70 lines) — basic usage flow
- pipeline/overview.md (31 lines) — pipeline diagram
- pipeline/initialize.md (26 lines) — brief
- pipeline/plan.md (35 lines) — brief
- pipeline/execute.md (29 lines) — brief
- pipeline/verify.md (29 lines) — brief
- pipeline/ship.md (16 lines) — very brief
- architecture/how-it-works.md (82 lines) — good overview
- commands.md (29 lines) — table of commands + MCP tools

### Gaps vs. Success Criteria

1. **"All commands documented with examples"** — commands.md has a table but no examples, no argument docs, no output samples
2. **"Configuration reference is complete"** — No config reference page at all. config.toml fields undocumented.
3. **"Architecture docs reflect MCP-based design"** — how-it-works.md is decent but missing: state machine diagram, plan file format, MCP tool schemas
4. **"Troubleshooting guide covers common issues"** — No troubleshooting page exists
5. **"Hook system is documented"** — how-it-works.md mentions hooks briefly but no dedicated page with destructive command list, tracking behavior, or configuration

### Missing from mkdocs.yml nav:
- Configuration reference page
- Troubleshooting page
- Hooks page
- State machine reference
