//! # Parse Options & Security Policy
//!
//! Provides configurable parsing options and security thresholds to mitigate DoS and XXE vulnerabilities.

/// Security policy options and resource limits for the XML parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    /// Maximum XML document size in bytes (default: 100 MB).
    pub max_xml_size: usize,
    /// Maximum depth for recursive entity reference expansion (default: 512).
    pub max_entity_expansion_depth: usize,
    /// Maximum element nesting depth (default: 1000).
    pub max_nesting_depth: usize,
    /// Maximum total element count in document (default: 1,000,000).
    pub max_element_count: usize,
    /// Maximum attribute count per element (default: 10,000).
    pub max_attribute_count: usize,
    /// Maximum total attribute count across document (default: 1,000,000).
    pub max_total_attribute_count: usize,
    /// Maximum text node character size (default: 1 MB).
    pub max_text_node_size: usize,
    /// Allow fetching external entity URIs (default: false for XXE protection).
    pub allow_external_entities: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_xml_size: 100 * 1024 * 1024,
            max_entity_expansion_depth: 512,
            max_nesting_depth: 1000,
            max_element_count: 1_000_000,
            max_attribute_count: 10_000,
            max_total_attribute_count: 1_000_000,
            max_text_node_size: 1024 * 1024,
            allow_external_entities: false,
        }
    }
}
