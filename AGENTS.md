# Agent instructions

## Project and workflow

- Project: `sortini-rust`; Rust library crate `sortini` and binary `sortini`.
- Integration checkout: `C:\Users\kodyb\Projects\sortini-rust`.
- Development and integration branch: `develop`.
- Ordinary work defaults to the main checkout on `develop`; worktrees require
  an explicit request from the user. This includes implementation, documentation,
  and read-only work in `C:\Users\kodyb\Projects\sortini-rust`.
- Do not create a worktree or AI feature branch based on behavioral changes,
  task size, or duration. Keep ordinary follow-up work in the main checkout on
  `develop`.
- Before editing, verify the exact directory, active branch, and Git status.
  Preserve unrelated changes; never stash or discard user work to change branches.
- Commit only scoped, verified work on `develop`. Promote to `main`, manage other
  branches/worktrees, or push only when explicitly requested.
- The C# source is archived. Use the supplied reference executable
  `C:\Users\kodyb\Bin\sortini_cs.exe` for comparisons and benchmarks; do not
  modify or replace it. Preserve the copied golden fixtures.
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
- `--check` never writes files or backups. Exit 0 means unchanged, 1 means changes
  are needed, and actual processing errors take precedence across multiple files.
- Natural comparison must define consistent keys for all labels. Never restore
  the legacy pair-dependent numeric/lexical fallback, which creates order cycles.
- Focus development and verification on Windows for now.
- Use Cargo directly. Do not add Make/Makefiles or dotask; dotask is deferred
  until explicitly requested after it stabilizes.

## Verification

- Full gate: `cargo fmt --all --check`,
  `cargo clippy --locked --all-targets -- -D warnings`,
  `cargo test --locked`, `cargo test --locked --release`,
  `cargo build --locked --release`, and `git diff --check`.
- Run parser/CLI/I/O tests for behavioral changes. For parity-sensitive changes,
  also run the optional C# comparison against the supplied executable;
  see `docs/development.md`.
- Report conditional symlink-test skips. Do not claim an interactive pause,
  unsupported platform, deployment, or installation was verified by unit tests.
- Rust artifacts use Cargo's ignored `target/` or `CARGO_TARGET_DIR`.
- Benchmark and package via `cargo run --release --example benchmark` and
  `cargo run --release --example package`; see `docs/development.md`. Package
  preparation is local only and does not install, publish, or modify PATH.
- Handoff must include exact `cd`, verification, and run commands for the main
  project directory on `develop`, with committed versus promoted/pushed state
  made explicit.
