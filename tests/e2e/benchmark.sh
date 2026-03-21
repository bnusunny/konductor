#!/usr/bin/env bash
# Benchmark harness for konductor E2E pipeline
# Wraps the E2E test with timing and metric collection
#
# Usage:
#   KONDUCTOR_E2E=1 bash tests/e2e/benchmark.sh
#
# Output:
#   Prints summary table and writes results.toml to test dir

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"

if [[ "${KONDUCTOR_E2E:-}" != "1" ]]; then
  echo "Skipping benchmark (set KONDUCTOR_E2E=1 to run)"
  exit 0
fi

if ! command -v kiro-cli >/dev/null 2>&1; then
  echo "ERROR: kiro-cli not found in PATH"
  exit 1
fi

echo -e "${CYAN}═══ Konductor Benchmark ═══${NC}"

setup_test_dir
trap teardown_test_dir EXIT
cd "$TEST_DIR"

RESULTS_FILE="$TEST_DIR/results.toml"
TOTAL_START=$(date +%s)

# --- Helper to time a step ---
time_step() {
  local step_name="$1"
  local message="$2"
  local start=$(date +%s)

  log_step "$step_name"
  local output
  output=$(konductor_chat "$message" 2>&1) || true
  local end=$(date +%s)
  local elapsed=$((end - start))

  # Extract credits from kiro-cli output
  local credits
  credits=$(echo "$output" | grep -oP 'Credits: \K[0-9.]+' | tail -1 || echo "0")

  echo "${step_name}_time_s = $elapsed" >> "$RESULTS_FILE"
  echo "${step_name}_credits = $credits" >> "$RESULTS_FILE"
  echo -e "  ${GREEN}✓ ${elapsed}s, credits: $credits${NC}"
}

# Initialize results file
cat > "$RESULTS_FILE" << EOF
# Konductor Benchmark Results
# Generated: $(date -Iseconds)
# Test dir: $TEST_DIR

EOF

# --- Run pipeline ---

time_step "init" "Initialize this project with Konductor. Build a simple Python CLI calculator: calc.py takes 3 args (number1 operator number2), supports + - x /, handles errors. Single phase project. Do NOT ask questions. Run the konductor-init skill."

time_step "plan" "Plan phase 01 for execution. Research the ecosystem, create execution plans with tasks and acceptance criteria, and validate them. Run the konductor-plan skill."

time_step "exec" "Execute the plans for the current phase. Spawn executor subagents to implement each plan following TDD workflow. Run the konductor-exec skill."

time_step "verify" "Verify the current phase after execution. Validate that phase goals were achieved using the 3-level verification framework. Run the konductor-verify skill."

TOTAL_END=$(date +%s)
TOTAL_ELAPSED=$((TOTAL_END - TOTAL_START))

# --- Collect metrics ---

plan_count=$(find "$TEST_DIR/.konductor/phases" -name "*.md" ! -name "*-summary.md" ! -name "verification.md" ! -name "research.md" 2>/dev/null | wc -l)
summary_count=$(find "$TEST_DIR/.konductor/phases" -name "*-summary.md" 2>/dev/null | wc -l)
gap_count=0
verification_status="unknown"

if [[ -f $(find "$TEST_DIR/.konductor/phases" -name "verification.md" 2>/dev/null | head -1) ]]; then
  local_verification=$(find "$TEST_DIR/.konductor/phases" -name "verification.md" | head -1)
  if grep -qi "gaps\|issues" "$local_verification" 2>/dev/null; then
    gap_count=$(grep -ci "gap\|issue\|fail" "$local_verification" 2>/dev/null || echo "0")
  fi
  if grep -qi "status.*ok\|all.*pass" "$local_verification" 2>/dev/null; then
    verification_status="ok"
  else
    verification_status="issues-found"
  fi
fi

python_files=$(find "$TEST_DIR" -maxdepth 3 -name "*.py" ! -path "*/.konductor/*" 2>/dev/null | wc -l)

cat >> "$RESULTS_FILE" << EOF

# Summary
total_time_s = $TOTAL_ELAPSED
plan_count = $plan_count
summary_count = $summary_count
gap_count = $gap_count
verification_status = "$verification_status"
python_files_created = $python_files
EOF

# --- Print summary ---

echo -e "\n${CYAN}═══ Benchmark Results ═══${NC}"
echo ""
cat "$RESULTS_FILE"
echo ""
echo -e "${CYAN}Results written to: $RESULTS_FILE${NC}"

# Copy results to a persistent location
cp "$RESULTS_FILE" "$SCRIPT_DIR/last-results.toml" 2>/dev/null || true
echo -e "Also saved to: $SCRIPT_DIR/last-results.toml"
