#!/usr/bin/env bash
# E2E pipeline test for konductor
# Drives kiro-cli with the konductor agent through the full pipeline
# for each synthetic project independently.
#
# Usage:
#   KONDUCTOR_E2E=1 bash tests/e2e/run.sh
#   KONDUCTOR_E2E=1 bash tests/e2e/run.sh --project cli-calculator
#
# Options:
#   KEEP_TEST_DIR=1    Keep the test directory after run
#   KONDUCTOR_TIMEOUT  Per-step timeout in seconds (default: 300)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib.sh"

FILTER_PROJECT=""
[[ "${1:-}" == "--project" ]] && FILTER_PROJECT="${2:-}"

if [[ "${KONDUCTOR_E2E:-}" != "1" ]]; then
  echo "Skipping E2E test (set KONDUCTOR_E2E=1 to run)"
  echo "Requires: kiro-cli with authenticated session and LLM access"
  exit 0
fi

if ! command -v kiro-cli >/dev/null 2>&1; then
  echo "ERROR: kiro-cli not found in PATH"
  exit 1
fi

# Discover projects
PROJECTS=()
while IFS= read -r p; do
  if [[ -z "$FILTER_PROJECT" || "$p" == "$FILTER_PROJECT" ]]; then
    PROJECTS+=("$p")
  fi
done < <(list_synthetic_projects)

echo -e "${CYAN}═══ Konductor E2E Pipeline Test ═══${NC}"
echo "Projects: ${PROJECTS[*]}"

for PROJECT in "${PROJECTS[@]}"; do
  echo -e "\n${CYAN}═══ Project: $PROJECT ═══${NC}"

  SPEC_FILE="$SCRIPT_DIR/synthetic-projects/$PROJECT/spec.md"
  SPEC_CONTENT=$(cat "$SPEC_FILE")

  # Detect if this is a SAM project
  IS_SAM=false
  grep -qi "sam\|template\.yaml" "$SPEC_FILE" && IS_SAM=true

  setup_project_test_dir "$PROJECT"
  trap teardown_test_dir EXIT
  cd "$TEST_DIR"

  # --- Step 1: Initialize ---

  log_step "[$PROJECT] Initialize"

  konductor_chat "Initialize this project with Konductor. Here is the project spec:

$SPEC_CONTENT

This is a small 1-phase project. Generate project.md, requirements.md, roadmap.md with a single phase.
Do NOT ask me questions — use the spec above to generate all documents.
Run the konductor-init skill." > /dev/null

  assert_file_exists "$TEST_DIR/.konductor/state.toml" "state.toml"
  assert_file_exists "$TEST_DIR/.konductor/project.md" "project.md"
  assert_file_not_empty "$TEST_DIR/.konductor/project.md" "project.md non-empty"
  log_step_done

  # --- Step 2: Plan ---

  log_step "[$PROJECT] Plan"

  konductor_chat "Plan phase 01 for execution. Research the ecosystem, create execution plans with tasks and acceptance criteria, and validate them. Run the konductor-plan skill." > /dev/null

  assert_dir_exists "$TEST_DIR/.konductor/phases" "phases directory"
  assert_min_file_count "$TEST_DIR/.konductor/phases" "*.md" 1 "at least 1 plan file"
  log_step_done

  # --- Step 3: Execute ---

  log_step "[$PROJECT] Execute"

  konductor_chat "Execute the plans for the current phase. Spawn executor subagents to implement each plan following TDD workflow. Run the konductor-exec skill." > /dev/null

  assert_min_file_count "$TEST_DIR" "*.py" 1 "at least 1 Python file"
  assert_min_file_count "$TEST_DIR" "test_*.py" 1 "at least 1 test file"
  log_step_done

  # --- Step 4: Verify hooks ---

  log_step "[$PROJECT] Verify hooks"
  assert_tracking_log_exists
  assert_tracking_log_contains "\.py" "tracking log recorded Python files"
  assert_hook_blocks_destructive
  log_step_done

  # --- Step 5: Quality checks ---

  log_step "[$PROJECT] Quality checks"

  # Lint
  if command -v ruff >/dev/null 2>&1; then
    py_files=$(find "$TEST_DIR" -maxdepth 2 -name "*.py" ! -path "*/.konductor/*")
    if [[ -n "$py_files" ]]; then
      if (cd "$TEST_DIR" && ruff check $py_files 2>&1); then
        log_pass "ruff lint passes"
      else
        log_fail "ruff lint has errors"
      fi
    fi
  else
    log_warn "ruff not available, skipping lint"
  fi

  # Tests
  if command -v pytest >/dev/null 2>&1; then
    test_files=$(find "$TEST_DIR" -maxdepth 2 -name "test_*.py" ! -path "*/.konductor/*")
    if [[ -n "$test_files" ]]; then
      if (cd "$TEST_DIR" && pytest -v --tb=short 2>&1); then
        log_pass "pytest passes"
      else
        log_fail "pytest has failures"
      fi
    fi
  else
    log_warn "pytest not available, skipping tests"
  fi

  log_step_done

  # --- Step 6: SAM-specific checks ---

  if [[ "$IS_SAM" == "true" ]]; then
    log_step "[$PROJECT] SAM template checks"

    assert_file_exists "$TEST_DIR/template.yaml" "template.yaml"

    if [[ -f "$TEST_DIR/template.yaml" ]]; then
      # YAML validity
      if python3 -c "import yaml; yaml.safe_load(open('$TEST_DIR/template.yaml'))" 2>/dev/null; then
        log_pass "template.yaml is valid YAML"

        # Check for expected SAM resources
        if grep -q "AWS::Serverless::Function" "$TEST_DIR/template.yaml"; then
          log_pass "template contains AWS::Serverless::Function"
        else
          log_fail "template missing AWS::Serverless::Function"
        fi

        if grep -q "Transform.*AWS::Serverless" "$TEST_DIR/template.yaml"; then
          log_pass "template has SAM Transform"
        else
          log_fail "template missing SAM Transform"
        fi
      else
        log_fail "template.yaml is not valid YAML"
      fi

      # sam validate if available
      if command -v sam >/dev/null 2>&1; then
        if (cd "$TEST_DIR" && sam validate 2>&1); then
          log_pass "sam validate passes"
        else
          log_fail "sam validate fails"
        fi
      else
        log_warn "sam CLI not available, skipping sam validate"
      fi
    fi

    log_step_done
  fi

  # --- Step 7: Verify phase ---

  log_step "[$PROJECT] Verify phase"

  konductor_chat "Verify the current phase after execution. Validate that phase goals were achieved using the 3-level verification framework. Run the konductor-verify skill." > /dev/null

  log_step_done

  # --- Cleanup ---

  echo -e "\n${CYAN}── $PROJECT Artifacts ──${NC}"
  echo "Python files:"
  find "$TEST_DIR" -maxdepth 2 -name "*.py" ! -path "*/.konductor/*" -exec basename {} \; | sort | sed 's/^/  /'
  if [[ "$IS_SAM" == "true" && -f "$TEST_DIR/template.yaml" ]]; then
    echo "SAM template: present"
  fi

  teardown_test_dir
  trap - EXIT
done

log_summary
