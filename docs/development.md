# Development

Use Rust 1.95 or newer and Cargo with the `rustfmt` and `clippy` components.
The project uses edition 2024. `Cargo.lock` is committed for repeatable executable
builds. The reference C# project is only needed for optional differential tests.
GitHub Actions runs the same gate on Windows, Linux, and macOS after pushes or
pull requests; that workflow is configured but has not run for this local task.

## Full verification

From the active task worktree:

```powershell
pwsh -NoProfile -File scripts/verify.ps1
cargo run --locked -- --help
```

The same gate without PowerShell:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
cargo build --locked --release
git diff --check
```

Tests cover golden fixtures, natural decimal comparison, stable duplicates,
comment attachment, disabled entries, raw/normalized output, Unicode/BOM/EOL
matrices, parser aliases and prefixes, environment precedence, actual subprocess
I/O, multiple errors, separate output, and backup recovery. On Windows, creating
file symlinks can require Developer Mode or elevated privileges; the conditional
test prints an explicit skip when that privilege is unavailable. Hard-link
preservation is tested separately without that requirement.

## Live C# comparison

Build the C# reference with its own output policy, then obtain its evaluated
assembly path. The test compares 200 stdout scenarios and 108 byte-level file
outputs against the built C# app, totaling 308 comparisons. The suite intentionally
uses well-formed files with uniform EOLs and final newlines to isolate compatible
behavior from documented preservation corrections.

```powershell
$referenceProject = 'C:\Users\kodyb\Projects\sortini\sortini.csproj'
dotnet build $referenceProject
if ($LASTEXITCODE -ne 0) { throw 'C# build failed' }
$env:SORTINI_CSHARP_DLL = (dotnet msbuild $referenceProject -getProperty:TargetPath).Trim()
cargo test --locked --test parity -- --ignored --nocapture
Remove-Item Env:SORTINI_CSHARP_DLL
```

The normal Rust gate marks this single oracle test ignored, so a C# checkout is
not required on contributors' machines. Changing sorting, parsing, or formatting
should run it when the reference is available. The C# fixtures retain their
original bytes, including BOM and line endings; do not autoformat them.

## Library use

```rust
use sortini::{Options, sort_text};

let output = sort_text("[s]\nz=2\na=1\n", &Options::default());
assert_eq!(output, "[s]\na=1\nz=2\n");
```

Use `sort_bytes` for encoding-aware callers. It returns a `Result` and preserves
source encoding/BOM unless `Options::encoding` is supplied.

## Manual acceptance

- Run the release executable on representative personal INI copies, including
  the option combinations you normally use.
- Check `-pause always` and `-pause iferror` from an interactive terminal.
- Run the symlink test on a machine permitted to create file symlinks.
- Linux/macOS behavior is covered by portable code and tests but must be run on
  those platforms before claiming local verification there.
