#!/usr/bin/env bash
set -e

KONDUCTOR_VERSION="${KONDUCTOR_VERSION:-latest}"
REPO="bnusunny/konductor"
FORCE=false

# Parse arguments
for arg in "$@"; do
  case "$arg" in
    --global|-g) SCOPE="global"; TARGET_DIR="$HOME/.kiro" ;;
    --local|-l) SCOPE="local"; TARGET_DIR="./.kiro" ;;
    --force|-f) FORCE=true ;;
  esac
done

# Defaults
SCOPE="${SCOPE:-global}"
TARGET_DIR="${TARGET_DIR:-$HOME/.kiro}"

# Safe copy: skip existing files unless --force
safe_cp() {
  local src="$1" dst="$2"
  if [[ -e "$dst" && "$FORCE" != true ]]; then
    echo "  ⏭ Skipping $(basename "$dst") (already exists, use --force to overwrite)"
    return
  fi
  cp "$src" "$dst"
}

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
  for f in "$SCRIPT_DIR/agents"/*.json; do
    safe_cp "$f" "$TARGET_DIR/agents/$(basename "$f")"
  done
fi

# Copy skills
if [[ -d "$SCRIPT_DIR/skills" ]]; then
  echo "Installing skills..."
  for skill_dir in "$SCRIPT_DIR/skills"/konductor-*; do
    if [[ -d "$skill_dir" ]]; then
      skill_name=$(basename "$skill_dir")
      if [[ -d "$TARGET_DIR/skills/$skill_name" && "$FORCE" != true ]]; then
        echo "  ⏭ Skipping $skill_name/ (already exists, use --force to overwrite)"
      else
        cp -r "$skill_dir" "$TARGET_DIR/skills/"
      fi
    fi
  done
fi

# Copy hooks
if [[ -f "$SCRIPT_DIR/hooks/konductor-hooks.json" ]]; then
  echo "Installing hooks..."
  safe_cp "$SCRIPT_DIR/hooks/konductor-hooks.json" "$TARGET_DIR/hooks/konductor-hooks.json"
fi

# Install konductor binary (unified: mcp server + hook processor)
KONDUCTOR_BINARY="$TARGET_DIR/bin/konductor"
if [[ -e "$KONDUCTOR_BINARY" && "$FORCE" != true ]]; then
  echo "  ⏭ Skipping konductor binary (already exists, use --force to overwrite)"
elif [[ -f "$SCRIPT_DIR/konductor-cli/target/release/konductor" ]]; then
  echo "Installing locally built konductor binary..."
  cp "$SCRIPT_DIR/konductor-cli/target/release/konductor" "$KONDUCTOR_BINARY"
  chmod +x "$KONDUCTOR_BINARY"
else
  echo "Downloading prebuilt konductor binary..."

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

  BINARY_NAME="konductor-${PLATFORM}-${ARCH}"

  if [[ "$KONDUCTOR_VERSION" == "latest" ]]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
  else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${KONDUCTOR_VERSION}/${BINARY_NAME}"
  fi

  echo "Downloading from: $DOWNLOAD_URL"

  if command -v curl >/dev/null 2>&1; then
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$KONDUCTOR_BINARY" 2>/dev/null; then
      echo "⚠ Binary not available (no release found). Skipping."
      rm -f "$KONDUCTOR_BINARY"
    else
      chmod +x "$KONDUCTOR_BINARY"
    fi
  elif command -v wget >/dev/null 2>&1; then
    if ! wget -q "$DOWNLOAD_URL" -O "$KONDUCTOR_BINARY" 2>/dev/null; then
      echo "⚠ Binary not available (no release found). Skipping."
      rm -f "$KONDUCTOR_BINARY"
    else
      chmod +x "$KONDUCTOR_BINARY"
    fi
  else
    echo "⚠ curl or wget is required to download the binary. Skipping."
  fi
fi

echo ""
echo "✓ Konductor installed successfully!"
echo ""
echo "Usage:"
echo "  kiro-cli --agent konductor"
echo ""
echo "Prompts available (type @ then Tab to autocomplete):"
echo "  @k-init    @k-plan    @k-exec    @k-verify"
echo "  @k-ship    @k-next    @k-status  @k-discuss  @k-map"
echo ""
echo "MCP tools available:"
echo "  state_get, state_transition, state_add_blocker,"
echo "  state_resolve_blocker, plans_list, status"
echo ""
echo "For more information, see: https://github.com/${REPO}"
