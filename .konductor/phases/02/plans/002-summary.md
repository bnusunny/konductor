# Plan 002 Summary: CI Pipeline — Tests, Checksums, and Version Sync

## Status: Complete

## Changes
- Added `test` job to build-release.yml that runs `cargo test` and `cargo clippy -- -D warnings`
- Build job now `needs: test` — tests gate the release
- Added checksum generation step: creates .sha256 file per binary
- Added checksum upload step: uploads .sha256 alongside each binary
- Synced Cargo.toml version from 0.1.0 → 0.2.1 to match version.txt
- `konductor --version` now reports 0.2.1
