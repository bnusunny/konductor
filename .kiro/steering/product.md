# Product — Konductor v0.5.0

## Purpose
Rebuild konductor's self-improvement loop with diverse synthetic projects, granular benchmarking, and real AWS deployment verification via SAM.

## Target Users
- Konductor maintainers validating skill quality
- CI systems gating releases on benchmarks and deployment verification
- The self-improvement agent using richer signals

## Key Features
- Multiple synthetic test projects (CLI → SAM serverless)
- Per-project granular benchmark metrics (timing, test pass rates, lint, artifact quality)
- SAM deployment pipeline (deploy → verify → teardown)
- Structured self-improvement agent with per-project analysis
- Generated code quality verification (linting, test suites, coverage)
