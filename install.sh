#!/usr/bin/env bash
set -e

KONDUCTOR_VERSION="${KONDUCTOR_VERSION:-latest}"
REPO="bnusunny/konductor"

# Detect scope: global or local
if [[ "$1" == "--global" || "$1" == "-g" ]]; then
  SCOPE="global"
  TARGET_DIR="$HOME/.kiro"
elif [[ "$1" == "--local" || "$1" == "-l" ]]; then
  SCOPE="local"
  TARGET_DIR="./.kiro"
else
  SCOPE="global"
  TARGET_DIR="$HOME/.kiro"
fi

echo "Installing Konductor ($SCOPE scope) to $TARGET_DIR..."

# Create target directories
mkdir -p "$TARGET_DIR/agents"
mkdir -p "$TARGET_DIR/skills"
mkdir -p "$TARGET_DIR/hooks"
mkdir -p "$TARGET_DIR/bin"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Copy agents
if [[ -d "$SCRIPT_DIR/agents" ]]; then
  echo "Installing agents..."
  cp -r "$SCRIPT_DIR/agents"/*.json "$TARGET_DIR/agents/"
fi

# Copy skills
if [[ -d "$SCRIPT_DIR/skills" ]]; then
  echo "Installing skills..."
  for skill_dir in "$SCRIPT_DIR/skills"/konductor-*; do
    if [[ -d "$skill_dir" ]]; then
      skill_name=$(basename "$skill_dir")
      cp -r "$skill_dir" "$TARGET_DIR/skills/"
    fi
  done
fi

# Copy hooks
if [[ -f "$SCRIPT_DIR/hooks/konductor-hooks.json" ]]; then
  echo "Installing hooks..."
  cp "$SCRIPT_DIR/hooks/konductor-hooks.json" "$TARGET_DIR/hooks/"
fi

# Install hook binary
HOOK_BINARY="$TARGET_DIR/bin/konductor-hook"
if [[ -f "$SCRIPT_DIR/konductor-hook/target/release/konductor-hook" ]]; then
  # Use locally built binary
  echo "Installing locally built hook binary..."
  cp "$SCRIPT_DIR/konductor-hook/target/release/konductor-hook" "$HOOK_BINARY"
  chmod +x "$HOOK_BINARY"
else
  # Download prebuilt binary from GitHub releases
  echo "Downloading prebuilt hook binary..."

  # Detect platform
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)

  case "$OS" in
    darwin) PLATFORM="macos" ;;
    linux) PLATFORM="linux" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
  esac

  case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
  esac

  BINARY_NAME="konductor-hook-${PLATFORM}-${ARCH}"

  if [[ "$KONDUCTOR_VERSION" == "latest" ]]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
  else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${KONDUCTOR_VERSION}/${BINARY_NAME}"
  fi

  echo "Downloading from: $DOWNLOAD_URL"

  if command -v curl >/dev/null 2>&1; then
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$HOOK_BINARY" 2>/dev/null; then
      echo "⚠ Hook binary not available (no release found). Skipping."
      rm -f "$HOOK_BINARY"
    else
      chmod +x "$HOOK_BINARY"
    fi
  elif command -v wget >/dev/null 2>&1; then
    if ! wget -q "$DOWNLOAD_URL" -O "$HOOK_BINARY" 2>/dev/null; then
      echo "⚠ Hook binary not available (no release found). Skipping."
      rm -f "$HOOK_BINARY"
    else
      chmod +x "$HOOK_BINARY"
    fi
  else
    echo "⚠ curl or wget is required to download the hook binary. Skipping."
  fi
fi

echo ""
echo "✓ Konductor installed successfully!"
echo ""
echo "Usage:"
echo "  kiro-cli --agent konductor"
echo "  > initialize my project"
echo "  > next"
echo ""
echo "For more information, see: https://github.com/${REPO}"
