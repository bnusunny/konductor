#!/usr/bin/env bash
# E2E pipeline test for konductor
# Drives kiro-cli with the konductor agent through the full pipeline
#
# Usage:
#   KONDUCTOR_E2E=1 bash tests/e2e/run.sh
#
# Options:
#   KEEP_TEST_DIR=1    Keep the test directory after run
#   KONDUCTOR_TIMEOUT  Per-step timeout in seconds (default: 300)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"

# Gate behind env var
if [[ "${KONDUCTOR_E2E:-}" != "1" ]]; then
  echo "Skipping E2E test (set KONDUCTOR_E2E=1 to run)"
  echo "Requires: kiro-cli with authenticated session and LLM access"
  exit 0
fi

# Check prerequisites
if ! command -v kiro-cli >/dev/null 2>&1; then
  echo "ERROR: kiro-cli not found in PATH"
  exit 1
fi

echo -e "${CYAN}═══ Konductor E2E Pipeline Test ═══${NC}"

setup_test_dir
trap teardown_test_dir EXIT
cd "$TEST_DIR"

# --- Step 1: Initialize ---

log_step "Step 1: Initialize project"

# Provide the spec as context for the init conversation
cat > init_prompt.txt << 'PROMPT'
Initialize this project with Konductor. Here is the project spec:

Build a simple command-line calculator in Python.
- Single file: calc.py
- Takes 3 args: number1 operator number2
- Operators: + - x /
- Handles division by zero and invalid input
- Test file: test_calc.py

This is a small 1-phase project. Generate project.md, requirements.md, roadmap.md with a single phase.
Do NOT ask me questions — use the spec above to generate all documents.
Run the konductor-init skill.
PROMPT

konductor_chat "$(cat init_prompt.txt)" > /dev/null

assert_file_exists "$TEST_DIR/.konductor/state.toml" "state.toml"
assert_file_exists "$TEST_DIR/.konductor/project.md" "project.md"
assert_file_exists "$TEST_DIR/.konductor/requirements.md" "requirements.md"
assert_file_exists "$TEST_DIR/.konductor/roadmap.md" "roadmap.md"
assert_file_not_empty "$TEST_DIR/.konductor/project.md" "project.md non-empty"
log_step_done

# --- Step 2: Plan ---

log_step "Step 2: Plan phase"

konductor_chat "Plan phase 01 for execution. Research the ecosystem, create execution plans with tasks and acceptance criteria, and validate them. Run the konductor-plan skill." > /dev/null

assert_dir_exists "$TEST_DIR/.konductor/phases" "phases directory"
assert_min_file_count "$TEST_DIR/.konductor/phases" "*.md" 1 "at least 1 plan file"
log_step_done

# --- Step 3: Execute ---

log_step "Step 3: Execute phase"

konductor_chat "Execute the plans for the current phase. Spawn executor subagents to implement each plan following TDD workflow. Run the konductor-exec skill." > /dev/null

assert_min_file_count "$TEST_DIR/.konductor/phases" "*-summary.md" 1 "at least 1 summary file"
# Check that some code was generated
assert_min_file_count "$TEST_DIR" "*.py" 1 "at least 1 Python file created"
log_step_done

# --- Step 3b: Verify hooks fired during execution ---

log_step "Step 3b: Verify hooks"

# PostToolUse hook should have tracked written files
assert_tracking_log_exists
assert_tracking_log_contains "\.py" "tracking log recorded Python files"

# Show what the hook tracked
if [[ -f "$TEST_DIR/.konductor/.tracking/modified-files.log" ]]; then
  echo "  Hook tracked files:"
  sort -u "$TEST_DIR/.konductor/.tracking/modified-files.log" | while read -r f; do
    echo "    $f"
  done
fi

# PreToolUse hook should block destructive commands
assert_hook_blocks_destructive
log_step_done

# --- Step 4: Verify ---

log_step "Step 4: Verify phase"

konductor_chat "Verify the current phase after execution. Validate that phase goals were achieved using the 3-level verification framework. Run the konductor-verify skill." > /dev/null

assert_min_file_count "$TEST_DIR/.konductor/phases" "verification.md" 1 "verification report"
log_step_done

# --- Summary ---

echo ""
echo -e "${CYAN}═══ Artifact Summary ═══${NC}"
echo "Files in .konductor/:"
find "$TEST_DIR/.konductor" -type f | sed "s|$TEST_DIR/||" | sort
echo ""
echo "Hook tracking log:"
if [[ -f "$TEST_DIR/.konductor/.tracking/modified-files.log" ]]; then
  echo "  $(wc -l < "$TEST_DIR/.konductor/.tracking/modified-files.log") entries, $(sort -u "$TEST_DIR/.konductor/.tracking/modified-files.log" | wc -l) unique files"
else
  echo "  (not created)"
fi
echo ""
echo "Project files:"
find "$TEST_DIR" -maxdepth 2 -name "*.py" -o -name "*.txt" -o -name "*.md" | grep -v .konductor | grep -v synthetic-project | sed "s|$TEST_DIR/||" | sort || echo "  (none)"

log_summary
