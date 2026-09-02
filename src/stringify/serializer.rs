use crate::document::Document;
use crate::io::destination::XmlDestination;
use crate::io::encoding::Format;
use crate::node::{NodeId, NodeKind};

#[derive(Debug, Clone)]
pub struct SerializeOptions {
    pub indent_spaces: Option<usize>,
    pub format: Format,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            indent_spaces: Some(2),
            format: Format::Utf8,
        }
    }
}

pub struct XmlSerializer;

impl XmlSerializer {
    pub fn serialize(doc: &Document, options: &SerializeOptions) -> Vec<u8> {
        let mut dest = XmlDestination::new(options.format);
        if let Some(prolog_id) = doc.prolog_id() {
            Self::serialize_children(doc, prolog_id, &mut dest, options, 0);
        }
        if let Some(root_id) = doc.root_id() {
            Self::serialize_children(doc, root_id, &mut dest, options, 0);
        }
        dest.into_bytes()
    }

    pub fn serialize_to_string(doc: &Document, options: &SerializeOptions) -> String {
        let mut dest = XmlDestination::new(Format::Utf8);
        if let Some(prolog_id) = doc.prolog_id() {
            Self::serialize_children(doc, prolog_id, &mut dest, options, 0);
        }
        if let Some(root_id) = doc.root_id() {
            Self::serialize_children(doc, root_id, &mut dest, options, 0);
        }
        dest.buffer
    }

    fn serialize_children(
        doc: &Document,
        parent_id: NodeId,
        dest: &mut XmlDestination,
        options: &SerializeOptions,
        depth: usize,
    ) {
        if let Some(node) = doc.get_node(parent_id) {
            for &child_id in &node.children {
                Self::serialize_node(doc, child_id, dest, options, depth);
            }
        }
    }

    fn serialize_node(
        doc: &Document,
        node_id: NodeId,
        dest: &mut XmlDestination,
        options: &SerializeOptions,
        depth: usize,
    ) {
        let node = match doc.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        let indent = options
            .indent_spaces
            .map(|sp| " ".repeat(sp * depth))
            .unwrap_or_default();

        match &node.kind {
            NodeKind::Prolog | NodeKind::Root => {
                Self::serialize_children(doc, node_id, dest, options, depth);
            }
            NodeKind::Declaration {
                version,
                encoding,
                standalone,
            } => {
                dest.write_str(&format!("<?xml version=\"{version}\""));
                if let Some(enc) = encoding {
                    dest.write_str(&format!(" encoding=\"{enc}\""));
                }
                if let Some(sa) = standalone {
                    dest.write_str(&format!(" standalone=\"{}\"", if *sa { "yes" } else { "no" }));
                }
                dest.write_str("?>");
                if options.indent_spaces.is_some() {
                    dest.write_char('\n');
                }
            }
            NodeKind::ProcessingInstruction { target, data } => {
                if options.indent_spaces.is_some() && depth > 0 {
                    dest.write_str(&indent);
                }
                dest.write_str(&format!("<?{target} {data}?>"));
                if options.indent_spaces.is_some() {
                    dest.write_char('\n');
                }
            }
            NodeKind::Comment(content) => {
                if options.indent_spaces.is_some() && depth > 0 {
                    dest.write_str(&indent);
                }
                dest.write_str(&format!("<!--{content}-->"));
                if options.indent_spaces.is_some() {
                    dest.write_char('\n');
                }
            }
            NodeKind::DocTypeDefinition {
                name,
                public_id,
                system_id,
                internal_subset,
            } => {
                if options.indent_spaces.is_some() && depth > 0 {
                    dest.write_str(&indent);
                }
                dest.write_str(&format!("<!DOCTYPE {name}"));
                if let Some(pub_id) = public_id {
                    dest.write_str(&format!(" PUBLIC \"{pub_id}\""));
                    if let Some(sys_id) = system_id {
                        dest.write_str(&format!(" \"{sys_id}\""));
                    }
                } else if let Some(sys_id) = system_id {
                    dest.write_str(&format!(" SYSTEM \"{sys_id}\""));
                }
                if let Some(subset) = internal_subset {
                    dest.write_str(&format!(" [{subset}]"));
                }
                dest.write_str(">");
                if options.indent_spaces.is_some() {
                    dest.write_char('\n');
                }
            }
            NodeKind::Element { name, attributes } => {
                if options.indent_spaces.is_some() && depth > 0 {
                    dest.write_str(&indent);
                }
                dest.write_str(&format!("<{name}"));
                for attr in attributes {
                    let escaped_val = Self::escape_attribute(&attr.value);
                    dest.write_str(&format!(" {}=\"{escaped_val}\"", attr.name));
                }

                if node.children.is_empty() {
                    dest.write_str("/>");
                } else {
                    dest.write_char('>');
                    let has_complex_children = node.children.iter().any(|&c_id| {
                        doc.get_node(c_id).map_or(false, |c| {
                            matches!(c.kind, NodeKind::Element { .. })
                        })
                    });

                    if has_complex_children && options.indent_spaces.is_some() {
                        dest.write_char('\n');
                    }

                    Self::serialize_children(
                        doc,
                        node_id,
                        dest,
                        options,
                        if has_complex_children { depth + 1 } else { depth },
                    );

                    if has_complex_children && options.indent_spaces.is_some() {
                        dest.write_str(&indent);
                    }
                    dest.write_str(&format!("</{name}>"));
                }

                if options.indent_spaces.is_some() && depth == 0 {
                    dest.write_char('\n');
                } else if options.indent_spaces.is_some() && depth > 0 {
                    dest.write_char('\n');
                }
            }
            NodeKind::Text(text) => {
                dest.write_str(&Self::escape_text(text));
            }
            NodeKind::CData(cdata) => {
                dest.write_str(&format!("<![CDATA[{cdata}]]>"));
            }
            NodeKind::EntityReference(name) => {
                dest.write_str(&format!("&{name};"));
            }
        }
    }

    fn escape_text(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    fn escape_attribute(val: &str) -> String {
        val.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}
