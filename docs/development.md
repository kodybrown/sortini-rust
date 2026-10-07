# Development

Use Rust 1.95 or newer and Cargo with `rustfmt` and `clippy`. The project uses
edition 2024 and a committed `Cargo.lock`. Focus is Windows for now; CI runs on
`windows-latest`. Build, test, benchmark, and packaging entry points are Cargo
commands. Make is not used, and dotask is deferred until it stabilizes.

## Checkout workflow

Work directly in `C:\Users\kodyb\Projects\sortini-rust` on `develop`. Do not
create disposable task worktrees or AI task branches for routine work. Check the
directory, branch, and status before edits, preserve unrelated changes, and commit
only scoped, verified work on `develop`. Promotion to `main` and pushes require
an explicit request.

## Verification

From the main project directory on `develop`:

```powershell
cd C:\Users\kodyb\Projects\sortini-rust
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
cargo build --locked --release
git diff --check
git diff --cached --check
```

The property suites configure 1,536 generated encoding/byte-preservation/EOL/
idempotence cases and 2,048 comparator cases with a reproducible seed and shrinking.
Persisted regression seeds replay earlier failures. Separate
regressions pin the legacy comparison-cycle correction. Process tests cover
`--check`, readonly files, exact-byte comparisons, multiple-file error precedence,
stdin, CLI prefixes/aliases, environment precedence, and ordinary file output.

File-symlink creation may require Developer Mode or elevated privileges. The
conditional symlink test explicitly reports a skip when Windows returns error
1314; hard-link preservation is tested separately. Do not change machine security
settings just to make that test pass.

Interactive acceptance in this task exercised `-pause always`, `-pause iferror`
after a read failure, successful `-pause iferror`, and check-result behavior in a
real terminal. Check status 1 does not trigger `iferror`; a missing terminal input
in check mode returns 2. Piped input never waits for a pause.

## C# executable comparison

The source has been archived. Use the supplied executable directly:

```powershell
$env:SORTINI_CSHARP_EXE = 'C:\Users\kodyb\Bin\sortini_cs.exe'
cargo test --locked --release --test parity -- --ignored --nocapture
Remove-Item Env:SORTINI_CSHARP_EXE
```

The normal suite ignores this one machine-specific test. It pins the supplied
June build's SHA-256 and checks 308 scenarios: 200 stdout cases plus 108 file-byte
cases. Both apps receive explicit `--expand` before any scenario-specific flags,
because this older build unexpectedly expands formatting by default. Numeric
prefix families are compared independently; Rust's corrected mixed-prefix order
has separate golden and comparator regressions.

There are 260 exact matches and 48 explicitly asserted legacy difference cases.
The difference cases include 36 BOM additions on `utf8`/`utf32` overrides and 20
UTF-16/32 CRLF detection failures, with 8 overlapping cases. Both programs' file
bytes are checked against independently constructed expectations; mismatches are
not hidden by stripping BOMs or normalizing EOLs. See [parity](parity.md).

## Benchmark

```powershell
cargo build --locked --release
cargo run --locked --release --example benchmark -- --reference C:\Users\kodyb\Bin\sortini_cs.exe --report target\benchmark-windows.md
```

The example generates bounded deterministic INI files, validates the two apps'
outputs, then alternates their execution for seven warmed samples per workload.
Use `--iterations` for an odd count from 3 to 31, and `--rust` to select another
release executable. Each sample has a 30-second limit. Timed stdout goes to NUL.
Peak working-set counters are sampled through Win32, waiting on the process
handle so short runs do not incur a polling-sleep tick after exit.

[Recorded results](benchmark-windows.md) include executable hashes, input sizes,
median wall times, end-to-end throughput, approximate process memory, and limits.
These are synthetic local results and exclude write/backup/fsync costs. No
performance optimization was introduced based solely on presumed speed.

## Local Windows package

```powershell
cargo build --locked --release
cargo run --locked --release --example package
```

The default output is
`target/release/distribution/sortini-rust-4.0.0-windows-x86_64.zip` on this x64
machine, with an adjacent `.zip.sha256` file. The ZIP contains `sortini_rust.exe`,
`README.txt`, `LICENSE`, and `SHA256SUMS.txt`. Entries are read back and compared to
their source bytes, and the restored executable is smoke-tested for version and
nonmutating check behavior before success is reported. The utility refuses to overwrite
an existing package; choose a new `--output-dir` to prepare another.

Packaging does not copy into the user Bin folder, modify PATH, create a GitHub
release, or publish anywhere. CI runs the same Cargo commands and prepares a
Windows ZIP after a push; it has not been executed remotely during this task.

## Library

```rust
use sortini::{Options, sort_text};

let output = sort_text("[s]\nz=2\na=1\n", &Options::default());
assert_eq!(output, "[s]\na=1\nz=2\n");
```

Use `sort_bytes` for encoding-aware callers. It returns a `Result` and preserves
source encoding/BOM unless overridden. Library callers can implement their own
check by comparing the returned bytes with the source bytes.
