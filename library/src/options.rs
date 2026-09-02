#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    pub max_xml_size: usize,
    pub max_entity_expansion_depth: usize,
    pub max_nesting_depth: usize,
    pub max_element_count: usize,
    pub max_attribute_count: usize,
    pub max_total_attribute_count: usize,
    pub max_text_node_size: usize,
    pub allow_external_entities: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_xml_size: 100 * 1024 * 1024, // 100 MB
            max_entity_expansion_depth: 512,
            max_nesting_depth: 1000,
            max_element_count: 1_000_000,
            max_attribute_count: 10_000,
            max_total_attribute_count: 1_000_000,
            max_text_node_size: 1024 * 1024, // 1 MB
            allow_external_entities: false,
        }
    }
}
