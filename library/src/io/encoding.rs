//! # Character Encoding & BOM Auto-Detection
//!
//! Auto-detects UTF-8 / UTF-16 Byte Order Marks (BOM) and decodes raw byte slices into UTF-8 string buffers.

use crate::error::{Result, XmlError};

/// Detected character encoding format and BOM configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Standard UTF-8 without BOM.
    Utf8,
    /// UTF-8 with Byte Order Mark (`EF BB BF`).
    Utf8Bom,
    /// UTF-16 Little Endian (`FF FE`).
    Utf16Le,
    /// UTF-16 Big Endian (`FE FF`).
    Utf16Be,
}

/// Inspects byte slice leading bytes, strips BOM marker if present, and decodes content into UTF-8 [`String`].
pub fn detect_encoding_and_strip_bom(bytes: &[u8]) -> Result<(String, Format)> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let content = std::str::from_utf8(&bytes[3..])
            .map_err(|e| XmlError::Io(format!("Invalid UTF-8 after BOM: {e}")))?;
        Ok((content.to_string(), Format::Utf8Bom))
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        let u16_slice: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        let content = String::from_utf16(&u16_slice)
            .map_err(|e| XmlError::Io(format!("Invalid UTF-16 LE data: {e}")))?;
        Ok((content, Format::Utf16Le))
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        let u16_slice: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        let content = String::from_utf16(&u16_slice)
            .map_err(|e| XmlError::Io(format!("Invalid UTF-16 BE data: {e}")))?;
        Ok((content, Format::Utf16Be))
    } else {
        let content = std::str::from_utf8(bytes)
            .map_err(|e| XmlError::Io(format!("Invalid UTF-8 data: {e}")))?;
        Ok((content.to_string(), Format::Utf8))
    }
}
