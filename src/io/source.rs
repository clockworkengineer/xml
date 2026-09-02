use crate::error::{Result, XmlError};
use crate::io::encoding::{decode_to_utf8, detect_bom, normalize_line_endings, Format};

#[derive(Debug, Clone)]
pub struct XmlSource {
    buffer: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    format: Format,
}

impl XmlSource {
    pub fn from_string(input: &str) -> Self {
        let normalized = normalize_line_endings(input);
        Self {
            buffer: normalized.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            format: Format::Utf8,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (format, bom_offset) = detect_bom(bytes);
        let utf8_str = decode_to_utf8(bytes, format, bom_offset)?;
        let normalized = normalize_line_endings(&utf8_str);
        Ok(Self {
            buffer: normalized.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            format,
        })
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.buffer.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.buffer.get(self.position).copied()
    }

    pub fn peek_ahead(&self, count: usize) -> Option<char> {
        self.buffer.get(self.position + count).copied()
    }

    pub fn next_char(&mut self) -> Option<char> {
        let ch = self.buffer.get(self.position).copied()?;
        self.position += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        if self.position + prefix_chars.len() > self.buffer.len() {
            return false;
        }
        &self.buffer[self.position..self.position + prefix_chars.len()] == prefix_chars.as_slice()
    }

    pub fn consume_prefix(&mut self, prefix: &str) -> bool {
        if self.starts_with(prefix) {
            for _ in 0..prefix.chars().count() {
                self.next_char();
            }
            true
        } else {
            false
        }
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    pub fn syntax_error(&self, message: impl Into<String>) -> XmlError {
        XmlError::SyntaxError {
            message: message.into(),
            line: self.line,
            col: self.column,
        }
    }
}
