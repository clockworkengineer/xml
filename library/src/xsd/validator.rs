use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct XsdElementRule {
    pub name: String,
    pub elem_type: String,
    pub min_occurs: usize,
    pub max_occurs: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct XsdValidator {
    elements: HashMap<String, XsdElementRule>,
}

impl XsdValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_schema(&mut self, schema_xml: &str) -> Result<()> {
        let source = crate::io::source::XmlSource::from_string(schema_xml);
        let options = crate::options::ParseOptions::default();
        let mut parser = crate::parser::XmlParser::new(source, options);
        let schema_doc = parser.parse()?;

        if let Some(root_elem_id) = schema_doc.root_element_id() {
            if let Some(node) = schema_doc.get_node(root_elem_id) {
                for &child_id in &node.children {
                    if let Some(child) = schema_doc.get_node(child_id) {
                        if let NodeKind::Element { name, attributes } = &child.kind {
                            if name.ends_with("element") || name == "xs:element" || name == "xsd:element" {
                                let elem_name = attributes
                                    .iter()
                                    .find(|a| a.name == "name")
                                    .map(|a| a.value.clone())
                                    .unwrap_or_default();
                                let elem_type = attributes
                                    .iter()
                                    .find(|a| a.name == "type")
                                    .map(|a| a.value.clone())
                                    .unwrap_or_else(|| "xs:string".into());
                                
                                if !elem_name.is_empty() {
                                    self.elements.insert(
                                        elem_name.clone(),
                                        XsdElementRule {
                                            name: elem_name,
                                            elem_type,
                                            min_occurs: 1,
                                            max_occurs: Some(1),
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self, doc: &Document) -> Result<()> {
        if let Some(root_id) = doc.root_element_id() {
            self.validate_element(doc, root_id)?;
        }
        Ok(())
    }

    fn validate_element(&self, doc: &Document, elem_id: NodeId) -> Result<()> {
        let node = doc
            .get_node(elem_id)
            .ok_or_else(|| XmlError::XsdError("Invalid node".into()))?;

        if let NodeKind::Element { name, .. } = &node.kind {
            if let Some(rule) = self.elements.get(name) {
                // Type checks for primitive types
                if rule.elem_type == "xs:integer" || rule.elem_type == "xsd:integer" {
                    let text = self.get_element_text(doc, elem_id);
                    if !text.trim().is_empty() && text.trim().parse::<i64>().is_err() {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> value '{text}' is not a valid integer"
                        )));
                    }
                } else if rule.elem_type == "xs:boolean" || rule.elem_type == "xsd:boolean" {
                    let text = self.get_element_text(doc, elem_id);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() && trimmed != "true" && trimmed != "false" && trimmed != "1" && trimmed != "0" {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> value '{text}' is not a valid boolean"
                        )));
                    }
                }
            }

            for &c_id in &node.children {
                if let Some(c) = doc.get_node(c_id) {
                    if matches!(c.kind, NodeKind::Element { .. }) {
                        self.validate_element(doc, c_id)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn get_element_text(&self, doc: &Document, elem_id: NodeId) -> String {
        let mut text = String::new();
        if let Some(node) = doc.get_node(elem_id) {
            for &c_id in &node.children {
                if let Some(c) = doc.get_node(c_id) {
                    if let NodeKind::Text(t) = &c.kind {
                        text.push_str(t);
                    }
                }
            }
        }
        text
    }
}
