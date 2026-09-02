//! # XML Serializer / Stringifier
//!
//! Formats and serializes a [`Document`] DOM tree back into valid XML string output with optional pretty printing.

use crate::document::Document;
use crate::io::destination::XmlDestination;
use crate::node::{NodeId, NodeKind};

/// Formatting options for XML serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializeOptions {
    /// Enable pretty-printing indentation (default: true).
    pub pretty_print: bool,
    /// Indentation step size in spaces (default: 2).
    pub indent_step: usize,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            pretty_print: true,
            indent_step: 2,
        }
    }
}

/// Serializer traversing the DOM tree and emitting formatted XML output.
pub struct XmlSerializer;

impl XmlSerializer {
    /// Serializes a [`Document`] DOM tree into a formatted string.
    pub fn serialize_to_string(doc: &Document, options: &SerializeOptions) -> String {
        let mut dest = XmlDestination::new();

        if let Some(decl_id) = doc.declaration_id() {
            if let Some(node) = doc.get_node(decl_id) {
                if let NodeKind::Declaration {
                    version,
                    encoding,
                    standalone,
                } = &node.kind
                {
                    dest.write_str("<?xml version=\"");
                    dest.write_str(version);
                    dest.write_str("\"");
                    if let Some(enc) = encoding {
                        dest.write_str(" encoding=\"");
                        dest.write_str(enc);
                        dest.write_str("\"");
                    }
                    if let Some(st) = standalone {
                        dest.write_str(" standalone=\"");
                        dest.write_str(if *st { "yes" } else { "no" });
                        dest.write_str("\"");
                    }
                    dest.write_str("?>\n");
                }
            }
        }

        if let Some(prolog_id) = doc.prolog_id() {
            Self::serialize_children(doc, prolog_id, 0, options, &mut dest);
        }

        if let Some(root_id) = doc.root_id() {
            Self::serialize_children(doc, root_id, 0, options, &mut dest);
        }

        dest.into_string()
    }

    fn serialize_children(
        doc: &Document,
        parent_id: NodeId,
        indent_level: usize,
        options: &SerializeOptions,
        dest: &mut XmlDestination,
    ) {
        if let Some(node) = doc.get_node(parent_id) {
            for &c_id in &node.children {
                if doc.declaration_id() == Some(c_id) {
                    continue; // Already processed XML Declaration
                }
                Self::serialize_node(doc, c_id, indent_level, options, dest);
            }
        }
    }

    fn serialize_node(
        doc: &Document,
        node_id: NodeId,
        indent_level: usize,
        options: &SerializeOptions,
        dest: &mut XmlDestination,
    ) {
        let node = match doc.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        let indent = if options.pretty_print {
            " ".repeat(indent_level * options.indent_step)
        } else {
            String::new()
        };

        match &node.kind {
            NodeKind::Element { name, attributes } => {
                dest.write_str(&indent);
                dest.write_str("<");
                dest.write_str(name);

                for attr in attributes {
                    dest.write_str(" ");
                    dest.write_str(&attr.name);
                    dest.write_str("=\"");
                    Self::write_escaped_attr(&attr.value, dest);
                    dest.write_str("\"");
                }

                if node.children.is_empty() {
                    dest.write_str("/>");
                    if options.pretty_print {
                        dest.write_char('\n');
                    }
                } else {
                    dest.write_str(">");
                    let has_only_text = node.children.iter().all(|&c_id| {
                        doc.get_node(c_id).map_or(false, |c| {
                            matches!(c.kind, NodeKind::Text(_) | NodeKind::CData(_))
                        })
                    });

                    if has_only_text {
                        for &c_id in &node.children {
                            if let Some(child) = doc.get_node(c_id) {
                                match &child.kind {
                                    NodeKind::Text(t) => Self::write_escaped_text(t, dest),
                                    NodeKind::CData(c) => {
                                        dest.write_str("<![CDATA[");
                                        dest.write_str(c);
                                        dest.write_str("]]>");
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        if options.pretty_print {
                            dest.write_char('\n');
                        }
                        Self::serialize_children(doc, node_id, indent_level + 1, options, dest);
                        dest.write_str(&indent);
                    }

                    dest.write_str("</");
                    dest.write_str(name);
                    dest.write_str(">");
                    if options.pretty_print {
                        dest.write_char('\n');
                    }
                }
            }
            NodeKind::Text(t) => {
                if !options.pretty_print || !t.trim().is_empty() {
                    Self::write_escaped_text(t, dest);
                }
            }
            NodeKind::CData(c) => {
                dest.write_str("<![CDATA[");
                dest.write_str(c);
                dest.write_str("]]>");
            }
            NodeKind::Comment(c) => {
                dest.write_str(&indent);
                dest.write_str("<!--");
                dest.write_str(c);
                dest.write_str("-->");
                if options.pretty_print {
                    dest.write_char('\n');
                }
            }
            NodeKind::ProcessingInstruction { target, data } => {
                dest.write_str(&indent);
                dest.write_str("<?");
                dest.write_str(target);
                if !data.is_empty() {
                    dest.write_str(" ");
                    dest.write_str(data);
                }
                dest.write_str("?>");
                if options.pretty_print {
                    dest.write_char('\n');
                }
            }
            _ => {}
        }
    }

    fn write_escaped_text(s: &str, dest: &mut XmlDestination) {
        let mut last = 0;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let esc = match b {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                _ => continue,
            };
            dest.write_str(&s[last..i]);
            dest.write_str(esc);
            last = i + 1;
        }
        dest.write_str(&s[last..]);
    }

    fn write_escaped_attr(s: &str, dest: &mut XmlDestination) {
        let mut last = 0;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let esc = match b {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'"' => "&quot;",
                _ => continue,
            };
            dest.write_str(&s[last..i]);
            dest.write_str(esc);
            last = i + 1;
        }
        dest.write_str(&s[last..]);
    }
}
