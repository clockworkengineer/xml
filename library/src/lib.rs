pub mod document;
pub mod dtd;
pub mod entity;
pub mod error;
pub mod io;
pub mod node;
pub mod options;
pub mod parser;
pub mod stringify;
pub mod xsd;
pub mod xpath;

pub use document::Document;
pub use dtd::DtdValidator;
pub use entity::{EntityMapper, EntityResolver};
pub use error::{Result, XmlError};
pub use io::{Format, XmlDestination, XmlSource};
pub use node::{Attribute, NodeData, NodeId, NodeKind};
pub use options::ParseOptions;
pub use parser::XmlParser;
pub use stringify::{SerializeOptions, XmlSerializer};
pub use xsd::XsdValidator;
pub use xpath::{XPathEngine, XPathValue};

/// Parse an XML string into a `Document` with default options
pub fn parse(xml: &str) -> Result<Document> {
    parse_with_options(xml, ParseOptions::default())
}

/// Parse an XML string into a `Document` with custom options
pub fn parse_with_options(xml: &str, options: ParseOptions) -> Result<Document> {
    let source = XmlSource::from_string(xml);
    let mut parser = XmlParser::new(source, options);
    parser.parse()
}

/// Parse XML bytes into a `Document` with BOM detection and default options
pub fn parse_bytes(bytes: &[u8]) -> Result<Document> {
    let source = XmlSource::from_bytes(bytes)?;
    let options = ParseOptions::default();
    let mut parser = XmlParser::new(source, options);
    parser.parse()
}

/// Serialize a `Document` into a formatted UTF-8 XML string
pub fn stringify(doc: &Document) -> String {
    XmlSerializer::serialize_to_string(doc, &SerializeOptions::default())
}
