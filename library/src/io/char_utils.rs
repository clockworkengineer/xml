//! # Character Classification Utilities
//!
//! Centralized helpers for validating XML character categories according to W3C XML 1.0 recommendations.

/// Returns `true` if character is valid as the first character of an XML Name.
#[inline]
pub fn is_xml_name_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == ':'
}

/// Returns `true` if character is valid within an XML Name tag.
#[inline]
pub fn is_xml_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ':' || ch == '.'
}

/// Returns `true` if character is standard XML whitespace (`' '`, `'\t'`, `'\r'`, `'\n'`).
#[inline]
pub fn is_xml_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n')
}
