# CLI and C# parity

Reference: C# sortini v4.0, commit
`6fc204f0cce0b87bff068ae06be3da29d2ea1ac5`, inspected on 2026-10-07.
The reference implementation, not only its help text, defines the behavior tested
by the optional direct comparison suite. Four original input/expected fixture
pairs are copied byte-for-byte into `tests/fixtures/`.

The source is now archived. Ongoing comparisons use
`C:\Users\kodyb\Bin\sortini_cs.exe`, version
`4.0.2605.1521+d2259ec3a087e788bce9905063933ab9522f93e5`, SHA-256
`0752682ea7e13b49c798df03e9aa7961fbbd5f33f5785e7a2c1fb87f519fd714`.
This June binary predates the refreshed source: it expands formatting by default,
adds BOMs for explicit `utf8`/`utf32`, misdetects Unicode CRLF, and crashes on
redirected help. These observed differences do not change Rust's preservation
defaults. The executable suite compares explicit formatting and pins the legacy
BOM/EOL defects as distinct byte expectations.

## Supported options

All names below accept `-`, `--`, and `/` prefixes and case-insensitive option
names. Boolean flags default to true when supplied, and `!` makes their value
false (`-!sort`). False values on one-way switches such as `case-sensitive` or
`ordinal` are no-ops, as in C#. Later flags apply after earlier flags.

| Options / aliases | Behavior |
| --- | --- |
| `o`, `out-file` | Output file; only one input permitted |
| `stdout`, `redirect-to-stdout` | Text to stdout, leaving input unchanged |
| `check` | Check exact output bytes without writing; 0 unchanged, 1 changes needed |
| `sort` | Set both section and entry sorting; default true |
| `sort-sections`, `sort-entries` | Set each sort independently; defaults true |
| `sort-asc`, `sort-desc` | Ascending (default) or descending |
| `natural`, `ordinal` | Natural numeric sorting (default) or disable it |
| `ignore-case`, `i` | Case-insensitive sections and entries (default) |
| `case-sensitive`, `t` | Case-sensitive sections and entries |
| `ignore-case-sections`, `ignore-case-entries` | Case-insensitive comparison for one scope |
| `case-sensitive-sections`, `case-sensitive-entries` | Case-sensitive comparison for one scope |
| `expand`, `compact` | Normalize lines and section spacing together |
| `expand-lines`, `compact-lines` | Normalize spaces around `=` |
| `expand-sections`, `compact-sections` | Normalize section blank lines |
| `encoding` | File output: `utf8`, `ascii`, `unicode`, `utf32`, `os` |
| `eol`, `end-of-line` | EOL: literal `\n`, `\r\n`, `\r`, `os` |
| `help`, `?`, `h` | Help, with optional option name, `examples`, or `envars` |
| `about`, `v`, `version` | App details, short version, full version |
| `quiet` | Suppress extra information; never suppress data or errors |
| `pause` | `never` (default), `always`, `iferror`; bare flag means always |
| `show-examples`, `examples` | Examples, also supported by C# as hidden aliases |
| `show-envars`, `envars` | Environment listing, also supported by C# as hidden aliases |

No chained short options. Rust also supports `--option=value` and a standalone
`--` to stop option parsing. A recognized `/option` is a flag on every platform;
other slash-prefixed strings are paths. Use `-- /help` for a literal `/help`
filename. Unknown dash-prefixed options are errors instead of being treated as
filenames. Ordinary filenames, including non-Unicode native paths, are preserved.

## Environment

These exact variable names are supported:

```text
sortini_quiet
sortini_pause
sortini_sort
sortini_sort_sections
sortini_sort_entries
sortini_natural
sortini_ordinal
sortini_ignore_case
sortini_case_sensitive
sortini_ignore_case_sections
sortini_ignore_case_entries
sortini_case_sensitive_sections
sortini_case_sensitive_entries
sortini_expand_lines
sortini_expand_sections
sortini_encoding
sortini_eol
```

Environment applies in the C# declaration order above, after defaults and before
CLI options. Explicit CLI aliases suppress the same option's environment value,
including invalid values. Scope-specific environment settings override combined
settings. CLI options then apply in argument order. Empty environment variables
are ignored. Boolean values accept `true/t/yes/y/1` and `false/f/no/n/0`, ignoring
case and surrounding whitespace. Other values are rejected with a diagnostic.

## Important legacy semantics

1. Preamble entries are not sorted; sorting starts inside explicit sections.
2. All non-entry lines preceding an entry move with it, including comments,
   blank lines, plain text, and malformed headers. Trailing lines stay last.
3. Disabled entries have `;` or `#` immediately followed by text containing `=`.
   A literal space after the prefix makes the line a regular comment.
4. Inline comments begin at a space followed by `;` or `#` outside single or
   double quotes. Comment characters without that space are part of the value.
5. Entry comparison uses key, then value. Natural sorting recognizes complete
   decimal numbers and trailing numeric suffixes, rather than every digit run.
   Rust now derives prefix/number keys consistently; see the correction below.
6. Despite its description, C# `ordinal` only disables natural numbers. It keeps
   invariant-culture text comparison. Rust preserves this behavior.
7. Any expand/compact flag enables whole-document normalization. Line-only
   switches therefore also reconstruct headers and remove blank lines unless
   section expansion is enabled; section-only switches compact `=` spacing unless
   line expansion is enabled. This is observed C# behavior, even though one C#
   example says compact-lines preserves section spacing.
8. `unicode` writes UTF-16 LE with BOM; `utf32` writes UTF-32 LE without BOM;
   `utf8` and `os` write UTF-8 without BOM. `ascii` explicitly replaces
   unrepresentable UTF-16 units with `?`, matching C#.
9. `stdout` wins over `out-file`; stdout emits UTF-8 text, not the file encoding
   override. Multiple input files are emitted consecutively without labels.

## Deliberate corrections and platform limits

- Natural comparison no longer switches between numeric and full-text ordering
  depending on the other label. That C# rule creates cycles such as
  `-2 < -1 < -1a < -2`, invalid for Rust's sorting contract. Each label now gets a
  `(text prefix, optional decimal)` key: complete numbers have an empty prefix,
  suffix numbers retain their prefix, and other labels use their entire text.
  Prefixes compare by root collation, then absent/present numbers and exact
  decimal values. This changes some mixed-prefix orders: `item1.10` now precedes
  `item-10`. `ordinal` retains full-text collation without numeric keys.

- Raw blank-line whitespace, mixed EOLs, and missing final newline are preserved.
  C# removes blank-line whitespace, normalizes to the first EOL, and appends a
  final newline. Explicit Rust formatting still normalizes these.
- If sorting places an LF blank line after a CR-ending entry, Rust widens that
  boundary's CR to CRLF so the two endings cannot merge and erase the blank line.
  Other physical EOLs remain unchanged, and a second sort is byte-identical.
- Equal comparisons use stable sorting, preventing duplicate entry/section
  reordering from C#'s unstable `List.Sort`.
- BOM-less Unicode decoding works for recognizable initial ASCII syntax.
  Malformed source bytes are rejected without overwriting the input.
- Stdin is decoded directly from bytes, preserving Unicode input without a
  temporary file. Empty piped input succeeds as an empty INI document.
- Repeated input failures report every affected file and continue processing;
  C# can fail while adding duplicate error codes to its dictionary.
- File writes require a unique recovery backup for existing targets. C# uses a
  fixed backup filename and proceeds even if backup creation fails.
- Invalid environment values fail clearly. C# silently treats unknown boolean
  environment values as false, and skips some invalid enum settings.
- Numeric parsing is deterministic across locales. Unicode text ordering uses
  the pinned ICU4X root data; C# runtime ICU/NLS versions can differ for unusual
  characters. Direct comparisons cover accents, symbols, Greek, emoji, and case.
- Pause waits for Enter when stdin is a terminal. Piped input never pauses.
  Interactive terminal behavior requires manual acceptance.
- `quiet` is effectively silent during ordinary file processing in both apps;
  explicitly requested information and sorted stdout remain visible.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Success; in check mode, every input is byte-identical to its configured output |
| 1 | Check mode needs changes; ordinary mode has no input file and stdin is a terminal |
| 2 | Invalid/missing option/environment value, conflicting check output options, or missing check input |
| 10 | Read, decode, or input processing failure |
| 13 | Write/output failure |
| 99 | Multiple input files with `out-file` |

Rust uses a portable positive code 2 for parse failures rather than the C#
binder's negative codes. Errors go to stderr. An INI-producing stdout stream
never contains error text or pause prompts.

## Check mode

`--check` runs the same decode/sort/encode pipeline as file output and compares
the exact bytes. Encoding, BOM, EOL, sorting and formatting options all count.
It reads files or stdin, creates no output or backup, and prints per-input status
to stderr. `--quiet` suppresses statuses but keeps errors. Multiple files are all
checked; an actual processing failure overrides the changed status regardless of
input order. `--stdout` and `--out-file` conflict with check mode and are rejected
before input processing. `-!check` restores ordinary behavior. A positive check
result does not trigger `-pause iferror`.
