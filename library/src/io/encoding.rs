use crate::error::{Result, XmlError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Utf8,
    Utf8Bom,
    Utf16Be,
    Utf16Le,
    Utf32Be,
    Utf32Le,
}

impl Default for Format {
    fn default() -> Self {
        Format::Utf8
    }
}

pub fn detect_bom(bytes: &[u8]) -> (Format, usize) {
    if bytes.len() >= 4 {
        match (bytes[0], bytes[1], bytes[2], bytes[3]) {
            (0x00, 0x00, 0xFE, 0xFF) => return (Format::Utf32Be, 4),
            (0xFF, 0xFE, 0x00, 0x00) => return (Format::Utf32Le, 4),
            _ => {}
        }
    }
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return (Format::Utf8Bom, 3);
    }
    if bytes.len() >= 2 {
        match (bytes[0], bytes[1]) {
            (0xFE, 0xFF) => return (Format::Utf16Be, 2),
            (0xFF, 0xFE) => return (Format::Utf16Le, 2),
            _ => {}
        }
    }
    (Format::Utf8, 0)
}

pub fn decode_to_utf8(bytes: &[u8], format: Format, bom_offset: usize) -> Result<String> {
    let payload = &bytes[bom_offset..];
    match format {
        Format::Utf8 | Format::Utf8Bom => {
            String::from_utf8(payload.to_vec())
                .map_err(|e| XmlError::SyntaxError {
                    message: format!("Invalid UTF-8 sequence: {e}"),
                    line: 1,
                    col: 1,
                })
        }
        Format::Utf16Be => {
            if payload.len() % 2 != 0 {
                return Err(XmlError::SyntaxError {
                    message: "Invalid UTF-16 BE byte length".into(),
                    line: 1,
                    col: 1,
                });
            }
            let u16s: Vec<u16> = payload
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&u16s).map_err(|e| XmlError::SyntaxError {
                message: format!("Invalid UTF-16 BE sequence: {e}"),
                line: 1,
                col: 1,
            })
        }
        Format::Utf16Le => {
            if payload.len() % 2 != 0 {
                return Err(XmlError::SyntaxError {
                    message: "Invalid UTF-16 LE byte length".into(),
                    line: 1,
                    col: 1,
                });
            }
            let u16s: Vec<u16> = payload
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&u16s).map_err(|e| XmlError::SyntaxError {
                message: format!("Invalid UTF-16 LE sequence: {e}"),
                line: 1,
                col: 1,
            })
        }
        Format::Utf32Be | Format::Utf32Le => {
            if payload.len() % 4 != 0 {
                return Err(XmlError::SyntaxError {
                    message: "Invalid UTF-32 byte length".into(),
                    line: 1,
                    col: 1,
                });
            }
            let chars: Result<Vec<char>> = payload
                .chunks_exact(4)
                .map(|c| {
                    let code = if format == Format::Utf32Be {
                        u32::from_be_bytes([c[0], c[1], c[2], c[3]])
                    } else {
                        u32::from_le_bytes([c[0], c[1], c[2], c[3]])
                    };
                    char::from_u32(code).ok_or_else(|| XmlError::SyntaxError {
                        message: format!("Invalid Unicode codepoint: {code:#X}"),
                        line: 1,
                        col: 1,
                    })
                })
                .collect();
            Ok(chars?.into_iter().collect())
        }
    }
}

pub fn normalize_line_endings(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            normalized.push('\n');
        } else {
            normalized.push(c);
        }
    }
    normalized
}
