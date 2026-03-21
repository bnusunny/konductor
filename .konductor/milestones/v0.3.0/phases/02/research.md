# Phase 02 Research: Install and Distribution

## Current State Analysis

### install.sh
**What exists:**
- Scope selection (--global/-g, --local/-l) with default global
- --force flag for overwriting existing files
- safe_cp helper to skip existing files
- Copies agents, skills, hooks from source directory
- Binary install: prefers local build, falls back to GitHub release download
- Platform detection: linux/darwin, x86_64/arm64
- curl/wget fallback for downloads
- Graceful skip when binary not available

**Gaps identified:**
1. No checksum verification after download — binary integrity not validated
2. No permission error handling — `mkdir -p` and `cp` can fail silently on permission issues
3. No validation that downloaded binary is actually executable (could be an HTML error page)
4. Version mismatch: `version.txt` says 0.2.1, `Cargo.toml` says 0.1.0
5. No `--version` flag on install.sh itself
6. No `--help` flag
7. No uninstall guidance
8. The one-line installer (`docs/install`) doesn't pass through `--force` or version args properly

### docs/install (one-line installer)
- Downloads main branch tarball, extracts, runs install.sh
- Passes through `$@` args — this actually works for --force etc.
- No version pinning — always installs from main branch
- No error handling for network failures beyond curl/wget check

### CI/CD Workflows

**build-release.yml:**
- Triggers on release published
- Builds 4 targets: linux x86_64, linux arm64, macos x86_64, macos arm64
- Uses svenstaro/upload-release-action to attach binaries
- No `cargo test` step before build — could release broken code
- No checksum generation — no .sha256 files uploaded alongside binaries

**release-please.yml:**
- Simple release-type, reads version from version.txt
- No Cargo.toml version sync — version.txt and Cargo.toml can drift

**conventional-commits.yml:**
- PR title linting only — works fine

### Version Reporting (REQ-06)
- `konductor --version` prints Cargo.toml version (0.1.0)
- `version.txt` says 0.2.1
- These are out of sync — version.txt drives releases, Cargo.toml drives --version
- Need to sync Cargo.toml version with version.txt, or read version.txt at build time

## Recommendations

1. Add checksum generation to build-release workflow + verification to install.sh
2. Add `cargo test` step to build-release workflow before building
3. Sync Cargo.toml version with version.txt
4. Add error handling to install.sh for permissions and download validation
5. Add --help flag to install.sh
