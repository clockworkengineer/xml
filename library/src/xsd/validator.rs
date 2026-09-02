use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct XsdRestriction {
    pub min_inclusive: Option<i64>,
    pub max_inclusive: Option<i64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub enumerations: Vec<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct XsdElementRule {
    pub name: String,
    pub elem_type: String,
    pub min_occurs: usize,
    pub max_occurs: Option<usize>,
    pub restriction: XsdRestriction,
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
            self.collect_elements(&schema_doc, root_elem_id);
        }
        Ok(())
    }

    fn collect_elements(&mut self, schema_doc: &Document, node_id: NodeId) {
        if let Some(node) = schema_doc.get_node(node_id) {
            if let NodeKind::Element { name, attributes } = &node.kind {
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
                        let restriction = self.extract_restriction(schema_doc, node_id);
                        self.elements.insert(
                            elem_name.clone(),
                            XsdElementRule {
                                name: elem_name,
                                elem_type,
                                min_occurs: 1,
                                max_occurs: Some(1),
                                restriction,
                            },
                        );
                    }
                }
            }

            for &c_id in &node.children {
                self.collect_elements(schema_doc, c_id);
            }
        }
    }

    fn extract_restriction(&self, doc: &Document, elem_id: NodeId) -> XsdRestriction {
        let mut rest = XsdRestriction::default();
        let mut stack = vec![elem_id];

        while let Some(nid) = stack.pop() {
            if let Some(node) = doc.get_node(nid) {
                if let NodeKind::Element { name, attributes } = &node.kind {
                    let local_name = name.split(':').last().unwrap_or(name);
                    let get_val = || attributes.iter().find(|a| a.name == "value").map(|a| a.value.as_str());

                    match local_name {
                        "minInclusive" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<i64>().ok()) {
                                rest.min_inclusive = Some(v);
                            }
                        }
                        "maxInclusive" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<i64>().ok()) {
                                rest.max_inclusive = Some(v);
                            }
                        }
                        "minLength" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<usize>().ok()) {
                                rest.min_length = Some(v);
                            }
                        }
                        "maxLength" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<usize>().ok()) {
                                rest.max_length = Some(v);
                            }
                        }
                        "enumeration" => {
                            if let Some(v) = get_val() {
                                rest.enumerations.push(v.to_string());
                            }
                        }
                        "pattern" => {
                            if let Some(v) = get_val() {
                                rest.pattern = Some(v.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                stack.extend(node.children.iter().copied());
            }
        }
        rest
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
                let text = self.get_element_text(doc, elem_id);
                let trimmed = text.trim();

                // Primitive Type Validation
                if rule.elem_type == "xs:integer" || rule.elem_type == "xsd:integer" {
                    if !trimmed.is_empty() {
                        let parsed = trimmed.parse::<i64>().map_err(|_| {
                            XmlError::XsdError(format!("Element <{name}> value '{trimmed}' is not a valid integer"))
                        })?;

                        if let Some(min) = rule.restriction.min_inclusive {
                            if parsed < min {
                                return Err(XmlError::XsdError(format!(
                                    "Element <{name}> value '{parsed}' violates minInclusive ({min})"
                                )));
                            }
                        }
                        if let Some(max) = rule.restriction.max_inclusive {
                            if parsed > max {
                                return Err(XmlError::XsdError(format!(
                                    "Element <{name}> value '{parsed}' violates maxInclusive ({max})"
                                )));
                            }
                        }
                    }
                } else if rule.elem_type == "xs:boolean" || rule.elem_type == "xsd:boolean" {
                    if !trimmed.is_empty() && trimmed != "true" && trimmed != "false" && trimmed != "1" && trimmed != "0" {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> value '{trimmed}' is not a valid boolean"
                        )));
                    }
                }

                // String Restrictions
                if let Some(min_len) = rule.restriction.min_length {
                    if trimmed.chars().count() < min_len {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> length violates minLength ({min_len})"
                        )));
                    }
                }
                if let Some(max_len) = rule.restriction.max_length {
                    if trimmed.chars().count() > max_len {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> length violates maxLength ({max_len})"
                        )));
                    }
                }
                if !rule.restriction.enumerations.is_empty() {
                    if !rule.restriction.enumerations.iter().any(|e| e == trimmed) {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> value '{trimmed}' not in enumeration list"
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
