# sortini-rust

A Rust implementation of [C# sortini](https://github.com/kodybrown/sortini).
The executable is named **sortini**, and implements the v4.0 option set, including
single-dash long options, double-dash options, slash options, boolean `!` overrides,
and `sortini_*` environment variables.

Sort sections and entries together or independently, ascending or descending,
with natural numeric sorting and separate case settings. Comments, disabled
entries, inline comments, spacing, encoding/BOM, and line endings are preserved
by default. Expand/compact switches explicitly normalize formatting.

## Build and run

Requires Rust 1.95 or newer, with Cargo. No .NET runtime is needed.

```powershell
cargo build --release --locked
cargo run --locked -- --help
cargo run --locked -- --stdout settings.ini
cargo run --locked -- settings.ini -o sorted.ini
cargo run --locked -- -expand -sort-desc settings.ini
Get-Content settings.ini -Raw | cargo run --locked -- -!sort-sections
```

Development and verification currently focus on Windows. The release executable
is `target/release/sortini.exe`. Cargo's standard `target/` directory is
ignored; `CARGO_TARGET_DIR` is supported if you prefer external Rust artifacts.
PowerShell's text pipelines may transcode input before sortini receives it; use
file arguments when exact source bytes matter.

Prepare a Windows ZIP with a distinct `sortini_rust.exe` name to coexist with
`sortini_cs.exe`:

```powershell
cargo build --release --locked
cargo run --release --locked --example package
```

## Examples

```text
sortini settings.ini                      # sort in place
sortini one.ini two.ini                    # process multiple files
sortini settings.ini -stdout               # print without changing the file
sortini settings.ini -o sorted.ini         # write a separate file
sortini settings.ini /sort-desc /!natural  # slash flags and boolean negation
sortini settings.ini --check               # 0 unchanged, 1 changes needed; no writes
sortini settings.ini -!sort-sections        # sort entries only
sortini settings.ini -!sort-entries         # sort sections only
sortini settings.ini -case-sensitive       # case-sensitive comparison
sortini settings.ini -expand               # normalize spaces and section breaks
sortini settings.ini -encoding utf8 -eol \n # override file encoding and EOL
sortini -- --help                          # filename that resembles an option
```

Piped input defaults to stdout unless `-o` is supplied. `-stdout` takes precedence
over `-o` and emits UTF-8 text without a BOM; `-encoding` applies to file output,
matching the C# output behavior.

## Verification and design

```powershell
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --release
cargo build --locked --release
```

These commands check Rust formatting, Clippy warnings, Debug/Release tests, and
the release build. Use `git diff --check` for whitespace. Builds use Cargo
directly; Make and dotask are not used. The optional C# executable comparison is
documented separately.

- [Development and verification](docs/development.md)
- [CLI behavior and parity decisions](docs/parity.md)
- [Architecture](docs/design.md)
- [Windows benchmark results](docs/benchmark-windows.md)
- [Changelog](docs/CHANGELOG.md)
- [MIT license](LICENSE)
