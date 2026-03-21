#!/usr/bin/env bash
# Compare per-project benchmark results against baselines
#
# Usage:
#   bash tests/e2e/compare.sh [--verbose]
#
# Reads:
#   tests/e2e/results/{project}.toml
#   tests/e2e/baselines/{project}.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
BASELINES_DIR="$SCRIPT_DIR/baselines"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

VERBOSE=false
[[ "${1:-}" == "--verbose" ]] && VERBOSE=true

# Parse a TOML value (simple key = value, handles quoted strings and booleans)
toml_get() {
  local file="$1" key="$2"
  grep "^${key} " "$file" 2>/dev/null | head -1 | sed 's/.*= *//' | tr -d '"'
}

TOTAL_FAIL=0
TOTAL_PASS=0
PROJECT_SUMMARIES=()

echo -e "${CYAN}═══ Benchmark Comparison ═══${NC}"
echo ""

# Iterate over all result files
for result_file in "$RESULTS_DIR"/*.toml; do
  [[ -f "$result_file" ]] || continue

  project=$(basename "$result_file" .toml)
  baseline_file="$BASELINES_DIR/$project.toml"

  if [[ ! -f "$baseline_file" ]]; then
    echo -e "${YELLOW}⚠ $project: no baseline found, skipping${NC}"
    continue
  fi

  PROJ_FAIL=0
  PROJ_PASS=0
  FAILURES=()

  echo -e "${CYAN}── $project ──${NC}"

  # --- Timing checks (1.5x threshold) ---
  for step in init plan exec verify; do
    actual=$(toml_get "$result_file" "${step}_s")
    limit=$(toml_get "$baseline_file" "${step}_s")
    if [[ -z "$actual" || -z "$limit" ]]; then
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${YELLOW}⚠ ${step}_s: skipped (missing data)${NC}"
      continue
    fi
    threshold=$(( (limit * 3 + 1) / 2 ))  # 1.5x, integer math
    if [[ $actual -gt $threshold ]]; then
      echo -e "  ${RED}✗ ${step}: ${actual}s > ${threshold}s (1.5x of ${limit}s)${NC}"
      FAILURES+=("${step} timing: ${actual}s > ${threshold}s")
      PROJ_FAIL=$((PROJ_FAIL + 1))
    else
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${GREEN}✓ ${step}: ${actual}s <= ${threshold}s${NC}"
      PROJ_PASS=$((PROJ_PASS + 1))
    fi
  done

  # --- Artifact checks (minimum counts) ---
  for metric in plan_count python_files test_files; do
    actual=$(toml_get "$result_file" "$metric")
    min=$(toml_get "$baseline_file" "${metric}_min")
    if [[ -z "$actual" || -z "$min" ]]; then
      continue
    fi
    if [[ $actual -lt $min ]]; then
      echo -e "  ${RED}✗ ${metric}: $actual < $min minimum${NC}"
      FAILURES+=("${metric}: $actual < $min")
      PROJ_FAIL=$((PROJ_FAIL + 1))
    else
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${GREEN}✓ ${metric}: $actual >= $min${NC}"
      PROJ_PASS=$((PROJ_PASS + 1))
    fi
  done

  # --- Quality checks (strict pass/fail) ---
  for metric in lint_pass tests_pass; do
    actual=$(toml_get "$result_file" "$metric")
    expected=$(toml_get "$baseline_file" "$metric")
    if [[ -z "$actual" || -z "$expected" ]]; then
      continue
    fi
    if [[ "$actual" == "skipped" ]]; then
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${YELLOW}⚠ ${metric}: skipped (tool not available)${NC}"
      continue
    fi
    if [[ "$actual" != "$expected" ]]; then
      echo -e "  ${RED}✗ ${metric}: $actual (expected $expected)${NC}"
      FAILURES+=("${metric}: $actual != $expected")
      PROJ_FAIL=$((PROJ_FAIL + 1))
    else
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${GREEN}✓ ${metric}: $actual${NC}"
      PROJ_PASS=$((PROJ_PASS + 1))
    fi
  done

  # --- Test count check ---
  actual_tests=$(toml_get "$result_file" "tests_total")
  min_tests=$(toml_get "$baseline_file" "tests_total_min")
  if [[ -n "$actual_tests" && -n "$min_tests" && "$actual_tests" != "-1" ]]; then
    if [[ $actual_tests -lt $min_tests ]]; then
      echo -e "  ${RED}✗ tests_total: $actual_tests < $min_tests minimum${NC}"
      FAILURES+=("tests_total: $actual_tests < $min_tests")
      PROJ_FAIL=$((PROJ_FAIL + 1))
    else
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${GREEN}✓ tests_total: $actual_tests >= $min_tests${NC}"
      PROJ_PASS=$((PROJ_PASS + 1))
    fi
  fi

  # --- SAM checks ---
  expected_sam=$(toml_get "$baseline_file" "sam_template")
  if [[ "$expected_sam" == "true" ]]; then
    actual_sam=$(toml_get "$result_file" "sam_template")
    if [[ "$actual_sam" != "true" ]]; then
      echo -e "  ${RED}✗ sam_template: missing (expected)${NC}"
      FAILURES+=("sam_template: missing")
      PROJ_FAIL=$((PROJ_FAIL + 1))
    else
      [[ "$VERBOSE" == "true" ]] && echo -e "  ${GREEN}✓ sam_template: present${NC}"
      PROJ_PASS=$((PROJ_PASS + 1))

      actual_valid=$(toml_get "$result_file" "sam_valid")
      expected_valid=$(toml_get "$baseline_file" "sam_valid")
      if [[ -n "$expected_valid" && "$actual_valid" != "skipped" && "$actual_valid" != "$expected_valid" ]]; then
        echo -e "  ${RED}✗ sam_valid: $actual_valid (expected $expected_valid)${NC}"
        FAILURES+=("sam_valid: $actual_valid != $expected_valid")
        PROJ_FAIL=$((PROJ_FAIL + 1))
      elif [[ "$actual_valid" != "skipped" ]]; then
        [[ "$VERBOSE" == "true" ]] && echo -e "  ${GREEN}✓ sam_valid: $actual_valid${NC}"
        PROJ_PASS=$((PROJ_PASS + 1))
      fi
    fi
  fi

  # --- Project summary ---
  TOTAL_FAIL=$((TOTAL_FAIL + PROJ_FAIL))
  TOTAL_PASS=$((TOTAL_PASS + PROJ_PASS))

  if [[ $PROJ_FAIL -eq 0 ]]; then
    PROJECT_SUMMARIES+=("${GREEN}✓ $project: $PROJ_PASS passed${NC}")
  else
    failure_reasons=$(printf '%s; ' "${FAILURES[@]}")
    PROJECT_SUMMARIES+=("${RED}✗ $project: $PROJ_FAIL failed ($failure_reasons)${NC}")
  fi
  echo ""
done

# --- Overall matrix ---

echo -e "${CYAN}═══ Summary Matrix ═══${NC}"
for s in "${PROJECT_SUMMARIES[@]}"; do
  echo -e "  $s"
done
echo ""
echo -e "  Total: ${GREEN}$TOTAL_PASS passed${NC}, ${RED}$TOTAL_FAIL failed${NC}"
echo ""

if [[ $TOTAL_FAIL -gt 0 ]]; then
  echo -e "${RED}REGRESSION DETECTED${NC}"
  exit 1
else
  echo -e "${GREEN}ALL PROJECTS PASSED${NC}"
  exit 0
fi
