//! # XML Lib (Rust Port)
//!
//! A full-featured, high-performance, pure Rust XML library offering:
//! - XML Parsing & DOM Construction (Arena-based `Document` model)
//! - XML Serialization & Formatting (`XmlSerializer`)
//! - Entity Mapping & XXE Protection (`EntityMapper`)
//! - DTD Content Model & Constraint Validation (`DtdValidator`)
//! - XSD Schema & Restriction Facet Validation (`XsdValidator`)
//! - XPath 1.0 Lexing, Parsing, and Evaluation Engine (`XPathEngine`)

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub mod alloc_prelude;
pub mod document;
pub mod dtd;
pub mod entity;
pub mod error;
pub mod io;
pub mod node;
pub mod options;
pub mod parser;
pub mod stringify;
pub mod validator;
pub mod xsd;
pub mod xpath;

use crate::alloc_prelude::*;

pub use document::Document;
pub use dtd::DtdValidator;
pub use entity::{EntityMapper, EntityResolver};
pub use error::{Result, XmlError};
pub use io::{Format, XmlDestination, XmlSource};
pub use node::{Attribute, NodeData, NodeId, NodeKind};
pub use options::ParseOptions;
pub use parser::{XmlParser, XmlPullAttribute, XmlPullEvent, XmlPullParser};
pub use stringify::{SerializeOptions, XmlSerializer};
pub use validator::XmlValidator;
pub use xsd::XsdValidator;
pub use xpath::{XPathEngine, XPathValue};

/// Parse an XML string slice into a DOM `Document` using default parsing options.
///
/// # Errors
/// Returns [`XmlError::SyntaxError`] if the input XML is malformed or violates syntax rules.
pub fn parse(xml: &str) -> Result<Document> {
    parse_with_options(xml, ParseOptions::default())
}

/// Parse an XML string slice into a DOM `Document` with custom security limits and parse options.
///
/// # Arguments
/// * `xml` - Input XML string slice.
/// * `options` - Custom [`ParseOptions`] configuring depth limits and attribute thresholds.
pub fn parse_with_options(xml: &str, options: ParseOptions) -> Result<Document> {
    let source = XmlSource::from_string(xml);
    let mut parser = XmlParser::new(source, options);
    parser.parse()
}

/// Parse raw byte slice into a DOM `Document` with automatic UTF-8 / UTF-16 BOM detection.
///
/// # Arguments
/// * `bytes` - Raw byte slice (UTF-8, UTF-16 LE, or UTF-16 BE with optional BOM).
pub fn parse_bytes(bytes: &[u8]) -> Result<Document> {
    let source = XmlSource::from_bytes(bytes)?;
    let options = ParseOptions::default();
    let mut parser = XmlParser::new(source, options);
    parser.parse()
}

/// Serialize a DOM `Document` into a formatted UTF-8 XML string representation.
///
/// # Arguments
/// * `doc` - Reference to target [`Document`].
pub fn stringify(doc: &Document) -> String {
    XmlSerializer::serialize_to_string(doc, &SerializeOptions::default())
}

/// Parse an XML file from disk into a DOM `Document` with automatic UTF-8 / UTF-16 BOM detection.
///
/// # Arguments
/// * `path` - Path to target XML file on disk (`impl AsRef<std::path::Path>`).
///
/// # Errors
/// Returns [`XmlError::Io`] if reading the file fails, or [`XmlError::SyntaxError`] if XML is malformed.
#[cfg(feature = "std")]
pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Document> {
    parse_file_with_options(path, ParseOptions::default())
}

/// Parse an XML file from disk into a DOM `Document` with custom security options.
///
/// # Arguments
/// * `path` - Path to target XML file on disk (`impl AsRef<std::path::Path>`).
/// * `options` - Custom [`ParseOptions`] configuring depth limits and attribute thresholds.
#[cfg(feature = "std")]
pub fn parse_file_with_options(
    path: impl AsRef<std::path::Path>,
    options: ParseOptions,
) -> Result<Document> {
    let source = XmlSource::from_file(path)?;
    let mut parser = XmlParser::new(source, options);
    parser.parse()
}
