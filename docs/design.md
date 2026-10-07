# Architecture

The library provides `sort_text` and `sort_bytes`, configured by a Rust `Options`
value. The binary owns argument/environment parsing, terminal detection, input
selection, error reporting, pause behavior, and file output.

## Source representation

The INI parser borrows raw text, line endings, section names, keys, values, and
inline comments from the decoded source. An enum distinguishes blank lines,
comments, section headers, active/disabled entries, plain text, and malformed
headers. The source remains alive throughout sorting and export.

Each section owns its header and ordered body. Before the first section, all
lines are preamble and stay in their original order. Entry groups own their
preceding non-entry lines. Stable `sort_by` moves groups and whole sections while
preserving equal comparisons and duplicates. Trailing non-entry lines remain at
the end of their section, matching the C# model.

Raw export preserves physical line endings. A missing final newline belongs to
the document, rather than to the original last entry: if that entry moves into
the middle, it receives a separator and the new last line has no final newline.
Explicit normalization reconstructs headers and entries, removes existing blank
lines, and generates requested section spacing using the detected or overridden
EOL. It retains comments and unrecognized lines.

## Comparison

[ICU4X root collation](https://docs.rs/icu_collator/2.2.1/icu_collator/struct.Collator.html)
provides invariant-style Unicode text comparison, with secondary strength for
case-insensitive and tertiary strength for case-sensitive comparisons. This
matches tested C# invariant-culture ordering more closely than UTF-8 byte order
or lowercasing. Collators are created once per document sort, not per comparison.

[Exact decimal arithmetic](https://docs.rs/rust_decimal/1.43.0/rust_decimal/struct.Decimal.html)
handles whole numbers and trailing decimal suffixes without floating-point
precision loss. Natural comparison first checks whole-string numbers, then
matching text prefixes with numeric suffixes. C# retains a minus sign in the
suffix's prefix; this implementation does too. Numeric parsing uses deterministic
dot decimals and comma grouping rather than the process locale.

## Encoding and persistence

Encoding is an enum describing Unicode width, byte order, and BOM, or an explicit
ASCII override. Decoding rejects malformed UTF-8/16/32 before file writes. BOMs
are detected longest first. Initial ASCII syntax also identifies BOM-less
UTF-16/32. Valid source encoding is preserved unless overridden.

The complete output is encoded in memory before persistence. Existing targets
receive a uniquely named recovery backup in the same directory. Backup failure
aborts the write. The target is opened and truncated directly, preserving symlink
and hard-link identity. Successful writes are flushed to disk before backup
cleanup; failed writes retain the original bytes in the backup and report its
path. This is recoverable writing, not atomic replacement. An interrupted write
can leave a partial destination requiring recovery from the backup.

Stdin is read directly into memory, avoiding the C# temporary-file lifecycle.
Multiple file failures accumulate as independent diagnostics, while processing
continues. The first error code becomes the process exit status.
