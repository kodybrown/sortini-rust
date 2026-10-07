//! INI sorting with source-preserving defaults and explicit normalization.

pub mod cli;
mod compare;
pub mod encoding;
mod ini;
pub mod persistence;

use encoding::EncodedText;

/// Configuration shared by the library and executable.
#[derive(Clone, Debug)]
pub struct Options {
    pub sort_sections: bool,
    pub sort_entries: bool,
    pub descending: bool,
    pub natural: bool,
    pub ignore_case_sections: bool,
    pub ignore_case_entries: bool,
    /// Explicit formatting enables the same whole-document normalization as C#.
    pub normalize: bool,
    pub expand_lines: bool,
    pub expand_sections: bool,
    pub encoding: Option<encoding::Encoding>,
    pub eol: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sort_sections: true,
            sort_entries: true,
            descending: false,
            natural: true,
            ignore_case_sections: true,
            ignore_case_entries: true,
            normalize: false,
            expand_lines: false,
            expand_sections: false,
            encoding: None,
            eol: None,
        }
    }
}

/// Sort decoded text. This leaves preamble entries alone, as in C# sortini.
pub fn sort_text(text: &str, options: &Options) -> String {
    let mut document = ini::Document::parse(text);
    document.sort(options);
    document.export(options)
}

/// Decode strictly, sort, and encode using the original BOM and byte order unless overridden.
pub fn sort_bytes(bytes: &[u8], options: &Options) -> Result<Vec<u8>, encoding::EncodingError> {
    let source = EncodedText::decode(bytes)?;
    let output = sort_text(&source.text, options);
    options.encoding.unwrap_or(source.encoding).encode(&output)
}

pub const OS_EOL: &str = if cfg!(windows) { "\r\n" } else { "\n" };
