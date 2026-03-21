#!/usr/bin/env bash
# E2E test helper library for konductor
# Source this file: source tests/e2e/lib.sh

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Counters
PASS_COUNT=0
FAIL_COUNT=0
STEP_START=0

# --- Logging ---

log_step() {
  local name="$1"
  STEP_START=$(date +%s)
  echo -e "\n${CYAN}▶ $name${NC}"
}

log_step_done() {
  local elapsed=$(( $(date +%s) - STEP_START ))
  echo -e "${GREEN}  ✓ done (${elapsed}s)${NC}"
}

log_pass() {
  echo -e "  ${GREEN}✓ $1${NC}"
  PASS_COUNT=$((PASS_COUNT + 1))
}

log_fail() {
  echo -e "  ${RED}✗ $1${NC}"
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

log_warn() {
  echo -e "  ${YELLOW}⚠ $1${NC}"
}

log_summary() {
  echo -e "\n${CYAN}═══ Results ═══${NC}"
  echo -e "  ${GREEN}Passed: $PASS_COUNT${NC}"
  echo -e "  ${RED}Failed: $FAIL_COUNT${NC}"
  if [[ $FAIL_COUNT -gt 0 ]]; then
    echo -e "\n${RED}FAIL${NC}"
    return 1
  else
    echo -e "\n${GREEN}PASS${NC}"
    return 0
  fi
}

# --- Kiro CLI wrapper ---

KONDUCTOR_TIMEOUT="${KONDUCTOR_TIMEOUT:-300}"

konductor_chat() {
  local message="$1"
  local output
  output=$(timeout "$KONDUCTOR_TIMEOUT" kiro-cli chat \
    --agent konductor \
    --no-interactive \
    --trust-all-tools \
    "$message" 2>&1) || true
  echo "$output"
}

# --- Code Quality Helpers ---

# Run ruff linter on Python files. Writes results to a TOML file.
# Usage: run_lint <project_dir> <results_toml>
run_lint() {
  local project_dir="$1"
  local results_file="$2"
  local py_files
  py_files=$(find "$project_dir" -maxdepth 2 -name "*.py" ! -path "*/.konductor/*" 2>/dev/null)

  if [[ -z "$py_files" ]]; then
    echo "lint_pass = false" >> "$results_file"
    echo "lint_errors = -1" >> "$results_file"
    return
  fi

  if ! command -v ruff >/dev/null 2>&1; then
    log_warn "ruff not found, skipping lint"
    echo "lint_pass = \"skipped\"" >> "$results_file"
    echo "lint_errors = -1" >> "$results_file"
    return
  fi

  local lint_output exit_code=0
  lint_output=$(cd "$project_dir" && ruff check $py_files 2>&1) || exit_code=$?
  local error_count
  error_count=$(echo "$lint_output" | grep -cE "^.+:\d+:\d+:" || echo "0")

  if [[ $exit_code -eq 0 ]]; then
    echo "lint_pass = true" >> "$results_file"
  else
    echo "lint_pass = false" >> "$results_file"
  fi
  echo "lint_errors = $error_count" >> "$results_file"
}

# Run pytest on test files. Writes results to a TOML file.
# Usage: run_tests <project_dir> <results_toml>
run_tests() {
  local project_dir="$1"
  local results_file="$2"
  local test_files
  test_files=$(find "$project_dir" -maxdepth 2 -name "test_*.py" ! -path "*/.konductor/*" 2>/dev/null)

  if [[ -z "$test_files" ]]; then
    echo "tests_pass = false" >> "$results_file"
    echo "tests_total = 0" >> "$results_file"
    echo "tests_failed = 0" >> "$results_file"
    return
  fi

  if ! command -v pytest >/dev/null 2>&1; then
    log_warn "pytest not found, skipping tests"
    echo "tests_pass = \"skipped\"" >> "$results_file"
    echo "tests_total = -1" >> "$results_file"
    echo "tests_failed = -1" >> "$results_file"
    return
  fi

  local test_output exit_code=0
  test_output=$(cd "$project_dir" && pytest -v --tb=short 2>&1) || exit_code=$?

  local total passed failed
  # Parse pytest summary line like "8 passed, 2 failed"
  total=$(echo "$test_output" | grep -oP '\d+ passed' | grep -oP '\d+' || echo "0")
  failed=$(echo "$test_output" | grep -oP '\d+ failed' | grep -oP '\d+' || echo "0")
  total=$(( ${total:-0} + ${failed:-0} ))

  if [[ $exit_code -eq 0 ]]; then
    echo "tests_pass = true" >> "$results_file"
  else
    echo "tests_pass = false" >> "$results_file"
  fi
  echo "tests_total = $total" >> "$results_file"
  echo "tests_failed = ${failed:-0}" >> "$results_file"
}

# Check SAM template validity. Writes results to a TOML file.
# Usage: check_sam_template <project_dir> <results_toml>
check_sam_template() {
  local project_dir="$1"
  local results_file="$2"

  if [[ ! -f "$project_dir/template.yaml" ]]; then
    echo "sam_template = false" >> "$results_file"
    echo "sam_valid = false" >> "$results_file"
    return
  fi

  echo "sam_template = true" >> "$results_file"

  # Basic YAML validation (no SAM CLI needed)
  if python3 -c "import yaml; yaml.safe_load(open('$project_dir/template.yaml'))" 2>/dev/null; then
    # Try sam validate if available
    if command -v sam >/dev/null 2>&1; then
      if (cd "$project_dir" && sam validate 2>/dev/null); then
        echo "sam_valid = true" >> "$results_file"
      else
        echo "sam_valid = false" >> "$results_file"
      fi
    else
      echo "sam_valid = \"skipped\"" >> "$results_file"
    fi
  else
    echo "sam_valid = false" >> "$results_file"
  fi
}

# --- AWS / Deployment Helpers ---

# Check if AWS credentials are available
has_aws_credentials() {
  aws sts get-caller-identity >/dev/null 2>&1
}

# Run deploy → verify → teardown cycle for a SAM project.
# Usage: run_deploy_cycle <project_dir> <results_toml>
# Always runs teardown even if verify fails.
run_deploy_cycle() {
  local project_dir="$1"
  local results_file="$2"
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

  if [[ ! -f "$project_dir/template.yaml" ]]; then
    echo "deploy_success = \"skipped\"" >> "$results_file"
    echo "verify_success = \"skipped\"" >> "$results_file"
    echo "teardown_success = \"skipped\"" >> "$results_file"
    return
  fi

  if ! has_aws_credentials; then
    log_warn "No AWS credentials — skipping deployment"
    echo "deploy_success = \"skipped\"" >> "$results_file"
    echo "verify_success = \"skipped\"" >> "$results_file"
    echo "teardown_success = \"skipped\"" >> "$results_file"
    return
  fi

  local deploy_start deploy_end deploy_ok=false verify_ok=false teardown_ok=false

  # Deploy
  deploy_start=$(date +%s)
  if bash "$script_dir/deploy.sh" "$project_dir" 2>&1; then
    deploy_ok=true
  fi
  deploy_end=$(date +%s)
  echo "deploy_success = $deploy_ok" >> "$results_file"
  echo "deploy_time_s = $((deploy_end - deploy_start))" >> "$results_file"

  # Verify (only if deploy succeeded)
  if [[ "$deploy_ok" == "true" ]]; then
    local project_type
    project_type=$(basename "$project_dir")
    if bash "$script_dir/verify-deployment.sh" "$project_dir" "$project_type" 2>&1; then
      verify_ok=true
    fi
  fi
  echo "verify_success = $verify_ok" >> "$results_file"

  # Teardown (always, even if verify failed)
  if bash "$script_dir/teardown.sh" "$project_dir" 2>&1; then
    teardown_ok=true
  fi
  echo "teardown_success = $teardown_ok" >> "$results_file"
}

# --- Assertions ---

assert_file_exists() {
  local path="$1"
  local label="${2:-$path}"
  if [[ -f "$path" ]]; then
    log_pass "$label exists"
  else
    log_fail "$label missing: $path"
  fi
}

assert_dir_exists() {
  local path="$1"
  local label="${2:-$path}"
  if [[ -d "$path" ]]; then
    log_pass "$label exists"
  else
    log_fail "$label missing: $path"
  fi
}

assert_file_contains() {
  local path="$1"
  local pattern="$2"
  local label="${3:-$path contains '$pattern'}"
  if [[ -f "$path" ]] && grep -q "$pattern" "$path"; then
    log_pass "$label"
  else
    log_fail "$label"
  fi
}

assert_file_not_empty() {
  local path="$1"
  local label="${2:-$path is non-empty}"
  if [[ -f "$path" ]] && [[ -s "$path" ]]; then
    log_pass "$label"
  else
    log_fail "$label"
  fi
}

assert_state_step() {
  local expected="$1"
  local state_file="${TEST_DIR}/.konductor/state.toml"
  if [[ ! -f "$state_file" ]]; then
    log_fail "state.toml missing (expected step=$expected)"
    return
  fi
  local actual
  actual=$(grep '^step' "$state_file" | head -1 | sed 's/.*= *"\(.*\)"/\1/')
  if [[ "$actual" == "$expected" ]]; then
    log_pass "state.step = $expected"
  else
    log_fail "state.step = $actual (expected $expected)"
  fi
}

assert_min_file_count() {
  local dir="$1"
  local pattern="$2"
  local min="$3"
  local label="${4:-at least $min files matching $pattern in $dir}"
  local count
  count=$(find "$dir" -name "$pattern" 2>/dev/null | wc -l)
  if [[ $count -ge $min ]]; then
    log_pass "$label (found $count)"
  else
    log_fail "$label (found $count, need $min)"
  fi
}

# --- Test directory management ---

TEST_DIR=""

setup_test_dir() {
  TEST_DIR=$(mktemp -d)
  echo -e "${CYAN}Test directory: $TEST_DIR${NC}"

  # Copy synthetic project spec (caller specifies which project)
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

  # Install hook config so kiro-cli triggers konductor hooks
  local hooks_dir="$TEST_DIR/.kiro/hooks"
  mkdir -p "$hooks_dir"

  local konductor_bin
  konductor_bin=$(command -v konductor 2>/dev/null || echo "$HOME/.kiro/bin/konductor")

  cat > "$hooks_dir/konductor-hooks.json" << EOF
{
  "hooks": [
    {
      "event": "PostToolUse",
      "matcher": "write",
      "command": "$konductor_bin hook",
      "timeout_ms": 2000
    },
    {
      "event": "PreToolUse",
      "matcher": "shell",
      "command": "$konductor_bin hook",
      "timeout_ms": 1000
    }
  ]
}
EOF
}

# Setup test dir for a specific synthetic project
# Usage: setup_project_test_dir <project_name>
setup_project_test_dir() {
  local project_name="$1"
  setup_test_dir

  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local spec_dir="$script_dir/synthetic-projects/$project_name"

  if [[ -d "$spec_dir" ]]; then
    cp -r "$spec_dir" "$TEST_DIR/synthetic-project"
  else
    echo "ERROR: Synthetic project not found: $spec_dir"
    return 1
  fi
}

# List all available synthetic projects
list_synthetic_projects() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  find "$script_dir/synthetic-projects" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort
}

# --- Hook assertions ---

assert_tracking_log_exists() {
  local log="$TEST_DIR/.konductor/.tracking/modified-files.log"
  if [[ -f "$log" ]] && [[ -s "$log" ]]; then
    log_pass "hook tracking log exists and non-empty"
  else
    log_fail "hook tracking log missing or empty: $log"
  fi
}

assert_tracking_log_contains() {
  local pattern="$1"
  local label="${2:-tracking log contains '$pattern'}"
  local log="$TEST_DIR/.konductor/.tracking/modified-files.log"
  if [[ -f "$log" ]] && grep -q "$pattern" "$log"; then
    log_pass "$label"
  else
    log_fail "$label"
  fi
}

assert_hook_blocks_destructive() {
  local konductor_bin
  konductor_bin=$(command -v konductor 2>/dev/null || echo "$HOME/.kiro/bin/konductor")
  local input='{"hook_event_name":"PreToolUse","tool_name":"shell","tool_input":{"command":"rm -rf /"}}'
  local exit_code=0
  echo "$input" | "$konductor_bin" hook 2>/dev/null || exit_code=$?
  if [[ $exit_code -eq 2 ]]; then
    log_pass "hook blocks destructive command (exit code 2)"
  else
    log_fail "hook should block destructive command with exit 2, got $exit_code"
  fi
}

teardown_test_dir() {
  if [[ -n "$TEST_DIR" && -d "$TEST_DIR" ]]; then
    if [[ "${KEEP_TEST_DIR:-}" == "1" ]]; then
      echo -e "${YELLOW}Keeping test dir: $TEST_DIR${NC}"
    else
      rm -rf "$TEST_DIR"
    fi
  fi
}
