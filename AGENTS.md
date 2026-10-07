# Agent instructions

## Project and workflow

- Project: `sortini-rust`; Rust library crate `sortini` and binary `sortini`.
- Integration checkout: `C:\Users\kodyb\Projects\sortini-rust`.
- Integration branch: `main`.
- Behavioral work uses disposable task worktrees under `worktrees/ai/` and
  branches named `codex/<short-name>`, based on `main`.
- Keep follow-up work in the same task worktree. Commit only scoped, verified
  work. Integrate and clean up only after explicit acceptance; never push
  without an explicit request.
- Preserve the C# reference checkout at `C:\Users\kodyb\Projects\sortini`.
  Do not change its implementation or fixtures as part of Rust work.
- Changelog: `docs/CHANGELOG.md`; update it for every completed task.

## Implementation

- Read `README.md` and `docs/parity.md` before behavioral changes.
- Preserve source text and bytes by default, including blank-line whitespace,
  BOM, byte order, mixed line endings, and final-newline presence.
- Preserve comment attachment, disabled-entry sorting, duplicate sections,
  and preamble behavior. Keep sorting stable for equal keys/values.
- Keep the preferred `-`, `--`, `/`, and `!` CLI conventions. Options cannot
  be chained. Paths must remain `OsString`/`PathBuf`, without lossy conversion.
- Use enums, ownership, borrowing, `Result`, and standard Rust conventions;
  do not port the C# reflection or inheritance architecture.
- Keep intent and constraint comments accurate across changes.
- Do not rename over symlink destinations. Do not truncate input until
  decoding, sorting, and output encoding succeed.

## Verification

- Full gate: `pwsh -NoProfile -File scripts/verify.ps1`.
- Portable commands: `cargo fmt --all --check`,
  `cargo clippy --locked --all-targets -- -D warnings`,
  `cargo test --locked`, `cargo test --locked --release`,
  `cargo build --locked --release`, and `git diff --check`.
- Run parser/CLI/I/O tests for behavioral changes. For parity-sensitive changes,
  also run the optional C# comparison when the reference build is available;
  see `docs/development.md`.
- Report conditional symlink-test skips. Do not claim an interactive pause,
  unsupported platform, deployment, or installation was verified by unit tests.
- Rust artifacts use Cargo's ignored `target/` or `CARGO_TARGET_DIR`. Any .NET
  reference builds must retain the user's authoritative MSBuild output policy.
- Handoff must include exact `cd`, verification, and run commands for the task
  worktree, with committed versus integrated/pushed state made explicit.
