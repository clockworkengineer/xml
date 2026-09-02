pub mod ast;
pub mod evaluator;
pub mod lexer;
pub mod parser;

pub use ast::{Axis, NodeTest, XPathExpr, XPathOperator};
pub use evaluator::{XPathEvaluator, XPathValue};
pub use lexer::{Token, XPathLexer};
pub use parser::XPathParser;

use crate::document::Document;
use crate::error::Result;
use crate::node::NodeId;

pub struct XPathEngine<'a> {
    evaluator: XPathEvaluator<'a>,
}

impl<'a> XPathEngine<'a> {
    pub fn new(doc: &'a Document) -> Self {
        Self {
            evaluator: XPathEvaluator::new(doc),
        }
    }

    pub fn evaluate(&self, expression: &str, context_node: Option<NodeId>) -> Result<XPathValue> {
        let mut parser = XPathParser::new(expression)?;
        let expr = parser.parse_expression()?;
        let ctx = context_node.unwrap_or_else(|| self.evaluator.doc.root_id().unwrap_or(0));
        self.evaluator.evaluate(&expr, ctx)
    }

    pub fn evaluate_nodes(&self, expression: &str, context_node: Option<NodeId>) -> Result<Vec<NodeId>> {
        match self.evaluate(expression, context_node)? {
            XPathValue::NodeSet(ns) => Ok(ns),
            _ => Ok(Vec::new()),
        }
    }
}
