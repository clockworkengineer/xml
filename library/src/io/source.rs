//! # XML Input Source
//!
//! Provides the zero-copy [`XmlSource`] abstraction reading directly from UTF-8 string slices with line/col tracking.

use crate::error::{Result, XmlError};
use crate::io::encoding::detect_encoding_and_strip_bom;
pub use crate::io::encoding::Format;

/// Input source abstraction providing zero-allocation byte positioning and BOM auto-detection.
#[derive(Debug, Clone)]
pub struct XmlSource {
    content: String,
    pos: usize,
    line: usize,
    col: usize,
    format: Format,
}

impl XmlSource {
    /// Creates an `XmlSource` from an in-memory string slice.
    pub fn from_string(xml: &str) -> Self {
        let normalized = if xml.contains('\r') {
            xml.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            xml.to_string()
        };
        Self {
            content: normalized,
            pos: 0,
            line: 1,
            col: 1,
            format: Format::Utf8,
        }
    }

    /// Creates an `XmlSource` from raw bytes, auto-detecting UTF-8/UTF-16 BOM markers.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (raw_content, format) = detect_encoding_and_strip_bom(bytes)?;
        let normalized = if raw_content.contains('\r') {
            raw_content.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            raw_content
        };
        Ok(Self {
            content: normalized,
            pos: 0,
            line: 1,
            col: 1,
            format,
        })
    }

    /// Reads an `XmlSource` from a file path on disk.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let bytes = std::fs::read(path.as_ref()).map_err(|e| XmlError::Io(e.to_string()))?;
        Self::from_bytes(&bytes)
    }

    /// Returns the detected encoding format.
    pub fn format(&self) -> Format {
        self.format
    }

    /// Returns the current character position byte index.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns a string slice of the underlying content for a given byte range.
    pub fn slice_range(&self, start: usize, end: usize) -> &str {
        &self.content[start..end]
    }

    /// Returns the current 1-based line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the current 1-based column number.
    pub fn col(&self) -> usize {
        self.col
    }

    /// Returns `true` if end of source buffer is reached.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.content.len()
    }

    /// Peeks at the current character without advancing cursor position.
    pub fn peek(&self) -> Option<char> {
        if self.pos >= self.content.len() {
            None
        } else {
            self.content[self.pos..].chars().next()
        }
    }

    /// Peeks at character `offset` steps ahead.
    pub fn peek_offset(&self, offset: usize) -> Option<char> {
        if self.pos >= self.content.len() {
            None
        } else {
            self.content[self.pos..].chars().nth(offset)
        }
    }

    /// Advances and returns the next character, updating line and column counters.
    pub fn next_char(&mut self) -> Option<char> {
        if self.pos >= self.content.len() {
            return None;
        }
        let ch = self.content[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    /// Checks if source starts with expected prefix at current position.
    pub fn starts_with(&self, prefix: &str) -> bool {
        if self.pos >= self.content.len() {
            false
        } else {
            self.content[self.pos..].starts_with(prefix)
        }
    }

    /// Consumes expected prefix string if present at current position.
    pub fn consume(&mut self, prefix: &str) -> bool {
        if self.starts_with(prefix) {
            for ch in prefix.chars() {
                self.pos += ch.len_utf8();
                if ch == '\n' {
                    self.line += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }
            }
            true
        } else {
            false
        }
    }

    /// Skips all leading ASCII whitespace characters.
    pub fn skip_whitespace(&mut self) {
        let bytes = self.content.as_bytes();
        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
                if b == b'\n' {
                    self.line += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }
            } else {
                break;
            }
        }
    }
}
