---
name: konductor-map-codebase
description: Analyze and map an existing codebase. Use when the user says map codebase, analyze codebase, understand this project, scan project, or audit code structure.
---

# Konductor Map Codebase — Codebase Analysis

You are the Konductor orchestrator. Analyze an existing codebase to understand its structure, tech stack, patterns, and conventions.

## Step 1: Verify Target Directory

Check if a project root exists. By default, analyze the current working directory.

If the user specified a different directory, verify it exists:
```bash
ls {target_directory}
```

If it doesn't exist, tell the user:
> "Directory '{target_directory}' not found. Please provide a valid path."

Then stop.

## Step 2: Quick Scan for Indicators

Check for common project files to determine if this is a real codebase:
```bash
ls package.json Cargo.toml go.mod pyproject.toml requirements.txt composer.json build.gradle pom.xml 2>/dev/null | wc -l
```

If no project files are found, tell the user:
> "No recognizable project structure found. This doesn't appear to be a codebase with standard project files."

Ask if they want to proceed anyway.

## Step 3: Spawn Codebase Mapper Agent

Spawn the **konductor-codebase-mapper** subagent with these instructions:

**Analysis scope:**
1. **File structure:**
   - Directory layout (top-level directories and their purpose)
   - Key directories (source code, tests, config, docs, build artifacts)
   - File organization patterns

2. **Tech stack:**
   - Programming languages (primary and secondary)
   - Frameworks and libraries (extracted from dependency files)
   - Package managers and build tools
   - Runtime versions (Node, Python, Go, Rust, etc. from config files)

3. **Architecture and patterns:**
   - Architecture style (monolithic, microservices, modular, etc.)
   - Design patterns observed (MVC, layered, event-driven, etc.)
   - Naming conventions (file names, directories, modules)
   - Code organization (how features are structured)

4. **Testing setup:**
   - Test framework(s) used
   - Test locations (unit, integration, e2e)
   - Coverage tooling (if present)
   - Test naming and organization patterns

5. **Integrations and external dependencies:**
   - APIs consumed (look for HTTP clients, API keys in configs)
   - Databases (connection strings, ORM/query tools)
   - External services (cloud providers, SaaS integrations)
   - CI/CD configuration (GitHub Actions, CircleCI, Jenkins, etc.)

**Output requirements:**
The mapper should write TWO files:

1. `.kiro/steering/structure.md` — File and directory structure analysis
2. `.kiro/steering/tech.md` — Tech stack, patterns, testing, and integrations

Create the directory first:
```bash
mkdir -p .kiro/steering
```

**Mapper behavior:**
- Read relevant files (package.json, requirements.txt, README, etc.)
- Use Grep/Glob to scan for patterns
- Do NOT modify any project files
- Focus on facts and observations, not recommendations
- Complete the analysis within reasonable time (don't analyze every single file)

Provide the mapper with:
- The target directory (absolute path)
- The output file paths (absolute paths)
- Reference to `references/codebase-analysis.md` if it exists

Wait for the mapper to complete.

## Step 4: Verify Mapper Output

Check that both output files were created:
```bash
ls .kiro/steering/structure.md .kiro/steering/tech.md 2>/dev/null
```

If either file is missing:
- Report which file is missing
- Show any error messages from the mapper
- Tell the user: "Codebase mapping incomplete. Review the mapper's output for errors."
- Stop

## Step 5: Read and Summarize Results

Read both files:
- `.kiro/steering/structure.md`
- `.kiro/steering/tech.md`

Extract key findings:
- Primary programming language(s)
- Main framework(s)
- Architecture style
- Number of top-level directories
- Test coverage status (if determinable)

## Step 6: Report to User

Present a summary:

```
# Codebase Mapped

## Tech Stack
{primary languages and frameworks}

## Structure
{brief description of directory layout}
- {key directories}

## Testing
{test framework and location, or "No tests found" if none}

## Integrations
{external services and APIs, or "None detected" if none}

---

Detailed analysis saved to:
- Structure: `.kiro/steering/structure.md`
- Tech stack: `.kiro/steering/tech.md`

These files will be used as context for planning and execution.
```

## Step 7: Optional State Update

If a Konductor project exists (`.konductor/state.toml`), update the metrics:
- Increment `[metrics].total_agent_sessions`
- Update `[metrics].last_activity`

Write a result file at `.konductor/.results/map-codebase.toml`:

```toml
step = "map"
timestamp = {current ISO timestamp}
languages = ["{lang1}", "{lang2}", ...]
primary_framework = "{framework}"
```

If no Konductor project exists, skip this step (mapping can be done standalone).

## Error Handling

**Mapper agent fails:**
If the mapper crashes or produces errors:
1. Show the error to the user
2. Suggest running the analysis on a smaller scope or manually reviewing the codebase
3. Stop

**Invalid directory:**
If the target directory is empty or inaccessible:
1. Report the issue
2. Stop

**Partial results:**
If the mapper creates only one file:
1. Report which file was created and which is missing
2. Display the content of the file that was created
3. Suggest the user review it and potentially fill in the gaps manually

**No recognizable project:**
If the mapper reports that it cannot identify a tech stack:
1. Show what it did find (if anything)
2. Ask the user if they want to provide manual context
3. If yes, guide them to write `.kiro/steering/tech.md` manually
