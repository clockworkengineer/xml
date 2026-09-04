//! # Character Encoding & BOM Auto-Detection
//!
//! Auto-detects UTF-8 / UTF-16 Byte Order Marks (BOM) and decodes raw byte slices into UTF-8 string buffers.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};

/// Enum representing supported byte encoding formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// UTF-8 encoding (default).
    Utf8,
    /// UTF-8 with Byte Order Mark (`EF BB BF`).
    Utf8Bom,
    /// UTF-16 Little Endian encoding.
    Utf16Le,
    /// UTF-16 Big Endian encoding.
    Utf16Be,
    /// ISO-8859-1 (Latin-1) single-byte encoding.
    Iso8859_1,
    /// Windows-1252 single-byte encoding.
    Windows1252,
    /// 7-bit US-ASCII encoding.
    Ascii,
}

/// Helper mapping Windows-1252 byte to Unicode character.
fn decode_windows_1252_byte(b: u8) -> char {
    match b {
        0x80 => '€',
        0x82 => '‚',
        0x83 => 'ƒ',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x88 => 'ˆ',
        0x89 => '‰',
        0x8A => 'Š',
        0x8B => '‹',
        0x8C => 'Œ',
        0x8E => 'Ž',
        0x91 => '‘',
        0x92 => '’',
        0x93 => '“',
        0x94 => '”',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x98 => '˜',
        0x99 => '™',
        0x9A => 'š',
        0x9B => '›',
        0x9C => 'œ',
        0x9E => 'ž',
        0x9F => 'Ÿ',
        other => other as char,
    }
}

/// Decodes raw byte slice into a UTF-8 string using an explicitly specified encoding name.
///
/// Supported encoding identifiers: `"UTF-8"`, `"ISO-8859-1"`, `"LATIN1"`, `"WINDOWS-1252"`, `"CP1252"`, `"US-ASCII"`, `"ASCII"`.
pub fn decode_with_encoding(bytes: &[u8], encoding: &str) -> Result<(String, Format)> {
    let enc_trimmed = encoding.trim().to_uppercase();
    match enc_trimmed.as_str() {
        "UTF-8" | "UTF8" => detect_encoding_and_strip_bom(bytes),
        "ISO-8859-1" | "ISO_8859_1" | "LATIN1" | "LATIN-1" => {
            let s: String = bytes.iter().map(|&b| b as char).collect();
            Ok((s, Format::Iso8859_1))
        }
        "WINDOWS-1252" | "WINDOWS_1252" | "CP1252" => {
            let s: String = bytes.iter().map(|&b| decode_windows_1252_byte(b)).collect();
            Ok((s, Format::Windows1252))
        }
        "US-ASCII" | "ASCII" => {
            for &b in bytes {
                if b > 127 {
                    return Err(XmlError::Io("Byte out of 7-bit US-ASCII range".into()));
                }
            }
            let s: String = bytes.iter().map(|&b| b as char).collect();
            Ok((s, Format::Ascii))
        }
        _ => Err(XmlError::Io(format!("Unsupported character encoding: {encoding}"))),
    }
}

/// Detects Byte Order Mark (BOM) preamble from raw bytes and returns normalized UTF-8 string.
pub fn detect_encoding_and_strip_bom(bytes: &[u8]) -> Result<(String, Format)> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let content = core::str::from_utf8(&bytes[3..])
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
        let content = core::str::from_utf8(bytes)
            .map_err(|e| XmlError::Io(format!("Invalid UTF-8 data: {e}")))?;
        Ok((content.to_string(), Format::Utf8))
    }
}
