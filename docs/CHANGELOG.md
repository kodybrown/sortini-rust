# Changelog

## Unreleased

### Added

- Rust edition 2024 `sortini` binary and reusable library, implementing C#
  sortini v4.0 sorting, comparison, formatting, encoding, EOL, file/stdin/stdout,
  and information options.
- Preferred `-`, `--`, `/`, and boolean `!` CLI conventions, aliases, environment
  precedence, inline values, and literal-path `--` support.
- Borrowed INI syntax enum, stable section/entry groups, exact natural decimal
  comparison, and invariant-style Unicode root collation.
- Strict UTF-8/16/32 decoding, BOM/byte-order preservation, raw blank-line and
  final-newline preservation, mixed EOL retention, and explicit normalization.
- Direct stdin processing, multiple-error reporting, and recovery backups with
  writes that preserve existing file/link identity.
- Original C# golden fixtures, library/parser/process/persistence tests, an
  optional 308-case live C# comparison, a PowerShell verification gate, and
  a Windows/Linux/macOS GitHub Actions workflow.
- Project instructions and development, design, and parity documentation,
  including intentional corrections to legacy defects.

### Verification

- `pwsh -NoProfile -File scripts/verify.ps1` passed on Windows with Rust 1.95.0:
  formatting, Clippy with warnings denied, Debug/Release tests, release build,
  and Git whitespace checks.
- All 308 direct comparisons passed against C# commit
  `6fc204f0cce0b87bff068ae06be3da29d2ea1ac5` (200 stdout and 108 file-byte cases).
- The C# reference's 83 tests passed; its checkout remained clean.
- Failed-write recovery and hard-link preservation passed. Windows refused
  file-symlink creation with error 1314, so that conditional test body was skipped.
