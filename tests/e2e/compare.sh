#!/usr/bin/env bash
# Compare benchmark results against baseline
#
# Usage:
#   bash tests/e2e/compare.sh [results.toml] [baseline.toml]
#
# Defaults:
#   results:  tests/e2e/last-results.toml
#   baseline: tests/e2e/baseline.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RESULTS="${1:-$SCRIPT_DIR/last-results.toml}"
BASELINE="${2:-$SCRIPT_DIR/baseline.toml}"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

if [[ ! -f "$RESULTS" ]]; then
  echo "ERROR: Results file not found: $RESULTS"
  echo "Run the benchmark first: KONDUCTOR_E2E=1 bash tests/e2e/benchmark.sh"
  exit 1
fi

if [[ ! -f "$BASELINE" ]]; then
  echo "ERROR: Baseline file not found: $BASELINE"
  exit 1
fi

# Parse a TOML value (simple key = value)
toml_get() {
  local file="$1" key="$2"
  grep "^${key} " "$file" | head -1 | sed 's/.*= *//' | tr -d '"'
}

FAIL=0

echo -e "${CYAN}═══ Benchmark Comparison ═══${NC}"
echo ""

# Compare timing (flag if >2x baseline)
for step in init plan exec verify; do
  actual=$(toml_get "$RESULTS" "${step}_time_s")
  limit=$(toml_get "$BASELINE" "${step}_time_s")
  if [[ -z "$actual" || -z "$limit" ]]; then
    echo -e "  ⚠ ${step}_time_s: skipped (missing data)"
    continue
  fi
  if [[ $actual -gt $((limit * 2)) ]]; then
    echo -e "  ${RED}✗ ${step}: ${actual}s > ${limit}s (2x baseline)${NC}"
    FAIL=1
  else
    echo -e "  ${GREEN}✓ ${step}: ${actual}s <= ${limit}s limit${NC}"
  fi
done

# Compare plan count
actual_plans=$(toml_get "$RESULTS" "plan_count")
min_plans=$(toml_get "$BASELINE" "plan_count_min")
if [[ -n "$actual_plans" && -n "$min_plans" ]]; then
  if [[ $actual_plans -lt $min_plans ]]; then
    echo -e "  ${RED}✗ plan_count: $actual_plans < $min_plans minimum${NC}"
    FAIL=1
  else
    echo -e "  ${GREEN}✓ plan_count: $actual_plans >= $min_plans${NC}"
  fi
fi

# Compare gap count
actual_gaps=$(toml_get "$RESULTS" "gap_count")
max_gaps=$(toml_get "$BASELINE" "gap_count_max")
if [[ -n "$actual_gaps" && -n "$max_gaps" ]]; then
  if [[ $actual_gaps -gt $max_gaps ]]; then
    echo -e "  ${RED}✗ gap_count: $actual_gaps > $max_gaps maximum${NC}"
    FAIL=1
  else
    echo -e "  ${GREEN}✓ gap_count: $actual_gaps <= $max_gaps${NC}"
  fi
fi

# Compare verification status
actual_status=$(toml_get "$RESULTS" "verification_status")
expected_status=$(toml_get "$BASELINE" "verification_status")
if [[ -n "$actual_status" && -n "$expected_status" ]]; then
  if [[ "$actual_status" != "$expected_status" ]]; then
    echo -e "  ${RED}✗ verification: $actual_status (expected $expected_status)${NC}"
    FAIL=1
  else
    echo -e "  ${GREEN}✓ verification: $actual_status${NC}"
  fi
fi

echo ""
if [[ $FAIL -eq 1 ]]; then
  echo -e "${RED}REGRESSION DETECTED${NC}"
  exit 1
else
  echo -e "${GREEN}ALL CHECKS PASSED${NC}"
  exit 0
fi
