//! # XPath 1.0 Evaluator
//!
//! Evaluates [`XPathExpr`] ASTs against a DOM [`Document`] and context node ID.

use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use crate::xpath::ast::{Axis, NodeTest, XPathExpr, XPathOperator};

/// Dynamic value type resulting from an XPath 1.0 expression evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum XPathValue {
    /// Node-set matching query (`Vec<NodeId>`).
    NodeSet(Vec<NodeId>),
    /// Boolean result.
    Boolean(bool),
    /// Numeric result (`f64`).
    Number(f64),
    /// String result.
    String(String),
}

/// Evaluator computing XPath ASTs against a DOM [`Document`].
pub struct XPathEvaluator<'a> {
    /// Reference to target [`Document`].
    pub doc: &'a Document,
}

impl<'a> XPathEvaluator<'a> {
    /// Instantiates a new [`XPathEvaluator`] for a target [`Document`].
    pub fn new(doc: &'a Document) -> Self {
        Self { doc }
    }

    /// Evaluates an [`XPathExpr`] AST relative to a given `context_node` ID.
    pub fn evaluate(&self, expr: &XPathExpr, context_node: NodeId) -> Result<XPathValue> {
        match expr {
            XPathExpr::LiteralString(s) => Ok(XPathValue::String(s.clone())),
            XPathExpr::LiteralNumber(n) => Ok(XPathValue::Number(*n)),
            XPathExpr::Step {
                axis,
                test,
                predicates,
            } => {
                if *axis == Axis::Attribute {
                    if let NodeTest::Name(attr_name) = test {
                        if let Some(node) = self.doc.get_node(context_node) {
                            if let NodeKind::Element { attributes, .. } = &node.kind {
                                if let Some(attr) = attributes.iter().find(|a| &*a.name == attr_name) {
                                    return Ok(XPathValue::String(attr.value.to_string()));
                                }
                            }
                        }
                    } else if let NodeTest::Wildcard | NodeTest::AttributeWildcard = test {
                        if let Some(node) = self.doc.get_node(context_node) {
                            if let NodeKind::Element { attributes, .. } = &node.kind {
                                if !attributes.is_empty() {
                                    return Ok(XPathValue::NodeSet(vec![context_node]));
                                }
                            }
                        }
                    }
                    return Ok(XPathValue::NodeSet(Vec::new()));
                }

                let initial_nodes = self.evaluate_axis(axis, test, context_node)?;
                let filtered = self.apply_predicates(initial_nodes, predicates)?;
                Ok(XPathValue::NodeSet(filtered))
            }
            XPathExpr::Path(steps) => {
                let mut current_nodes = vec![context_node];
                for step in steps {
                    let mut next_nodes = Vec::new();
                    for &ctx in &current_nodes {
                        if let XPathValue::NodeSet(ns) = self.evaluate(step, ctx)? {
                            next_nodes.extend(ns);
                        }
                    }
                    next_nodes.sort_unstable();
                    next_nodes.dedup();
                    current_nodes = next_nodes;
                }
                Ok(XPathValue::NodeSet(current_nodes))
            }
            XPathExpr::BinaryOp { op, left, right } => {
                let lval = self.evaluate(left, context_node)?;
                let rval = self.evaluate(right, context_node)?;
                self.evaluate_binary(op, lval, rval)
            }
            XPathExpr::FunctionCall { name, args } => {
                self.evaluate_function(name, args, context_node)
            }
            XPathExpr::VariableRef(var) => Err(XmlError::XPathError(format!(
                "Unbound variable reference '${var}'"
            ))),
        }
    }

    fn evaluate_axis(&self, axis: &Axis, test: &NodeTest, context_node: NodeId) -> Result<Vec<NodeId>> {
        let candidates = match axis {
            Axis::Root => vec![self.doc.root_id().unwrap_or(0)],
            Axis::Child => {
                self.doc.get_node(context_node).map_or(Vec::new(), |n| n.children.clone())
            }
            Axis::SelfAxis => vec![context_node],
            Axis::Parent => self.doc.get_node(context_node).and_then(|n| n.parent).into_iter().collect(),
            Axis::Descendant => {
                let mut desc = Vec::new();
                self.collect_descendants(context_node, &mut desc, false);
                desc
            }
            Axis::DescendantOrSelf => {
                let mut desc = Vec::new();
                self.collect_descendants(context_node, &mut desc, true);
                desc
            }
            Axis::Ancestor => {
                let mut anc = Vec::new();
                let mut curr = context_node;
                while let Some(parent) = self.doc.get_node(curr).and_then(|n| n.parent) {
                    anc.push(parent);
                    curr = parent;
                }
                anc
            }
            Axis::AncestorOrSelf => {
                let mut anc = vec![context_node];
                let mut curr = context_node;
                while let Some(parent) = self.doc.get_node(curr).and_then(|n| n.parent) {
                    anc.push(parent);
                    curr = parent;
                }
                anc
            }
            Axis::FollowingSibling => {
                if let Some(parent_id) = self.doc.get_node(context_node).and_then(|n| n.parent) {
                    if let Some(parent) = self.doc.get_node(parent_id) {
                        if let Some(pos) = parent.children.iter().position(|&id| id == context_node) {
                            parent.children[pos + 1..].to_vec()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Axis::PrecedingSibling => {
                if let Some(parent_id) = self.doc.get_node(context_node).and_then(|n| n.parent) {
                    if let Some(parent) = self.doc.get_node(parent_id) {
                        if let Some(pos) = parent.children.iter().position(|&id| id == context_node) {
                            parent.children[..pos].to_vec()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Axis::Following => {
                let mut nodes = Vec::new();
                for id in context_node + 1..self.doc.len() as u32 {
                    nodes.push(id);
                }
                nodes
            }
            Axis::Preceding => {
                let mut nodes = Vec::new();
                for id in (0..context_node).rev() {
                    nodes.push(id);
                }
                nodes
            }
            _ => {
                let mut desc = Vec::new();
                self.collect_descendants(context_node, &mut desc, false);
                desc
            }
        };

        Ok(candidates
            .into_iter()
            .filter(|&nid| self.matches_node_test(nid, test))
            .collect())
    }

    fn collect_descendants(&self, node_id: NodeId, out: &mut Vec<NodeId>, include_self: bool) {
        if include_self {
            out.push(node_id);
        }
        if let Some(node) = self.doc.get_node(node_id) {
            for &c_id in &node.children {
                self.collect_descendants(c_id, out, true);
            }
        }
    }

    fn matches_node_test(&self, node_id: NodeId, test: &NodeTest) -> bool {
        let node = match self.doc.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };

        match test {
            NodeTest::Wildcard => matches!(node.kind, NodeKind::Element { .. }),
            NodeTest::Node => true,
            NodeTest::Text => matches!(node.kind, NodeKind::Text(_)),
            NodeTest::Comment => matches!(node.kind, NodeKind::Comment(_)),
            NodeTest::Name(target_name) => match &node.kind {
                NodeKind::Element { name, .. } => &**name == target_name,
                NodeKind::ProcessingInstruction { target, .. } => &**target == target_name,
                NodeKind::DocTypeDefinition { name, .. } => &**name == target_name,
                _ => false,
            },
            _ => true,
        }
    }

    fn apply_predicates(&self, mut nodes: Vec<NodeId>, predicates: &[XPathExpr]) -> Result<Vec<NodeId>> {
        for pred in predicates {
            let mut filtered = Vec::new();
            for (idx, &nid) in nodes.iter().enumerate() {
                let pval = self.evaluate(pred, nid)?;
                let is_match = match pval {
                    XPathValue::Number(n) => (n as usize) == (idx + 1),
                    XPathValue::Boolean(b) => b,
                    XPathValue::NodeSet(ns) => !ns.is_empty(),
                    XPathValue::String(s) => !s.is_empty(),
                };
                if is_match {
                    filtered.push(nid);
                }
            }
            nodes = filtered;
        }
        Ok(nodes)
    }

    fn evaluate_binary(&self, op: &XPathOperator, left: XPathValue, right: XPathValue) -> Result<XPathValue> {
        match op {
            XPathOperator::Or => Ok(XPathValue::Boolean(self.to_bool(&left) || self.to_bool(&right))),
            XPathOperator::And => Ok(XPathValue::Boolean(self.to_bool(&left) && self.to_bool(&right))),
            XPathOperator::Eq => Ok(XPathValue::Boolean(self.to_string(&left) == self.to_string(&right))),
            XPathOperator::NotEq => Ok(XPathValue::Boolean(self.to_string(&left) != self.to_string(&right))),
            XPathOperator::Lt => Ok(XPathValue::Boolean(self.to_number(&left) < self.to_number(&right))),
            XPathOperator::LtEq => Ok(XPathValue::Boolean(self.to_number(&left) <= self.to_number(&right))),
            XPathOperator::Gt => Ok(XPathValue::Boolean(self.to_number(&left) > self.to_number(&right))),
            XPathOperator::GtEq => Ok(XPathValue::Boolean(self.to_number(&left) >= self.to_number(&right))),
            XPathOperator::Plus => Ok(XPathValue::Number(self.to_number(&left) + self.to_number(&right))),
            XPathOperator::Minus => Ok(XPathValue::Number(self.to_number(&left) - self.to_number(&right))),
            XPathOperator::Multiply => Ok(XPathValue::Number(self.to_number(&left) * self.to_number(&right))),
            XPathOperator::Div => Ok(XPathValue::Number(self.to_number(&left) / self.to_number(&right))),
            XPathOperator::Mod => Ok(XPathValue::Number(self.to_number(&left) % self.to_number(&right))),
            XPathOperator::Union => {
                let mut ns1 = match left {
                    XPathValue::NodeSet(ns) => ns,
                    _ => Vec::new(),
                };
                let ns2 = match right {
                    XPathValue::NodeSet(ns) => ns,
                    _ => Vec::new(),
                };
                ns1.extend(ns2);
                ns1.sort_unstable();
                ns1.dedup();
                Ok(XPathValue::NodeSet(ns1))
            }
        }
    }

    fn evaluate_function(&self, name: &str, args: &[XPathExpr], ctx: NodeId) -> Result<XPathValue> {
        match name {
            "count" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("count() expects 1 argument".into()));
                }
                let val = self.evaluate(&args[0], ctx)?;
                let count = match val {
                    XPathValue::NodeSet(ns) => ns.len(),
                    _ => 0,
                };
                Ok(XPathValue::Number(count as f64))
            }
            "name" | "local-name" => {
                let target_node = if args.is_empty() {
                    ctx
                } else if let XPathValue::NodeSet(ns) = self.evaluate(&args[0], ctx)? {
                    *ns.first().unwrap_or(&ctx)
                } else {
                    ctx
                };
                let name_str = self
                    .doc
                    .get_node(target_node)
                    .map(|n| n.kind.name().to_string())
                    .unwrap_or_default();
                Ok(XPathValue::String(name_str))
            }
            "string" => {
                if args.is_empty() {
                    Ok(XPathValue::String(self.get_node_text(ctx)))
                } else {
                    let val = self.evaluate(&args[0], ctx)?;
                    Ok(XPathValue::String(self.to_string(&val)))
                }
            }
            "boolean" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("boolean() expects 1 argument".into()));
                }
                let val = self.evaluate(&args[0], ctx)?;
                Ok(XPathValue::Boolean(self.to_bool(&val)))
            }
            "not" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("not() expects 1 argument".into()));
                }
                let val = self.evaluate(&args[0], ctx)?;
                Ok(XPathValue::Boolean(!self.to_bool(&val)))
            }
            "number" => {
                if args.is_empty() {
                    Ok(XPathValue::Number(self.to_number(&XPathValue::String(self.get_node_text(ctx)))))
                } else {
                    let val = self.evaluate(&args[0], ctx)?;
                    Ok(XPathValue::Number(self.to_number(&val)))
                }
            }
            "sum" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("sum() expects 1 argument".into()));
                }
                let val = self.evaluate(&args[0], ctx)?;
                let total = match val {
                    XPathValue::NodeSet(ns) => ns
                        .iter()
                        .map(|&nid| self.get_node_text(nid).parse::<f64>().unwrap_or(0.0))
                        .sum(),
                    _ => 0.0,
                };
                Ok(XPathValue::Number(total))
            }
            "contains" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("contains() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate(&args[0], ctx)?);
                let s2 = self.to_string(&self.evaluate(&args[1], ctx)?);
                Ok(XPathValue::Boolean(s1.contains(&s2)))
            }
            "starts-with" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("starts-with() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate(&args[0], ctx)?);
                let s2 = self.to_string(&self.evaluate(&args[1], ctx)?);
                Ok(XPathValue::Boolean(s1.starts_with(&s2)))
            }
            "string-length" => {
                let s = if args.is_empty() {
                    self.get_node_text(ctx)
                } else {
                    self.to_string(&self.evaluate(&args[0], ctx)?)
                };
                Ok(XPathValue::Number(s.chars().count() as f64))
            }
            "concat" => {
                let mut res = String::new();
                for arg in args {
                    let v = self.evaluate(arg, ctx)?;
                    res.push_str(&self.to_string(&v));
                }
                Ok(XPathValue::String(res))
            }
            "substring" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(XmlError::XPathError("substring() expects 2 or 3 arguments".into()));
                }
                let s = self.to_string(&self.evaluate(&args[0], ctx)?);
                let start_idx = self.to_number(&self.evaluate(&args[1], ctx)?) as i64 - 1;
                let chars: Vec<char> = s.chars().collect();
                if start_idx < 0 || (start_idx as usize) >= chars.len() {
                    return Ok(XPathValue::String(String::new()));
                }
                let start = start_idx as usize;
                let len = if args.len() == 3 {
                    self.to_number(&self.evaluate(&args[2], ctx)?) as usize
                } else {
                    chars.len() - start
                };
                let end = (start + len).min(chars.len());
                let sub: String = chars[start..end].iter().collect();
                Ok(XPathValue::String(sub))
            }
            "substring-before" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("substring-before() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate(&args[0], ctx)?);
                let s2 = self.to_string(&self.evaluate(&args[1], ctx)?);
                if let Some(pos) = s1.find(&s2) {
                    Ok(XPathValue::String(s1[..pos].to_string()))
                } else {
                    Ok(XPathValue::String(String::new()))
                }
            }
            "substring-after" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("substring-after() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate(&args[0], ctx)?);
                let s2 = self.to_string(&self.evaluate(&args[1], ctx)?);
                if let Some(pos) = s1.find(&s2) {
                    Ok(XPathValue::String(s1[pos + s2.len()..].to_string()))
                } else {
                    Ok(XPathValue::String(String::new()))
                }
            }
            "normalize-space" => {
                let s = if args.is_empty() {
                    self.get_node_text(ctx)
                } else {
                    self.to_string(&self.evaluate(&args[0], ctx)?)
                };
                let words: Vec<&str> = s.split_whitespace().collect();
                Ok(XPathValue::String(words.join(" ")))
            }
            "translate" => {
                if args.len() != 3 {
                    return Err(XmlError::XPathError("translate() expects 3 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate(&args[0], ctx)?);
                let from = self.to_string(&self.evaluate(&args[1], ctx)?);
                let to = self.to_string(&self.evaluate(&args[2], ctx)?);
                let from_chars: Vec<char> = from.chars().collect();
                let to_chars: Vec<char> = to.chars().collect();

                let mut result = String::new();
                for ch in s1.chars() {
                    if let Some(pos) = from_chars.iter().position(|&c| c == ch) {
                        if pos < to_chars.len() {
                            result.push(to_chars[pos]);
                        }
                    } else {
                        result.push(ch);
                    }
                }
                Ok(XPathValue::String(result))
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("floor() expects 1 argument".into()));
                }
                let num = self.to_number(&self.evaluate(&args[0], ctx)?);
                Ok(XPathValue::Number(num.floor()))
            }
            "ceiling" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("ceiling() expects 1 argument".into()));
                }
                let num = self.to_number(&self.evaluate(&args[0], ctx)?);
                Ok(XPathValue::Number(num.ceil()))
            }
            "round" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("round() expects 1 argument".into()));
                }
                let num = self.to_number(&self.evaluate(&args[0], ctx)?);
                Ok(XPathValue::Number(num.round()))
            }
            "true" => Ok(XPathValue::Boolean(true)),
            "false" => Ok(XPathValue::Boolean(false)),
            _ => Err(XmlError::XPathError(format!("Unknown XPath function: '{name}'"))),
        }
    }

    fn get_node_text(&self, node_id: NodeId) -> String {
        let mut text = String::new();
        if let Some(node) = self.doc.get_node(node_id) {
            match &node.kind {
                NodeKind::Text(t) | NodeKind::CData(t) => text.push_str(t),
                NodeKind::Element { .. } | NodeKind::Root | NodeKind::Prolog => {
                    for &c_id in &node.children {
                        text.push_str(&self.get_node_text(c_id));
                    }
                }
                _ => {}
            }
        }
        text
    }

    /// Converts an [`XPathValue`] to boolean.
    pub fn to_bool(&self, val: &XPathValue) -> bool {
        match val {
            XPathValue::Boolean(b) => *b,
            XPathValue::Number(n) => *n != 0.0 && !n.is_nan(),
            XPathValue::String(s) => !s.is_empty(),
            XPathValue::NodeSet(ns) => !ns.is_empty(),
        }
    }

    /// Converts an [`XPathValue`] to floating-point number.
    pub fn to_number(&self, val: &XPathValue) -> f64 {
        match val {
            XPathValue::Number(n) => *n,
            XPathValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
            XPathValue::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            XPathValue::NodeSet(ns) => {
                if let Some(&first) = ns.first() {
                    self.get_node_text(first).parse::<f64>().unwrap_or(f64::NAN)
                } else {
                    f64::NAN
                }
            }
        }
    }

    /// Converts an [`XPathValue`] to string representation.
    pub fn to_string(&self, val: &XPathValue) -> String {
        match val {
            XPathValue::String(s) => s.clone(),
            XPathValue::Boolean(b) => b.to_string(),
            XPathValue::Number(n) => n.to_string(),
            XPathValue::NodeSet(ns) => {
                if let Some(&first) = ns.first() {
                    self.get_node_text(first)
                } else {
                    String::new()
                }
            }
        }
    }
}
