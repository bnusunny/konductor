# Plan 001 Summary: Install Script Hardening

## Status: Complete

## Changes
- Added `usage()` function and `--help|-h` flag
- Added `verify_binary()` — validates downloaded file is ELF/Mach-O using `file` command
- Added `verify_checksum()` — downloads .sha256 file, verifies with sha256sum/shasum
- Added `download()` helper — consolidates curl/wget logic
- Added permission checks after mkdir with actionable error messages
- Improved error messages for unsupported OS/arch with supported values listed
- Checksum verification is optional — warns if no .sha256 file available
