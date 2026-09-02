use crate::document::Document;
use crate::entity::EntityMapper;
use crate::error::{Result, XmlError};
use crate::io::source::XmlSource;
use crate::node::{Attribute, NodeId, NodeKind};
use crate::options::ParseOptions;

#[derive(Debug)]
pub struct XmlParser<'a> {
    source: XmlSource,
    options: ParseOptions,
    entity_mapper: EntityMapper,
    element_count: usize,
    total_attribute_count: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> XmlParser<'a> {
    pub fn new(source: XmlSource, options: ParseOptions) -> Self {
        let mapper = EntityMapper::new(options.max_entity_expansion_depth);
        Self {
            source,
            options,
            entity_mapper: mapper,
            element_count: 0,
            total_attribute_count: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn parse(&mut self) -> Result<Document> {
        let mut doc = Document::new();
        let root_id = doc.root_id().unwrap();
        let prolog_id = doc.prolog_id().unwrap();

        // 1. Parse Prolog items before main element
        while !self.source.is_eof() {
            self.source.skip_whitespace();
            if self.source.is_eof() {
                break;
            }

            if self.source.starts_with("<?xml") {
                let decl_id = self.parse_declaration(&mut doc)?;
                doc.set_declaration_id(decl_id);
                doc.append_child(prolog_id, decl_id)?;
            } else if self.source.starts_with("<?") {
                let pi_id = self.parse_pi(&mut doc)?;
                doc.append_child(prolog_id, pi_id)?;
            } else if self.source.starts_with("<!--") {
                let comment_id = self.parse_comment(&mut doc)?;
                doc.append_child(prolog_id, comment_id)?;
            } else if self.source.starts_with("<!DOCTYPE") {
                let dtd_id = self.parse_doctype(&mut doc)?;
                doc.set_dtd_id(dtd_id);
                doc.append_child(prolog_id, dtd_id)?;
            } else if self.source.starts_with("<") {
                // Root element start
                break;
            } else {
                return Err(self.source.syntax_error("Unexpected character in prolog"));
            }
        }

        // 2. Parse Root Element
        self.source.skip_whitespace();
        if self.source.is_eof() {
            return Err(self.source.syntax_error("Empty XML document (missing root element)"));
        }

        let root_elem_id = self.parse_element(&mut doc, 0)?;
        doc.append_child(root_id, root_elem_id)?;

        // 3. Parse Trailing Comments / PIs after root element
        while !self.source.is_eof() {
            self.source.skip_whitespace();
            if self.source.is_eof() {
                break;
            }

            if self.source.starts_with("<?") {
                let pi_id = self.parse_pi(&mut doc)?;
                doc.append_child(root_id, pi_id)?;
            } else if self.source.starts_with("<!--") {
                let comment_id = self.parse_comment(&mut doc)?;
                doc.append_child(root_id, comment_id)?;
            } else {
                return Err(self.source.syntax_error("Unexpected content after root element"));
            }
        }

        Ok(doc)
    }

    fn parse_declaration(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume_prefix("<?xml");
        self.source.skip_whitespace();

        let mut version = "1.0".to_string();
        let mut encoding = None;
        let mut standalone = None;

        while !self.source.starts_with("?>") && !self.source.is_eof() {
            self.source.skip_whitespace();
            if self.source.starts_with("?>") {
                break;
            }
            let name = self.parse_name()?;
            self.source.skip_whitespace();
            if !self.source.consume_prefix("=") {
                return Err(self.source.syntax_error("Expected '=' after attribute name in XML declaration"));
            }
            self.source.skip_whitespace();
            let value = self.parse_quoted_string()?;

            match name.as_str() {
                "version" => version = value,
                "encoding" => encoding = Some(value),
                "standalone" => {
                    standalone = Some(value == "yes");
                }
                _ => {}
            }
            self.source.skip_whitespace();
        }

        if !self.source.consume_prefix("?>") {
            return Err(self.source.syntax_error("Unclosed XML declaration"));
        }

        let node_id = doc.add_node(NodeKind::Declaration {
            version,
            encoding,
            standalone,
        });
        Ok(node_id)
    }

    fn parse_pi(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume_prefix("<?");
        let target = self.parse_name()?;
        
        let mut data = String::new();
        while !self.source.starts_with("?>") && !self.source.is_eof() {
            if let Some(ch) = self.source.next_char() {
                data.push(ch);
            } else {
                break;
            }
        }

        if !self.source.consume_prefix("?>") {
            return Err(self.source.syntax_error("Unclosed processing instruction"));
        }

        let node_id = doc.add_node(NodeKind::ProcessingInstruction {
            target,
            data: data.trim().to_string(),
        });
        Ok(node_id)
    }

    fn parse_comment(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume_prefix("<!--");
        let mut content = String::new();

        while !self.source.starts_with("-->") && !self.source.is_eof() {
            if let Some(ch) = self.source.next_char() {
                content.push(ch);
            } else {
                break;
            }
        }

        if !self.source.consume_prefix("-->") {
            return Err(self.source.syntax_error("Unclosed comment"));
        }

        let node_id = doc.add_node(NodeKind::Comment(content));
        Ok(node_id)
    }

    fn parse_doctype(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume_prefix("<!DOCTYPE");
        self.source.skip_whitespace();

        let name = self.parse_name()?;
        self.source.skip_whitespace();

        let mut public_id = None;
        let mut system_id = None;
        let mut internal_subset = None;

        if self.source.starts_with("PUBLIC") {
            self.source.consume_prefix("PUBLIC");
            self.source.skip_whitespace();
            public_id = Some(self.parse_quoted_string()?);
            self.source.skip_whitespace();
            system_id = Some(self.parse_quoted_string()?);
            self.source.skip_whitespace();
        } else if self.source.starts_with("SYSTEM") {
            self.source.consume_prefix("SYSTEM");
            self.source.skip_whitespace();
            system_id = Some(self.parse_quoted_string()?);
            self.source.skip_whitespace();
        }

        if self.source.consume_prefix("[") {
            let mut subset = String::new();
            let mut depth = 1;
            while depth > 0 && !self.source.is_eof() {
                if self.source.starts_with("[") {
                    depth += 1;
                    subset.push(self.source.next_char().unwrap());
                } else if self.source.starts_with("]") {
                    depth -= 1;
                    if depth == 0 {
                        self.source.next_char();
                        break;
                    } else {
                        subset.push(self.source.next_char().unwrap());
                    }
                } else if let Some(ch) = self.source.next_char() {
                    subset.push(ch);
                }
            }
            internal_subset = Some(subset);
            if let Some(ref sub_str) = internal_subset {
                for line in sub_str.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("<!ENTITY") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let ename = parts[1];
                            let eval = parts[2..].join(" ");
                            let cleaned_val = eval.trim_matches('"').trim_matches('\'');
                            self.entity_mapper.register(ename, cleaned_val);
                        }
                    }
                }
            }
            self.source.skip_whitespace();
        }

        if !self.source.consume_prefix(">") {
            return Err(self.source.syntax_error("Unclosed DOCTYPE declaration"));
        }

        let node_id = doc.add_node(NodeKind::DocTypeDefinition {
            name,
            public_id,
            system_id,
            internal_subset,
        });
        Ok(node_id)
    }

    fn parse_element(&mut self, doc: &mut Document, depth: usize) -> Result<NodeId> {
        if depth > self.options.max_nesting_depth {
            return Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum element nesting depth ({}) exceeded",
                self.options.max_nesting_depth
            )));
        }

        self.element_count += 1;
        if self.element_count > self.options.max_element_count {
            return Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum element count ({}) exceeded",
                self.options.max_element_count
            )));
        }

        if !self.source.consume_prefix("<") {
            return Err(self.source.syntax_error("Expected '<' at start of element"));
        }

        let tag_name = self.parse_name()?;
        let mut attributes = Vec::new();

        while !self.source.is_eof() {
            self.source.skip_whitespace();
            if self.source.starts_with("/>") || self.source.starts_with(">") {
                break;
            }

            let attr_name = self.parse_name()?;
            self.source.skip_whitespace();
            if !self.source.consume_prefix("=") {
                return Err(self.source.syntax_error(format!("Expected '=' after attribute '{attr_name}'")));
            }
            self.source.skip_whitespace();
            let attr_raw_val = self.parse_quoted_string()?;
            let attr_val = self.entity_mapper.expand(&attr_raw_val)?;

            attributes.push(Attribute::new(attr_name, attr_val));

            if attributes.len() > self.options.max_attribute_count {
                return Err(XmlError::SecurityLimitExceeded(format!(
                    "Maximum attribute count per element ({}) exceeded",
                    self.options.max_attribute_count
                )));
            }
            self.total_attribute_count += 1;
            if self.total_attribute_count > self.options.max_total_attribute_count {
                return Err(XmlError::SecurityLimitExceeded(format!(
                    "Maximum total document attribute count ({}) exceeded",
                    self.options.max_total_attribute_count
                )));
            }
        }

        let is_self_closing = self.source.consume_prefix("/>");
        if !is_self_closing && !self.source.consume_prefix(">") {
            return Err(self.source.syntax_error(format!("Unclosed start tag for element '<{tag_name}>'")));
        }

        let elem_id = doc.add_node(NodeKind::Element {
            name: tag_name.clone(),
            attributes,
        });

        if is_self_closing {
            return Ok(elem_id);
        }

        // Parse Children
        while !self.source.is_eof() {
            if self.source.starts_with("</") {
                // End Tag
                self.source.consume_prefix("</");
                let end_name = self.parse_name()?;
                self.source.skip_whitespace();
                if !self.source.consume_prefix(">") {
                    return Err(self.source.syntax_error(format!("Unclosed end tag '</{end_name}>'")));
                }
                if end_name != tag_name {
                    return Err(self.source.syntax_error(format!(
                        "Mismatched closing tag: expected '</{tag_name}>', found '</{end_name}>'"
                    )));
                }
                return Ok(elem_id);
            } else if self.source.starts_with("<![CDATA[") {
                let cdata_id = self.parse_cdata(doc)?;
                doc.append_child(elem_id, cdata_id)?;
            } else if self.source.starts_with("<!--") {
                let comment_id = self.parse_comment(doc)?;
                doc.append_child(elem_id, comment_id)?;
            } else if self.source.starts_with("<?") {
                let pi_id = self.parse_pi(doc)?;
                doc.append_child(elem_id, pi_id)?;
            } else if self.source.starts_with("<") {
                let child_elem_id = self.parse_element(doc, depth + 1)?;
                doc.append_child(elem_id, child_elem_id)?;
            } else {
                let text_id = self.parse_text(doc)?;
                doc.append_child(elem_id, text_id)?;
            }
        }

        Err(self.source.syntax_error(format!("Unclosed element '<{tag_name}>' at EOF")))
    }

    fn parse_cdata(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume_prefix("<![CDATA[");
        let mut content = String::new();

        while !self.source.starts_with("]]>") && !self.source.is_eof() {
            if let Some(ch) = self.source.next_char() {
                content.push(ch);
            } else {
                break;
            }
        }

        if !self.source.consume_prefix("]]>") {
            return Err(self.source.syntax_error("Unclosed CDATA section"));
        }

        let node_id = doc.add_node(NodeKind::CData(content));
        Ok(node_id)
    }

    fn parse_text(&mut self, doc: &mut Document) -> Result<NodeId> {
        let mut raw_text = String::new();

        while !self.source.starts_with("<") && !self.source.is_eof() {
            if let Some(ch) = self.source.next_char() {
                raw_text.push(ch);
            } else {
                break;
            }
        }

        if raw_text.len() > self.options.max_text_node_size {
            return Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum text node size ({}) exceeded",
                self.options.max_text_node_size
            )));
        }

        let expanded_text = self.entity_mapper.expand(&raw_text)?;
        let node_id = doc.add_node(NodeKind::Text(expanded_text));
        Ok(node_id)
    }

    fn parse_name(&mut self) -> Result<String> {
        let mut name = String::new();
        while let Some(ch) = self.source.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == ':' || ch == '-' || ch == '.' {
                name.push(self.source.next_char().unwrap());
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(self.source.syntax_error("Expected valid XML identifier / name"));
        }
        Ok(name)
    }

    fn parse_quoted_string(&mut self) -> Result<String> {
        let quote = self.source.next_char().ok_or_else(|| self.source.syntax_error("Expected quote"))?;
        if quote != '"' && quote != '\'' {
            return Err(self.source.syntax_error(format!("Expected quote ('\"' or '\''), found '{quote}'")));
        }

        let mut val = String::new();
        while let Some(ch) = self.source.peek() {
            if ch == quote {
                self.source.next_char();
                return Ok(val);
            }
            val.push(self.source.next_char().unwrap());
        }

        Err(self.source.syntax_error("Unclosed quoted string"))
    }
}
