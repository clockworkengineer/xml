//! # Entity Mapper & XXE Security Guard
//!
//! Provides predefined entity mapping (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`), numeric reference decoding (`&#65;`, `&#x41;`), and recursive expansion limits.

use std::collections::HashMap;
use crate::error::{Result, XmlError};

/// Entity table mapping entity reference names to replacement text with depth recursion tracking.
#[derive(Debug, Clone)]
pub struct EntityMapper {
    entities: HashMap<String, String>,
    max_depth: usize,
}

impl Default for EntityMapper {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert("lt".into(), "<".into());
        map.insert("gt".into(), ">".into());
        map.insert("amp".into(), "&".into());
        map.insert("quot".into(), "\"".into());
        map.insert("apos".into(), "'".into());
        Self {
            entities: map,
            max_depth: 512,
        }
    }
}

impl EntityMapper {
    /// Instantiates a new [`EntityMapper`] with a given max recursion depth limit.
    pub fn new(max_depth: usize) -> Self {
        let mut s = Self::default();
        s.max_depth = max_depth;
        s
    }

    /// Registers a custom entity reference name and replacement value.
    pub fn register(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.entities.insert(name.into(), value.into());
    }

    /// Looks up an entity reference replacement value by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.entities.get(name)
    }

    /// Expands entity references and numeric character references within input string slice.
    ///
    /// # Errors
    /// Returns [`XmlError::SecurityLimitExceeded`] if expansion depth exceeds `max_depth` (XML bomb guard).
    pub fn expand(&self, input: &str) -> Result<String> {
        if !input.contains('&') {
            return Ok(input.to_string());
        }
        self.expand_with_depth(input, 0)
    }

    fn expand_with_depth(&self, input: &str, depth: usize) -> Result<String> {
        if depth > self.max_depth {
            return Err(XmlError::SecurityLimitExceeded(
                "Maximum entity expansion depth exceeded (possible XML Bomb/Billion Laughs)".into(),
            ));
        }

        let mut result = String::with_capacity(input.len());
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            if bytes[pos] == b'&' {
                if let Some(semi_offset) = input[pos..].find(';') {
                    let semi_idx = pos + semi_offset;
                    let entity_ref = &input[pos + 1..semi_idx];

                    if entity_ref.starts_with('#') {
                        // Numeric reference (dec or hex)
                        let code_str = &entity_ref[1..];
                        let codepoint = if code_str.starts_with('x') || code_str.starts_with('X') {
                            u32::from_str_radix(&code_str[1..], 16)
                        } else {
                            code_str.parse::<u32>()
                        };

                        match codepoint {
                            Ok(cp) => {
                                if let Some(ch) = char::from_u32(cp) {
                                    result.push(ch);
                                } else {
                                    return Err(XmlError::EntityError(format!(
                                        "Invalid character reference code point: {code_str}"
                                    )));
                                }
                            }
                            Err(_) => {
                                return Err(XmlError::EntityError(format!(
                                    "Malformed numeric entity reference: &{entity_ref};"
                                )));
                            }
                        }
                    } else {
                        // Named reference
                        match entity_ref {
                            "lt" => result.push('<'),
                            "gt" => result.push('>'),
                            "amp" => result.push('&'),
                            "quot" => result.push('"'),
                            "apos" => result.push('\''),
                            _ => {
                                if let Some(val) = self.entities.get(entity_ref) {
                                    let expanded_val = self.expand_with_depth(val, depth + 1)?;
                                    result.push_str(&expanded_val);
                                } else {
                                    return Err(XmlError::EntityError(format!(
                                        "Undeclared entity reference: &{entity_ref};"
                                    )));
                                }
                            }
                        }
                    }

                    pos = semi_idx + 1;
                } else {
                    return Err(XmlError::EntityError(
                        "Unterminated entity reference missing ';'".into(),
                    ));
                }
            } else {
                let ch = input[pos..].chars().next().unwrap();
                result.push(ch);
                pos += ch.len_utf8();
            }
        }

        Ok(result)
    }
}
