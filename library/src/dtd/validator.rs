use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentModel {
    Empty,
    Any,
    Mixed(Vec<String>),
    Children(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtdElementRule {
    pub name: String,
    pub model: ContentModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtdAttributeRule {
    pub element_name: String,
    pub attr_name: String,
    pub attr_type: String,
    pub default_decl: String,
}

#[derive(Debug, Clone, Default)]
pub struct DtdValidator {
    elements: HashMap<String, DtdElementRule>,
    attributes: HashMap<String, Vec<DtdAttributeRule>>,
}

impl DtdValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_subset(&mut self, subset: &str) -> Result<()> {
        let lines = subset.lines();
        for raw_line in lines {
            let line = raw_line.trim();
            if line.starts_with("<!ELEMENT") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let elem_name = parts[1].to_string();
                    let model_str = parts[2..].join(" ");
                    let model = if model_str.contains("EMPTY") {
                        ContentModel::Empty
                    } else if model_str.contains("ANY") {
                        ContentModel::Any
                    } else {
                        ContentModel::Children(model_str)
                    };
                    self.elements.insert(
                        elem_name.clone(),
                        DtdElementRule {
                            name: elem_name,
                            model,
                        },
                    );
                }
            } else if line.starts_with("<!ATTLIST") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let elem_name = parts[1].to_string();
                    let attr_name = parts[2].to_string();
                    let attr_type = parts[3].to_string();
                    let default_decl = if parts.len() >= 5 {
                        parts[4].to_string()
                    } else {
                        "#IMPLIED".to_string()
                    };
                    self.attributes.entry(elem_name.clone()).or_default().push(
                        DtdAttributeRule {
                            element_name: elem_name,
                            attr_name,
                            attr_type,
                            default_decl,
                        },
                    );
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self, doc: &Document) -> Result<()> {
        if let Some(dtd_id) = doc.dtd_id() {
            if let Some(node) = doc.get_node(dtd_id) {
                if let NodeKind::DocTypeDefinition {
                    internal_subset: Some(subset),
                    ..
                } = &node.kind
                {
                    let mut mut_self = self.clone();
                    mut_self.parse_subset(subset)?;
                    return mut_self.validate_doc(doc);
                }
            }
        }
        self.validate_doc(doc)
    }

    fn validate_doc(&self, doc: &Document) -> Result<()> {
        if let Some(root_elem_id) = doc.root_element_id() {
            self.validate_element(doc, root_elem_id)?;
        }
        Ok(())
    }

    fn validate_element(&self, doc: &Document, elem_id: NodeId) -> Result<()> {
        let node = doc
            .get_node(elem_id)
            .ok_or_else(|| XmlError::DtdError("Invalid node".into()))?;

        if let NodeKind::Element { name, attributes } = &node.kind {
            // Check Element Rule
            if let Some(rule) = self.elements.get(name) {
                match &rule.model {
                    ContentModel::Empty => {
                        let has_child_elems = node.children.iter().any(|&c_id| {
                            doc.get_node(c_id).map_or(false, |c| {
                                matches!(c.kind, NodeKind::Element { .. } | NodeKind::Text(_))
                            })
                        });
                        if has_child_elems {
                            return Err(XmlError::DtdError(format!(
                                "Element <{name}> is declared EMPTY but contains child content"
                            )));
                        }
                    }
                    ContentModel::Children(spec) => {
                        // Extract element names of direct children
                        let child_names: Vec<String> = node.children.iter().filter_map(|&c_id| {
                            doc.get_node(c_id).and_then(|c| match &c.kind {
                                NodeKind::Element { name, .. } => Some(name.clone()),
                                _ => None,
                            })
                        }).collect();

                        // Basic child specification matching
                        if spec.contains(',') && !spec.contains('*') && !spec.contains('?') {
                            // Required sequence like (to,from,heading,body)
                            let required_names: Vec<String> = spec
                                .trim_matches(|c| c == '(' || c == ')' || c == '>' || c == ' ')
                                .split(',')
                                .map(|s| s.trim().trim_matches('>').trim().to_string())
                                .filter(|s| !s.is_empty() && !s.starts_with('#'))
                                .collect();

                            for req in &required_names {
                                if !child_names.contains(req) {
                                    return Err(XmlError::DtdError(format!(
                                        "Element <{name}> does not conform to content specification {spec}: missing required child <{req}>"
                                    )));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check Attribute Rules
            if let Some(attr_rules) = self.attributes.get(name) {
                for rule in attr_rules {
                    if rule.default_decl.contains("#REQUIRED") {
                        let present = attributes.iter().any(|a| a.name == rule.attr_name);
                        if !present {
                            return Err(XmlError::DtdError(format!(
                                "Required attribute '{}' missing on element <{name}>",
                                rule.attr_name
                            )));
                        }
                    }
                }
            }

            // Recurse on child elements
            for &child_id in &node.children {
                if let Some(child) = doc.get_node(child_id) {
                    if matches!(child.kind, NodeKind::Element { .. }) {
                        self.validate_element(doc, child_id)?;
                    }
                }
            }
        }
        Ok(())
    }
}
