#!/usr/bin/env bash
# Self-improvement loop for konductor
# Analyzes benchmark results, improves skills, validates improvements
#
# Usage:
#   KONDUCTOR_E2E=1 bash tests/e2e/improve.sh
#
# Options:
#   MAX_ITERATIONS   Max improvement iterations (default: 3)
#   KEEP_TEST_DIR=1  Keep test directories

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib.sh"

if [[ "${KONDUCTOR_E2E:-}" != "1" ]]; then
  echo "Skipping improvement loop (set KONDUCTOR_E2E=1 to run)"
  exit 0
fi

if ! command -v kiro-cli >/dev/null 2>&1; then
  echo "ERROR: kiro-cli not found in PATH"
  exit 1
fi

MAX_ITERATIONS="${MAX_ITERATIONS:-3}"

echo -e "${CYAN}═══ Konductor Self-Improvement Loop ═══${NC}"
echo -e "Max iterations: $MAX_ITERATIONS"
echo ""

# --- Step 1: Run initial benchmark ---

log_step "Initial benchmark"
cd "$REPO_ROOT"
KONDUCTOR_E2E=1 bash "$SCRIPT_DIR/benchmark.sh"
cp "$SCRIPT_DIR/last-results.toml" "$SCRIPT_DIR/initial-results.toml" 2>/dev/null || true
log_step_done

# --- Step 2: Improvement iterations ---

for i in $(seq 1 "$MAX_ITERATIONS"); do
  echo -e "\n${CYAN}═══ Iteration $i / $MAX_ITERATIONS ═══${NC}"

  # Check if there's room to improve
  if bash "$SCRIPT_DIR/compare.sh" 2>/dev/null; then
    echo -e "${GREEN}All metrics within baseline — no improvement needed${NC}"
    break
  fi

  # Stash current skills for revert
  log_step "Saving current skills (git stash)"
  cd "$REPO_ROOT"
  git stash push -m "pre-improve-$i" -- skills/ 2>/dev/null || true
  log_step_done

  # Ask agent to analyze and improve
  log_step "Analyzing results and improving skills"
  RESULTS_CONTENT=$(cat "$SCRIPT_DIR/last-results.toml" 2>/dev/null || echo "no results")
  BASELINE_CONTENT=$(cat "$SCRIPT_DIR/baseline.toml" 2>/dev/null || echo "no baseline")

  IMPROVE_PROMPT="You are improving konductor's skills based on benchmark results.

Current benchmark results:
$RESULTS_CONTENT

Baseline thresholds:
$BASELINE_CONTENT

Analyze the results. Identify the worst-performing step (slowest time, highest gap count, or verification failures).

Then read the corresponding skill file at skills/konductor-{step}/SKILL.md (where step is one of: init, plan, exec, verify, ship, next, status).

Generate an improved version of that SKILL.md that:
- Makes instructions more specific and actionable
- Reduces ambiguity that could cause the agent to waste time
- Adds guardrails to prevent common failure modes
- Keeps the same overall structure

Write the improved version directly to the SKILL.md file. Only modify ONE skill file per iteration.

Tell me which skill you improved and what you changed."

  cd "$REPO_ROOT"
  IMPROVE_OUTPUT=$(konductor_chat "$IMPROVE_PROMPT" 2>&1) || true
  echo "$IMPROVE_OUTPUT" | tail -5
  log_step_done

  # Check if any skill files actually changed
  if git diff --quiet -- skills/; then
    echo -e "${YELLOW}No skill files were modified — skipping benchmark${NC}"
    git stash pop 2>/dev/null || true
    continue
  fi

  echo -e "  Modified files:"
  git diff --name-only -- skills/ | sed 's/^/    /'

  # Re-run benchmark with improved skills
  log_step "Re-running benchmark with improved skills"
  KONDUCTOR_E2E=1 bash "$SCRIPT_DIR/benchmark.sh"
  log_step_done

  # Compare against baseline
  log_step "Checking for regression"
  if bash "$SCRIPT_DIR/compare.sh"; then
    echo -e "${GREEN}  ✓ Improvement accepted — metrics within baseline${NC}"
    # Drop the stash since we're keeping the change
    git stash drop 2>/dev/null || true
    # Update baseline with improved values
    cp "$SCRIPT_DIR/last-results.toml" "$SCRIPT_DIR/baseline-after-iter-$i.toml" 2>/dev/null || true
  else
    echo -e "${RED}  ✗ Regression detected — reverting${NC}"
    git checkout -- skills/
    git stash pop 2>/dev/null || true
    # Restore previous results
    cp "$SCRIPT_DIR/initial-results.toml" "$SCRIPT_DIR/last-results.toml" 2>/dev/null || true
  fi
  log_step_done
done

# --- Summary ---

echo -e "\n${CYAN}═══ Improvement Summary ═══${NC}"
echo ""
echo "Initial results:"
cat "$SCRIPT_DIR/initial-results.toml" 2>/dev/null | grep -E "time_s|plan_count|gap_count|verification" || echo "  (not available)"
echo ""
echo "Final results:"
cat "$SCRIPT_DIR/last-results.toml" 2>/dev/null | grep -E "time_s|plan_count|gap_count|verification" || echo "  (not available)"
echo ""

# Show what skills changed (if any)
cd "$REPO_ROOT"
if git diff --quiet HEAD -- skills/; then
  echo -e "${YELLOW}No net skill changes after improvement loop${NC}"
else
  echo -e "${GREEN}Skills modified:${NC}"
  git diff --stat HEAD -- skills/
fi
