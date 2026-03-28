# Troubleshooting

## "No Konductor project found"

**Cause:** The `.konductor/` directory doesn't exist or `state.toml` is missing.

**Fix:** Initialize the project:

```
> /k-spec
```

## "Invalid transition: 'X' → 'Y'"

**Cause:** You're trying to advance to a step that isn't allowed from the current step.

**Fix:** Check your current state:

```
> /k-status
```

Then use the appropriate command. For example, if you're at `initialized`, you need to plan before executing:

```
> /k-plan 01
```

See the [State Machine Reference](reference/state-machine.md) for all valid transitions.

## "Binary not available (no release found)"

**Cause:** The npm postinstall couldn't download a prebuilt binary for your platform.

**Fix:** Build from source:

```bash
cd konductor/konductor-cli
cargo build --release
```

## Checksum Verification Failed

**Cause:** The downloaded binary doesn't match the expected SHA-256 checksum. This could indicate a corrupted download or tampered binary.

**Fix:**

1. Reinstall:
   ```bash
   npm install -g konductor
   ```

2. If the problem persists, build from source instead.

## State File Corruption

**Cause:** `state.toml` contains invalid TOML or unexpected values.

**Fix:** If the state is unrecoverable, delete and re-spec:

```bash
rm .konductor/state.toml
```

Then re-spec:

```
> /k-spec
```

!!! warning
    This resets your pipeline progress. Phase artifacts in `.konductor/phases/` are preserved.

## "Blocked" Status

**Cause:** A subagent failed during execution, and a blocker was added.

**Fix:** Check the blocker details:

```
> /k-status
```

The status output shows active blockers with their reasons. After fixing the underlying issue, say `next` to continue — the orchestrator will resolve the blocker and resume.

## Hook Binary Not Found

**Cause:** The `konductor` binary isn't on PATH.

**Fix:** Reinstall via npm:

```bash
npm install -g konductor
```

Or build and copy manually:

```bash
cd konductor/konductor-cli
cargo build --release
```

## MCP Server Won't Start

**Cause:** The `konductor mcp` command fails to initialize, usually due to a missing or incompatible binary.

**Fix:**

1. Check the binary works: `konductor --version`
2. Rebuild if needed: `cd konductor-cli && cargo build --release`
3. Ensure the agent config points to the correct path in `agents/konductor.json`
