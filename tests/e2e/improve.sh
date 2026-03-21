#!/usr/bin/env bash
# Self-improvement loop for konductor
# Analyzes per-project benchmark results with structured failure analysis,
# maps failures to specific skills, improves them, and validates.
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
echo "Max iterations: $MAX_ITERATIONS"
echo ""

# --- Helpers ---

# Read a TOML value from a file
toml_val() {
  local file="$1" key="$2"
  grep "^${key} " "$file" 2>/dev/null | head -1 | sed 's/.*= *//' | tr -d '"'
}

# Analyze a single project's results and return failure categories.
# Outputs lines of "category:detail" to stdout.
analyze_project() {
  local result_file="$1"
  local baseline_file="$2"

  local tests_pass lint_pass sam_valid test_files python_files
  tests_pass=$(toml_val "$result_file" "tests_pass")
  lint_pass=$(toml_val "$result_file" "lint_pass")
  sam_valid=$(toml_val "$result_file" "sam_valid")
  test_files=$(toml_val "$result_file" "test_files")
  python_files=$(toml_val "$result_file" "python_files")

  # Deployment metrics
  local deploy_success verify_success teardown_success
  deploy_success=$(toml_val "$result_file" "deploy_success")
  verify_success=$(toml_val "$result_file" "verify_success")
  teardown_success=$(toml_val "$result_file" "teardown_success")

  # Quality failures
  [[ "$tests_pass" == "false" ]] && echo "quality:tests_fail"
  [[ "$lint_pass" == "false" ]] && echo "quality:lint_fail"
  [[ "${test_files:-0}" == "0" ]] && echo "artifact:no_test_files"
  [[ "${python_files:-0}" == "0" ]] && echo "artifact:no_python_files"

  # SAM failures
  [[ "$sam_valid" == "false" ]] && echo "sam:template_invalid"

  # Deployment failures
  [[ "$deploy_success" == "false" ]] && echo "deploy:deploy_fail"
  [[ "$verify_success" == "false" ]] && echo "deploy:verify_fail"
  [[ "$teardown_success" == "false" ]] && echo "deploy:teardown_fail"

  # Timing regressions (1.5x baseline)
  for step in init plan exec verify; do
    local actual limit
    actual=$(toml_val "$result_file" "${step}_s")
    limit=$(toml_val "$baseline_file" "${step}_s")
    if [[ -n "$actual" && -n "$limit" ]]; then
      local threshold=$(( (limit * 3 + 1) / 2 ))
      [[ $actual -gt $threshold ]] && echo "timing:${step}_slow"
    fi
  done
}

# Map failure categories to the skill most likely to fix them.
# Returns the skill name.
map_failure_to_skill() {
  local failures="$1"

  # Count by category
  local quality_count artifact_count sam_count deploy_count timing_count
  quality_count=$(echo "$failures" | grep -c "^quality:" || echo 0)
  artifact_count=$(echo "$failures" | grep -c "^artifact:" || echo 0)
  sam_count=$(echo "$failures" | grep -c "^sam:" || echo 0)
  deploy_count=$(echo "$failures" | grep -c "^deploy:" || echo 0)
  timing_count=$(echo "$failures" | grep -c "^timing:" || echo 0)

  # No artifacts at all → plan skill (plans didn't produce enough tasks)
  if [[ $artifact_count -gt 0 ]] && echo "$failures" | grep -q "no_python_files"; then
    echo "plan"
    return
  fi

  # SAM template issues could be plan (didn't ask for template) or exec (bad template)
  if [[ $sam_count -gt 0 ]]; then
    # If we also have no_test_files, it's likely a plan issue
    if [[ $artifact_count -gt 0 ]]; then
      echo "plan"
    else
      echo "exec"
    fi
    return
  fi

  # Quality/deployment failures → exec skill (code generation quality)
  if [[ $((quality_count + deploy_count)) -gt 0 ]]; then
    echo "exec"
    return
  fi

  # Timing regressions → check which step is slowest
  if [[ $timing_count -gt 0 ]]; then
    local slowest_step
    slowest_step=$(echo "$failures" | grep "^timing:" | sed 's/timing://' | sed 's/_slow//' | head -1)
    case "$slowest_step" in
      init) echo "init" ;;
      plan) echo "plan" ;;
      exec) echo "exec" ;;
      verify) echo "verify" ;;
      *) echo "exec" ;;
    esac
    return
  fi

  echo "exec"  # default
}

# --- Step 1: Run initial benchmark ---

log_step "Initial benchmark (all projects)"
cd "$REPO_ROOT"
KONDUCTOR_E2E=1 bash "$SCRIPT_DIR/benchmark.sh"

# Save initial results for before/after comparison
rm -rf "$SCRIPT_DIR/results-initial" 2>/dev/null || true
cp -r "$SCRIPT_DIR/results" "$SCRIPT_DIR/results-initial"
log_step_done

# --- Step 2: Improvement iterations ---

for i in $(seq 1 "$MAX_ITERATIONS"); do
  echo -e "\n${CYAN}═══ Iteration $i / $MAX_ITERATIONS ═══${NC}"

  # Check if all projects pass
  if bash "$SCRIPT_DIR/compare.sh" 2>/dev/null; then
    echo -e "${GREEN}All projects within baseline — no improvement needed${NC}"
    break
  fi

  # --- Structured failure analysis ---

  log_step "Analyzing per-project failures"

  WORST_PROJECT=""
  WORST_FAILURES=""
  WORST_FAILURE_COUNT=0
  ANALYSIS_REPORT=""

  for result_file in "$SCRIPT_DIR/results"/*.toml; do
    [[ -f "$result_file" ]] || continue
    project=$(basename "$result_file" .toml)
    baseline_file="$SCRIPT_DIR/baselines/$project.toml"
    [[ -f "$baseline_file" ]] || continue

    failures=$(analyze_project "$result_file" "$baseline_file")
    failure_count=$(echo "$failures" | grep -c "." || echo 0)

    if [[ -n "$failures" ]]; then
      ANALYSIS_REPORT="${ANALYSIS_REPORT}
Project: $project ($failure_count failures)
$(echo "$failures" | sed 's/^/  - /')
"
    fi

    if [[ $failure_count -gt $WORST_FAILURE_COUNT ]]; then
      WORST_PROJECT="$project"
      WORST_FAILURES="$failures"
      WORST_FAILURE_COUNT=$failure_count
    fi
  done

  if [[ -z "$WORST_PROJECT" ]]; then
    echo -e "${YELLOW}No clear failure mode identified — stopping${NC}"
    break
  fi

  # Map failures to skill
  SKILL_TO_IMPROVE=$(map_failure_to_skill "$WORST_FAILURES")
  FAILURE_SUMMARY=$(echo "$WORST_FAILURES" | tr '\n' ' ')

  echo -e "  ${CYAN}Analysis Report:${NC}"
  echo "$ANALYSIS_REPORT" | sed '/^$/d'
  echo ""
  echo -e "  Worst project: ${RED}$WORST_PROJECT${NC} ($WORST_FAILURE_COUNT failures)"
  echo -e "  Target skill:  ${YELLOW}konductor-$SKILL_TO_IMPROVE${NC}"
  echo -e "  Failures:      ${RED}$FAILURE_SUMMARY${NC}"
  log_step_done

  # --- Stash current skills for revert ---

  log_step "Saving current skills (git stash)"
  cd "$REPO_ROOT"
  git stash push -m "pre-improve-$i" -- skills/ 2>/dev/null || true
  log_step_done

  # --- Generate targeted improvement ---

  log_step "Improving skill: konductor-$SKILL_TO_IMPROVE"

  RESULTS_CONTENT=$(cat "$SCRIPT_DIR/results/$WORST_PROJECT.toml" 2>/dev/null || echo "no results")
  BASELINE_CONTENT=$(cat "$SCRIPT_DIR/baselines/$WORST_PROJECT.toml" 2>/dev/null || echo "no baseline")
  SKILL_CONTENT=$(cat "$REPO_ROOT/skills/konductor-$SKILL_TO_IMPROVE/SKILL.md" 2>/dev/null || echo "no skill")
  SPEC_CONTENT=$(cat "$SCRIPT_DIR/synthetic-projects/$WORST_PROJECT/spec.md" 2>/dev/null || echo "no spec")

  # Build failure-specific instructions
  FAILURE_INSTRUCTIONS=""
  echo "$WORST_FAILURES" | grep -q "tests_fail" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- Generated tests are failing. Add explicit instructions to run pytest after writing tests and fix failures before moving on."
  echo "$WORST_FAILURES" | grep -q "lint_fail" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- Generated code has lint errors. Add instruction to run 'ruff check' and fix issues before completing each task."
  echo "$WORST_FAILURES" | grep -q "no_test_files" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- No test files were generated. Add explicit step requiring test file creation with minimum test count from spec."
  echo "$WORST_FAILURES" | grep -q "template_invalid" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- SAM template is invalid. Add explicit instructions for template.yaml structure: AWSTemplateFormatVersion, Transform, Resources with AWS::Serverless::Function, Outputs."
  echo "$WORST_FAILURES" | grep -q "deploy_fail" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- Deployment failed. Add instructions to validate SAM template with 'sam validate' and ensure Lambda handler path matches template Handler field."
  echo "$WORST_FAILURES" | grep -q "verify_fail" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- Deployed API returned wrong responses. Add instructions to test handler locally with sample events before considering task complete."
  echo "$WORST_FAILURES" | grep -q "no_python_files" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- No Python files were generated. Plans may be too vague. Add explicit file creation requirements with exact filenames."
  echo "$WORST_FAILURES" | grep -q "_slow" && \
    FAILURE_INSTRUCTIONS="${FAILURE_INSTRUCTIONS}
- Step is too slow. Reduce ambiguity in instructions to prevent the agent from exploring unnecessary paths."

  IMPROVE_PROMPT="You are improving konductor's konductor-$SKILL_TO_IMPROVE skill based on structured benchmark analysis.

## Failure Analysis
Project: $WORST_PROJECT
Failure count: $WORST_FAILURE_COUNT
Failures: $FAILURE_SUMMARY

## Specific Issues to Address
$FAILURE_INSTRUCTIONS

## Context

Project spec:
$SPEC_CONTENT

Benchmark results:
$RESULTS_CONTENT

Baseline thresholds:
$BASELINE_CONTENT

Current skill:
$SKILL_CONTENT

## Instructions
Modify skills/konductor-$SKILL_TO_IMPROVE/SKILL.md to address the specific failures listed above.
Keep the same overall structure and frontmatter.
Only modify this ONE skill file.
Tell me exactly what you changed and why."

  cd "$REPO_ROOT"
  IMPROVE_OUTPUT=$(konductor_chat "$IMPROVE_PROMPT" 2>&1) || true
  echo "$IMPROVE_OUTPUT" | tail -5
  log_step_done

  # --- Check if skill files changed ---

  if git diff --quiet -- skills/; then
    echo -e "${YELLOW}No skill files were modified — skipping benchmark${NC}"
    git stash pop 2>/dev/null || true
    continue
  fi

  echo "  Modified files:"
  git diff --name-only -- skills/ | sed 's/^/    /'

  # --- Re-run benchmark for worst project only ---

  log_step "Re-running benchmark for $WORST_PROJECT"
  KONDUCTOR_E2E=1 bash "$SCRIPT_DIR/benchmark.sh" --project "$WORST_PROJECT"
  log_step_done

  # --- Check for regression ---

  log_step "Checking for regression"
  if bash "$SCRIPT_DIR/compare.sh"; then
    echo -e "${GREEN}  ✓ Improvement accepted — all projects pass${NC}"
    git stash drop 2>/dev/null || true
  else
    echo -e "${RED}  ✗ Regression detected — reverting${NC}"
    git checkout -- skills/
    git stash pop 2>/dev/null || true
    # Restore previous results for the reverted project
    if [[ -f "$SCRIPT_DIR/results-initial/$WORST_PROJECT.toml" ]]; then
      cp "$SCRIPT_DIR/results-initial/$WORST_PROJECT.toml" "$SCRIPT_DIR/results/$WORST_PROJECT.toml"
    fi
  fi
  log_step_done
done

# --- Before/After Summary ---

echo -e "\n${CYAN}═══ Improvement Summary ═══${NC}"
echo ""
printf "%-25s  %-12s %-12s %-12s %-12s %-12s\n" "Project" "Tests" "Lint" "SAM" "Deploy" "Timing"
printf "%-25s  %-12s %-12s %-12s %-12s %-12s\n" "-------" "-----" "----" "---" "------" "------"

for f in "$SCRIPT_DIR/results"/*.toml; do
  [[ -f "$f" ]] || continue
  p=$(basename "$f" .toml)
  init_f="$SCRIPT_DIR/results-initial/$p.toml"

  # Current values
  c_tests=$(toml_val "$f" "tests_pass")
  c_lint=$(toml_val "$f" "lint_pass")
  c_sam=$(toml_val "$f" "sam_valid")
  c_deploy=$(toml_val "$f" "deploy_success")
  c_time=$(toml_val "$f" "total_s")

  # Initial values
  i_tests=$(toml_val "$init_f" "tests_pass" 2>/dev/null)
  i_lint=$(toml_val "$init_f" "lint_pass" 2>/dev/null)
  i_sam=$(toml_val "$init_f" "sam_valid" 2>/dev/null)
  i_deploy=$(toml_val "$init_f" "deploy_success" 2>/dev/null)
  i_time=$(toml_val "$init_f" "total_s" 2>/dev/null)

  # Format deltas
  fmt_delta() {
    local before="$1" after="$2"
    if [[ -z "$before" || -z "$after" || "$before" == "$after" ]]; then
      echo "${after:-n/a}"
    elif [[ "$after" == "true" && "$before" == "false" ]]; then
      echo -e "${GREEN}false→true${NC}"
    elif [[ "$after" == "false" && "$before" == "true" ]]; then
      echo -e "${RED}true→false${NC}"
    else
      echo "${before}→${after}"
    fi
  }

  d_tests=$(fmt_delta "${i_tests:-}" "${c_tests:-}")
  d_lint=$(fmt_delta "${i_lint:-}" "${c_lint:-}")
  d_sam=$(fmt_delta "${i_sam:-}" "${c_sam:-}")
  d_deploy=$(fmt_delta "${i_deploy:-}" "${c_deploy:-}")

  if [[ -n "$i_time" && -n "$c_time" ]]; then
    delta=$((c_time - i_time))
    if [[ $delta -lt 0 ]]; then
      d_time="${c_time}s (${delta}s)"
    else
      d_time="${c_time}s (+${delta}s)"
    fi
  else
    d_time="${c_time:-n/a}s"
  fi

  printf "%-25s  %-12b %-12b %-12b %-12b %-12s\n" "$p" "$d_tests" "$d_lint" "$d_sam" "$d_deploy" "$d_time"
done

echo ""

# Show net skill changes
cd "$REPO_ROOT"
if git diff --quiet HEAD -- skills/; then
  echo -e "${YELLOW}No net skill changes after improvement loop${NC}"
else
  echo -e "${GREEN}Skills modified:${NC}"
  git diff --stat HEAD -- skills/
fi

# Cleanup
rm -rf "$SCRIPT_DIR/results-initial" 2>/dev/null || true
