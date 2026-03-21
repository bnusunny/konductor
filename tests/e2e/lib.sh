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

  # Copy synthetic project spec
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cp -r "$script_dir/synthetic-project" "$TEST_DIR/synthetic-project"

  # Install hook config so kiro-cli triggers konductor hooks
  local hooks_dir="$TEST_DIR/.kiro/hooks"
  mkdir -p "$hooks_dir"

  # Resolve konductor binary — prefer the one in PATH, fall back to ~/.kiro/bin
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
  echo -e "${CYAN}Hook config installed: $hooks_dir/konductor-hooks.json${NC}"
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
