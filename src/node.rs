pub type NodeId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl Attribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Prolog,
    Declaration {
        version: String,
        encoding: Option<String>,
        standalone: Option<bool>,
    },
    Root,
    Element {
        name: String,
        attributes: Vec<Attribute>,
    },
    Text(String),
    CData(String),
    Comment(String),
    ProcessingInstruction {
        target: String,
        data: String,
    },
    DocTypeDefinition {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
        internal_subset: Option<String>,
    },
    EntityReference(String),
}

impl NodeKind {
    pub fn name(&self) -> &str {
        match self {
            NodeKind::Prolog => "Prolog",
            NodeKind::Declaration { .. } => "Declaration",
            NodeKind::Root => "Root",
            NodeKind::Element { name, .. } => name.as_str(),
            NodeKind::Text(_) => "Text",
            NodeKind::CData(_) => "CData",
            NodeKind::Comment(_) => "Comment",
            NodeKind::ProcessingInstruction { target, .. } => target.as_str(),
            NodeKind::DocTypeDefinition { name, .. } => name.as_str(),
            NodeKind::EntityReference(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeData {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: NodeKind,
}

impl NodeData {
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            kind,
        }
    }
}
