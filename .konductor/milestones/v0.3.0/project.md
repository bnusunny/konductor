# Konductor

Spec-driven development orchestrator for Kiro CLI. Orchestrates project planning, execution, verification, and shipping through a structured six-phase pipeline.

## Vision

Transform high-level project requirements into working software through a systematic pipeline: Initialize → Research → Plan → Execute → Verify → Ship. Each phase outputs artifacts to `.konductor/` that guide the next phase, preventing context rot and ensuring consistency.

## Target Users

- Developers using Kiro CLI who want structured, repeatable development workflows
- Teams that need autonomous agent-driven development with guardrails

## Tech Stack

- **Language:** Rust (konductor-cli binary)
- **MCP Framework:** rmcp (Model Context Protocol server)
- **Agent System:** Kiro CLI agents and skills (JSON configs + markdown skill definitions)
- **State Management:** TOML-based state machine (.konductor/state.toml)
- **Documentation:** MkDocs Material (docs-site)
- **CI/CD:** GitHub Actions (release-please, cross-platform builds, conventional commits)
- **Distribution:** GitHub Releases (Linux x86_64/arm64, macOS x86_64/arm64) + install script

## Key Components

1. **konductor-cli** — Unified Rust binary with MCP server and hook processor
2. **Agents** — JSON configs for orchestrator + 7 worker agents (discoverer, researcher, planner, plan-checker, executor, verifier, codebase-mapper)
3. **Skills** — Markdown skill definitions for each pipeline phase (init, plan, exec, verify, ship, next, status, discuss, map-codebase)
4. **Hooks** — Pre/post tool-use hooks for file tracking and destructive command blocking
5. **Install script** — Shell installer for global/local deployment
6. **Docs site** — MkDocs Material documentation hosted on GitHub Pages

## Current State (v0.2.1)

- Core MCP server with state management tools (state_get, state_transition, state_add_blocker, state_resolve_blocker, plans_list, status)
- MCP prompts for all pipeline commands (k-init, k-plan, k-exec, k-verify, k-ship, k-next, k-status, k-discuss, k-map)
- Hook system for file tracking and destructive command blocking
- Cross-platform release pipeline
- Documentation site
