# Tech — Konductor v0.5.0

## Languages
- Rust (konductor-cli binary, MCP server)
- Bash (benchmark harness, deployment scripts, E2E tests)
- Python (synthetic test projects — generated code targets)

## Frameworks & Tools
- rmcp — MCP server framework
- clap — CLI argument parsing
- AWS SAM CLI — IaC deployment (`sam validate`, `sam build`, `sam deploy`, `sam delete`)
- pytest — test runner for generated Python code
- ruff — Python linter for generated code quality checks

## Infrastructure
- AWS Lambda + API Gateway (SAM-deployed synthetic projects)
- AWS DynamoDB (synthetic project #3)
- Single region (configurable, default us-east-1)

## Conventions
- TOML for state and config files
- Markdown for specs, plans, and reports
- Shell scripts for automation (benchmark, deploy, compare, improve)
- Per-project isolation in benchmark runs (temp directories)
