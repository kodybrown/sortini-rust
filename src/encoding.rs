use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Encoding {
    Utf8 { bom: bool },
    Utf16 { big_endian: bool, bom: bool },
    Utf32 { big_endian: bool, bom: bool },
    Ascii,
}

#[derive(Debug)]
pub struct EncodingError(&'static str);

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for EncodingError {}

impl Encoding {
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name.to_ascii_lowercase().as_str() {
            "utf8" | "os" => Ok(Self::Utf8 { bom: false }),
            "ascii" => Ok(Self::Ascii),
            "unicode" => Ok(Self::Utf16 {
                big_endian: false,
                bom: true,
            }),
            "utf32" => Ok(Self::Utf32 {
                big_endian: false,
                bom: false,
            }),
            _ => Err(format!(
                "Invalid value \"{name}\" for encoding. Allowed values: utf8, ascii, unicode, utf32, os"
            )),
        }
    }

    pub fn encode(self, text: &str) -> Result<Vec<u8>, EncodingError> {
        let mut bytes = Vec::new();
        match self {
            Self::Utf8 { bom } => {
                if bom {
                    bytes.extend_from_slice(&[0xef, 0xbb, 0xbf]);
                }
                bytes.extend_from_slice(text.as_bytes());
            }
            Self::Utf16 { big_endian, bom } => {
                if bom {
                    bytes.extend_from_slice(if big_endian {
                        &[0xfe, 0xff]
                    } else {
                        &[0xff, 0xfe]
                    });
                }
                for unit in text.encode_utf16() {
                    bytes.extend_from_slice(&if big_endian {
                        unit.to_be_bytes()
                    } else {
                        unit.to_le_bytes()
                    });
                }
            }
            Self::Utf32 { big_endian, bom } => {
                if bom {
                    bytes.extend_from_slice(if big_endian {
                        &[0, 0, 0xfe, 0xff]
                    } else {
                        &[0xff, 0xfe, 0, 0]
                    });
                }
                for c in text.chars() {
                    let unit = c as u32;
                    bytes.extend_from_slice(&if big_endian {
                        unit.to_be_bytes()
                    } else {
                        unit.to_le_bytes()
                    });
                }
            }
            // Explicit ASCII conversion follows .NET's replacement fallback.
            Self::Ascii => bytes.extend(
                text.encode_utf16()
                    .map(|c| if c < 128 { c as u8 } else { b'?' }),
            ),
        }
        Ok(bytes)
    }
}

pub struct EncodedText {
    pub text: String,
    pub encoding: Encoding,
}

impl EncodedText {
    pub fn decode(bytes: &[u8]) -> Result<Self, EncodingError> {
        use Encoding::*;
        // UTF-32 LE must precede UTF-16 LE because their preambles overlap.
        let (encoding, offset) = if bytes.starts_with(&[0xff, 0xfe, 0, 0]) {
            (
                Utf32 {
                    big_endian: false,
                    bom: true,
                },
                4,
            )
        } else if bytes.starts_with(&[0, 0, 0xfe, 0xff]) {
            (
                Utf32 {
                    big_endian: true,
                    bom: true,
                },
                4,
            )
        } else if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
            (Utf8 { bom: true }, 3)
        } else if bytes.starts_with(&[0xff, 0xfe]) {
            (
                Utf16 {
                    big_endian: false,
                    bom: true,
                },
                2,
            )
        } else if bytes.starts_with(&[0xfe, 0xff]) {
            (
                Utf16 {
                    big_endian: true,
                    bom: true,
                },
                2,
            )
        } else if bytes.len() >= 4 && bytes[..3] == [0, 0, 0] {
            (
                Utf32 {
                    big_endian: true,
                    bom: false,
                },
                0,
            )
        } else if bytes.len() >= 4 && bytes[1..4] == [0, 0, 0] {
            (
                Utf32 {
                    big_endian: false,
                    bom: false,
                },
                0,
            )
        } else if bytes.len() >= 4 && bytes[0] == 0 && bytes[2] == 0 {
            (
                Utf16 {
                    big_endian: true,
                    bom: false,
                },
                0,
            )
        } else if bytes.len() >= 4 && bytes[1] == 0 && bytes[3] == 0 {
            (
                Utf16 {
                    big_endian: false,
                    bom: false,
                },
                0,
            )
        } else {
            (Utf8 { bom: false }, 0)
        };
        let payload = &bytes[offset..];
        let text = match encoding {
            Utf8 { .. } => std::str::from_utf8(payload)
                .map_err(|_| EncodingError("Invalid UTF-8 input; file was not changed"))?
                .to_owned(),
            Utf16 { big_endian, .. } => {
                if !payload.len().is_multiple_of(2) {
                    return Err(EncodingError(
                        "Truncated UTF-16 input; file was not changed",
                    ));
                }
                let units = payload
                    .chunks_exact(2)
                    .map(|c| {
                        let pair = [c[0], c[1]];
                        if big_endian {
                            u16::from_be_bytes(pair)
                        } else {
                            u16::from_le_bytes(pair)
                        }
                    })
                    .collect::<Vec<_>>();
                String::from_utf16(&units)
                    .map_err(|_| EncodingError("Invalid UTF-16 input; file was not changed"))?
            }
            Utf32 { big_endian, .. } => {
                if !payload.len().is_multiple_of(4) {
                    return Err(EncodingError(
                        "Truncated UTF-32 input; file was not changed",
                    ));
                }
                payload
                    .chunks_exact(4)
                    .map(|c| {
                        let word = [c[0], c[1], c[2], c[3]];
                        let unit = if big_endian {
                            u32::from_be_bytes(word)
                        } else {
                            u32::from_le_bytes(word)
                        };
                        char::from_u32(unit)
                            .ok_or(EncodingError("Invalid UTF-32 input; file was not changed"))
                    })
                    .collect::<Result<String, _>>()?
            }
            Ascii => unreachable!(),
        };
        Ok(Self { text, encoding })
    }
}
