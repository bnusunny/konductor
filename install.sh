#!/usr/bin/env bash
set -e

KONDUCTOR_VERSION="${KONDUCTOR_VERSION:-latest}"
REPO="bnusunny/konductor"
FORCE=false

usage() {
  cat <<EOF
Konductor Installer

Usage: install.sh [OPTIONS]

Options:
  --global, -g    Install to ~/.kiro (default)
  --local, -l     Install to ./.kiro (current project)
  --force, -f     Overwrite existing files
  --help, -h      Show this help message

Environment Variables:
  KONDUCTOR_VERSION   Version to install (default: latest)

Examples:
  bash install.sh                    # Global install, latest version
  bash install.sh --local --force    # Local install, overwrite existing
  KONDUCTOR_VERSION=0.2.1 bash install.sh  # Install specific version
EOF
}

# Parse arguments
for arg in "$@"; do
  case "$arg" in
    --global|-g) SCOPE="global"; TARGET_DIR="$HOME/.kiro" ;;
    --local|-l) SCOPE="local"; TARGET_DIR="./.kiro" ;;
    --force|-f) FORCE=true ;;
    --help|-h) usage; exit 0 ;;
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

# Verify a downloaded binary is a real executable, not an error page
verify_binary() {
  local binary_path="$1"
  if [[ ! -f "$binary_path" ]]; then
    return 1
  fi
  local file_type
  file_type=$(file -b "$binary_path" 2>/dev/null || echo "unknown")
  case "$file_type" in
    *ELF*|*Mach-O*|*executable*)
      return 0
      ;;
    *)
      echo "✗ Downloaded file is not a valid binary (got: $file_type)"
      rm -f "$binary_path"
      return 1
      ;;
  esac
}

# Download a file with curl or wget
download() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest" 2>/dev/null
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$dest" 2>/dev/null
  else
    echo "✗ curl or wget is required to download files."
    return 1
  fi
}

# Verify checksum if .sha256 file is available
verify_checksum() {
  local binary_path="$1" checksum_url="$2"
  local checksum_file
  checksum_file=$(mktemp)
  trap "rm -f '$checksum_file'" RETURN

  if ! download "$checksum_url" "$checksum_file"; then
    echo "  ⚠ No checksum file available — skipping verification"
    return 0
  fi

  local expected
  expected=$(awk '{print $1}' "$checksum_file")
  local actual
  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$binary_path" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$binary_path" | awk '{print $1}')
  else
    echo "  ⚠ sha256sum/shasum not found — skipping checksum verification"
    return 0
  fi

  if [[ "$expected" != "$actual" ]]; then
    echo "✗ Checksum verification failed!"
    echo "  Expected: $expected"
    echo "  Got:      $actual"
    rm -f "$binary_path"
    return 1
  fi
  echo "  ✓ Checksum verified"
}

echo "Installing Konductor ($SCOPE scope) to $TARGET_DIR..."

# Create target directories with permission check
if ! mkdir -p "$TARGET_DIR/agents" "$TARGET_DIR/skills" "$TARGET_DIR/bin" 2>/dev/null; then
  echo "✗ Failed to create directory $TARGET_DIR — check permissions"
  echo "  Try: sudo bash install.sh $*"
  exit 1
fi

if [[ ! -w "$TARGET_DIR" ]]; then
  echo "✗ Directory $TARGET_DIR is not writable — check permissions"
  exit 1
fi

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
    *) echo "✗ Unsupported OS: $OS (supported: linux, macOS)"; exit 1 ;;
  esac

  case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo "✗ Unsupported architecture: $ARCH (supported: x86_64, arm64)"; exit 1 ;;
  esac

  BINARY_NAME="konductor-${PLATFORM}-${ARCH}"

  if [[ "$KONDUCTOR_VERSION" == "latest" ]]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"
  else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${KONDUCTOR_VERSION}/${BINARY_NAME}"
  fi

  echo "Downloading from: $DOWNLOAD_URL"

  if download "$DOWNLOAD_URL" "$KONDUCTOR_BINARY"; then
    if verify_binary "$KONDUCTOR_BINARY"; then
      CHECKSUM_URL="${DOWNLOAD_URL}.sha256"
      if verify_checksum "$KONDUCTOR_BINARY" "$CHECKSUM_URL"; then
        chmod +x "$KONDUCTOR_BINARY"
      else
        exit 1
      fi
    else
      exit 1
    fi
  else
    echo "⚠ Binary not available (no release found). Skipping."
    rm -f "$KONDUCTOR_BINARY"
  fi
fi

echo ""
echo "✓ Konductor installed successfully!"

# Ensure konductor binary is on PATH
BIN_DIR="$TARGET_DIR/bin"
if ! command -v konductor >/dev/null 2>&1; then
  # Add to PATH for current session
  export PATH="$BIN_DIR:$PATH"

  if [[ "$SCOPE" == "global" ]]; then
    # Add to shell profile for future sessions
    SHELL_NAME=$(basename "${SHELL:-/bin/bash}")
    case "$SHELL_NAME" in
      zsh)  PROFILE="$HOME/.zshrc" ;;
      fish) PROFILE="$HOME/.config/fish/config.fish" ;;
      *)    PROFILE="$HOME/.bashrc" ;;
    esac

    PATH_LINE="export PATH=\"$BIN_DIR:\$PATH\""
    if [[ -f "$PROFILE" ]] && grep -qF "$BIN_DIR" "$PROFILE" 2>/dev/null; then
      : # Already in profile
    elif [[ -f "$PROFILE" ]]; then
      echo "" >> "$PROFILE"
      echo "# Konductor" >> "$PROFILE"
      echo "$PATH_LINE" >> "$PROFILE"
      echo "  Added $BIN_DIR to PATH in $PROFILE"
      echo "  Run: source $PROFILE (or restart your shell)"
    else
      echo "  ⚠ Add $BIN_DIR to your PATH manually:"
      echo "    $PATH_LINE"
    fi
  else
    echo "  ⚠ Local install: add .kiro/bin to PATH in your project, e.g.:"
    echo "    export PATH=\".kiro/bin:\$PATH\""
  fi
fi
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
echo "  state_resolve_blocker, plans_list, status, config_get"
echo ""
echo "For more information, see: https://github.com/${REPO}"
echo ""
echo "Alternative: npm install -g konductor"
