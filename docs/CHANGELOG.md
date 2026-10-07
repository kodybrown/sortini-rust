# Changelog

## Unreleased

### 2026-10-07 - Direct develop workflow

- Integrated the accepted check/reliability/Windows tooling commit `84fb8ed`
  with a fast-forward and removed its disposable task worktree and branch.
- Created `develop` from the integrated state and changed agent/development
  instructions to always use the main project directory directly on `develop`,
  replacing the task-worktree default. Promotion to `main` and pushes remain
  explicit actions.
- Clarified that ordinary implementation, documentation, and read-only work
  defaults to the main checkout on `develop`; worktrees require an explicit user
  request and must not be inferred from behavioral changes, task size, or duration.
- Preserved the reviewed Windows ZIP and checksum in
  `target/release/distribution/accepted-84fb8ed/`, with matching SHA-256 hashes.
- Verification: the full Cargo formatting, Clippy, Debug/Release test, and release
  build gate passed in the main directory; Git whitespace and final branch/worktree
  checks passed. The instruction change is documentation only.
  The follow-up policy clarification passed Git whitespace checks.

### 2026-10-07 - Check mode, reliability, and Windows tooling

- Added `--check` with exact byte comparison, nonmutating file/stdin processing,
  quiet status reporting, encoding/EOL/format overrides, and processing-error
  precedence across multiple inputs. Exit 0 means unchanged and 1 means changes
  are needed. Conflicting output options fail before I/O.
- Added generated tests for Unicode round trips, no-sort byte identity, arbitrary
  decoding, raw-line retention, normalization/idempotence, mixed EOLs, and
  comparator consistency, with a persisted failure seed and focused regressions.
- Fixed nontransitive natural comparison by assigning consistent prefix/decimal
  keys; documented intentional mixed-prefix ordering changes.
- Fixed a mixed-EOL sorting boundary that could merge CR and an LF blank line
  into one CRLF and remove that physical blank line.
- Switched the comparison oracle to the supplied Windows `sortini_cs.exe` and
  pinned its hash plus the older build's BOM/EOL differences. The archived C#
  source is no longer needed for comparisons.
- Added Cargo-run Windows benchmark and ZIP utilities. Benchmarks validate output
  before timing, alternate warmed samples, record executable hashes and sampled
  memory, and bound each sample. Packages include `sortini_rust.exe`, checksums,
  and license/readme; entries are read back and the restored executable is tested.
- Changed CI to Windows-only direct Cargo commands and removed the PowerShell
  build gate. Make is not used; dotask remains deferred.
- Verification: Cargo formatting, Clippy with warnings denied, Debug/Release
  tests and release builds passed. All 308 C# scenarios were checked: 260 exact
  matches and 48 explicitly asserted legacy difference cases. Windows package
  read-back and executable smoke tests passed; warmed benchmarks are recorded in
  `docs/benchmark-windows.md`.
- Real-terminal pause checks passed, including successful and failed `iferror`
  paths and check-mode status behavior. File-symlink creation still requires a
  Windows privilege unavailable in this session; that conditional body was skipped.

### Initial Rust implementation

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
