#[derive(Debug, Clone, PartialEq)]
pub enum Axis {
    Root,
    Child,
    Descendant,
    Parent,
    Ancestor,
    FollowingSibling,
    PrecedingSibling,
    Following,
    Preceding,
    Attribute,
    Namespace,
    SelfAxis,
    DescendantOrSelf,
    AncestorOrSelf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeTest {
    Name(String),
    Wildcard,
    AttributeWildcard,
    Text,
    Comment,
    ProcessingInstruction(Option<String>),
    Node,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XPathOperator {
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Multiply,
    Div,
    Mod,
    Union,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XPathExpr {
    LiteralString(String),
    LiteralNumber(f64),
    Step {
        axis: Axis,
        test: NodeTest,
        predicates: Vec<XPathExpr>,
    },
    Path(Vec<XPathExpr>),
    BinaryOp {
        op: XPathOperator,
        left: Box<XPathExpr>,
        right: Box<XPathExpr>,
    },
    FunctionCall {
        name: String,
        args: Vec<XPathExpr>,
    },
    VariableRef(String),
}
