#!/usr/bin/env bash
# Tests for install.sh
# Usage: bash tests/test_installer.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

assert_file_exists()  { [[ -f "$1" ]] && pass "$2" || fail "$2: $1 missing"; }
assert_dir_exists()   { [[ -d "$1" ]] && pass "$2" || fail "$2: $1 missing"; }
assert_file_contains() { [[ -f "$1" ]] && grep -q "$2" "$1" && pass "$3" || fail "$3"; }
assert_executable()   { [[ -x "$1" ]] && pass "$2" || fail "$2: $1 not executable"; }

# Create a fake konductor binary for testing (avoids downloading)
setup_fake_binary() {
  local release_dir="$PROJECT_DIR/konductor-cli/target/release"
  mkdir -p "$release_dir"
  if [[ ! -f "$release_dir/konductor" ]]; then
    echo '#!/bin/sh' > "$release_dir/konductor"
    echo 'echo "konductor fake binary"' >> "$release_dir/konductor"
    chmod +x "$release_dir/konductor"
    CREATED_FAKE_BINARY=true
  else
    CREATED_FAKE_BINARY=false
  fi
}

cleanup_fake_binary() {
  if [[ "$CREATED_FAKE_BINARY" == true ]]; then
    rm -f "$PROJECT_DIR/konductor-cli/target/release/konductor"
  fi
}

# ============================================================
echo "=== Test 1: Global install (default) ==="
# ============================================================

TMP=$(mktemp -d)
setup_fake_binary
HOME="$TMP" bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1
cleanup_fake_binary

TARGET="$TMP/.kiro"

# Directory structure
assert_dir_exists "$TARGET/agents" "agents dir created"
assert_dir_exists "$TARGET/skills" "skills dir created"
assert_dir_exists "$TARGET/hooks" "hooks dir created"
assert_dir_exists "$TARGET/bin" "bin dir created"

# Agents
assert_file_exists "$TARGET/agents/konductor.json" "main agent installed"
assert_file_exists "$TARGET/agents/konductor-executor.json" "executor agent installed"

# Agents reference hooks
assert_file_contains "$TARGET/agents/konductor.json" '"PostToolUse"' "main agent has PostToolUse hook"
assert_file_contains "$TARGET/agents/konductor-executor.json" '"PostToolUse"' "executor agent has PostToolUse hook"

# Hooks
assert_file_exists "$TARGET/hooks/konductor-hooks.json" "hooks config installed"
assert_file_contains "$TARGET/hooks/konductor-hooks.json" "konductor hook" "hooks use 'konductor hook' command"
assert_file_contains "$TARGET/hooks/konductor-hooks.json" "PostToolUse" "hooks include PostToolUse"
assert_file_contains "$TARGET/hooks/konductor-hooks.json" "PreToolUse" "hooks include PreToolUse"

# Skills
assert_dir_exists "$TARGET/skills/konductor-init" "init skill installed"
assert_dir_exists "$TARGET/skills/konductor-plan" "plan skill installed"
assert_dir_exists "$TARGET/skills/konductor-exec" "exec skill installed"
assert_dir_exists "$TARGET/skills/konductor-verify" "verify skill installed"
assert_dir_exists "$TARGET/skills/konductor-ship" "ship skill installed"

# Binary
assert_file_exists "$TARGET/bin/konductor" "binary installed"
assert_executable "$TARGET/bin/konductor" "binary is executable"

# Agent config uses bare command
assert_file_contains "$TARGET/agents/konductor.json" '"command": "konductor"' "agent uses bare 'konductor' command"

rm -rf "$TMP"

# ============================================================
echo "=== Test 2: Local install ==="
# ============================================================

TMP=$(mktemp -d)
setup_fake_binary
(cd "$TMP" && bash "$PROJECT_DIR/install.sh" --local --force > /dev/null 2>&1)
cleanup_fake_binary

TARGET="$TMP/.kiro"

assert_dir_exists "$TARGET" "local .kiro dir created"
assert_file_exists "$TARGET/agents/konductor.json" "agent installed locally"
assert_file_exists "$TARGET/hooks/konductor-hooks.json" "hooks installed locally"
assert_file_exists "$TARGET/bin/konductor" "binary installed locally"
assert_executable "$TARGET/bin/konductor" "local binary is executable"

rm -rf "$TMP"

# ============================================================
echo "=== Test 3: Skip existing files without --force ==="
# ============================================================

TMP=$(mktemp -d)
setup_fake_binary

# First install
HOME="$TMP" bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1

# Modify a file to detect if it gets overwritten
echo "MARKER" > "$TMP/.kiro/agents/konductor.json"

# Second install without --force
HOME="$TMP" bash "$PROJECT_DIR/install.sh" > /dev/null 2>&1
cleanup_fake_binary

if grep -q "MARKER" "$TMP/.kiro/agents/konductor.json"; then
  pass "existing files preserved without --force"
else
  fail "existing files were overwritten without --force"
fi

rm -rf "$TMP"

# ============================================================
echo "=== Test 4: --force overwrites existing files ==="
# ============================================================

TMP=$(mktemp -d)
setup_fake_binary

# First install
HOME="$TMP" bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1

# Modify a file
echo "MARKER" > "$TMP/.kiro/agents/konductor.json"

# Second install with --force
HOME="$TMP" bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1
cleanup_fake_binary

if grep -q "MARKER" "$TMP/.kiro/agents/konductor.json"; then
  fail "--force did not overwrite files"
else
  pass "--force overwrites existing files"
fi

rm -rf "$TMP"

# ============================================================
echo "=== Test 5: All skills installed ==="
# ============================================================

TMP=$(mktemp -d)
setup_fake_binary
HOME="$TMP" bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1
cleanup_fake_binary

TARGET="$TMP/.kiro"
for skill in init plan exec verify ship next status discuss map-codebase; do
  assert_dir_exists "$TARGET/skills/konductor-$skill" "skill konductor-$skill installed"
done

rm -rf "$TMP"

# ============================================================
echo "=== Test 6: All agents installed ==="
# ============================================================

TMP=$(mktemp -d)
setup_fake_binary
HOME="$TMP" bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1
cleanup_fake_binary

TARGET="$TMP/.kiro"
for agent in konductor konductor-executor konductor-planner konductor-researcher konductor-verifier konductor-discoverer konductor-plan-checker konductor-codebase-mapper; do
  assert_file_exists "$TARGET/agents/$agent.json" "agent $agent.json installed"
done

rm -rf "$TMP"

# ============================================================
echo "=== Test 7: PATH setup for global install ==="
# ============================================================

TMP=$(mktemp -d)
# Create a .bashrc so the installer can append to it
touch "$TMP/.bashrc"

setup_fake_binary
# Run with HOME overridden and konductor not on PATH
env HOME="$TMP" PATH="/usr/bin:/bin" SHELL="/bin/bash" \
  bash "$PROJECT_DIR/install.sh" --force > /dev/null 2>&1
cleanup_fake_binary

if grep -q ".kiro/bin" "$TMP/.bashrc"; then
  pass "PATH added to .bashrc"
else
  fail "PATH not added to .bashrc"
fi

rm -rf "$TMP"

# ============================================================
echo "=== Test 8: --help exits cleanly ==="
# ============================================================

output=$(bash "$PROJECT_DIR/install.sh" --help 2>&1)
if echo "$output" | grep -q "Usage:"; then
  pass "--help shows usage"
else
  fail "--help does not show usage"
fi

# ============================================================
# Summary
# ============================================================
echo ""
echo "Results: $PASS passed, $FAIL failed"
if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
