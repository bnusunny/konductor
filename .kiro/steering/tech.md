# Tech — Konductor v0.7.x

## Languages
- Rust (konductor-cli binary, MCP server)
- Bash (benchmark harness, deployment scripts, E2E tests, installer)
- Python (synthetic test projects — generated code targets)
- JavaScript/Node.js (npm package wrapper, postinstall script)

## Frameworks & Tools
- rmcp — MCP server framework
- clap — CLI argument parsing
- chrono — timestamp handling
- AWS SAM CLI — IaC deployment (`sam validate`, `sam build`, `sam deploy`, `sam delete`)
- pytest — test runner for generated Python code
- ruff — Python linter for generated code quality checks
- MkDocs — documentation site generator

## Infrastructure
- AWS Lambda + API Gateway (SAM-deployed synthetic projects)
- AWS DynamoDB (synthetic project #3)
- Single region (configurable, default us-east-1)
- GitHub Actions CI (build-release, test, npm-publish, docs, release-please, conventional-commits)
- GitHub Releases (prebuilt binaries for linux-x64, linux-arm64, darwin-x64, darwin-arm64)
- npm registry (konductor package)

## Conventions
- TOML for state and config files
- Markdown for specs, plans, skills, and reports
- JSON for agent and hook configs
- Shell scripts for automation (benchmark, deploy, compare, improve, install)
- Per-project isolation in benchmark runs (temp directories)
- version.txt as single source of truth for version number
