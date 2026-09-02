//! # XML Input Source
//!
//! Provides the [`XmlSource`] abstraction reading from string buffers, byte slices, or disk files with line/col tracking.

use crate::error::{Result, XmlError};
use crate::io::encoding::detect_encoding_and_strip_bom;
pub use crate::io::encoding::Format;

/// Input source abstraction providing character positioning and BOM auto-detection.
#[derive(Debug, Clone)]
pub struct XmlSource {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    format: Format,
}

impl XmlSource {
    /// Creates an `XmlSource` from an in-memory string slice.
    pub fn from_string(xml: &str) -> Self {
        let normalized = xml.replace("\r\n", "\n").replace('\r', "\n");
        Self {
            chars: normalized.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            format: Format::Utf8,
        }
    }

    /// Creates an `XmlSource` from raw bytes, auto-detecting UTF-8/UTF-16 BOM markers.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (content, format) = detect_encoding_and_strip_bom(bytes)?;
        let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
        Ok(Self {
            chars: normalized.chars().collect(),
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

    /// Returns the current character position index.
    pub fn position(&self) -> usize {
        self.pos
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
        self.pos >= self.chars.len()
    }

    /// Peeks at the current character without advancing cursor position.
    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Peeks at character `offset` steps ahead.
    pub fn peek_offset(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    /// Advances and returns the next character, updating line and column counters.
    pub fn next_char(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
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
        let prefix_chars: Vec<char> = prefix.chars().collect();
        if self.pos + prefix_chars.len() > self.chars.len() {
            return false;
        }
        self.chars[self.pos..self.pos + prefix_chars.len()] == prefix_chars[..]
    }

    /// Consumes expected prefix string if present at current position.
    pub fn consume(&mut self, prefix: &str) -> bool {
        if self.starts_with(prefix) {
            for _ in 0..prefix.chars().count() {
                self.next_char();
            }
            true
        } else {
            false
        }
    }

    /// Skips all leading ASCII whitespace characters.
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }
}
