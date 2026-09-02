//! # Node Data Types
//!
//! Defines node kinds, attributes, arena node indices, and node metadata.

/// Unique index identifier for a node within the arena [`Vec<NodeData>`].
pub type NodeId = usize;

/// Key-value attribute pair on an XML Element tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Attribute name/key (e.g., `id`, `xmlns:xs`, `category`).
    pub name: String,
    /// Attribute string value.
    pub value: String,
}

impl Attribute {
    /// Instantiates a new [`Attribute`] from name and value.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Enum representing all supported XML node variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Top-level Prolog virtual container.
    Prolog,
    /// XML Declaration (`<?xml version="..." encoding="..." standalone="..."?>`).
    Declaration {
        version: String,
        encoding: Option<String>,
        standalone: Option<bool>,
    },
    /// Top-level Root virtual container.
    Root,
    /// XML Element tag (`<name attr="val">...</name>`).
    Element {
        name: String,
        attributes: Vec<Attribute>,
    },
    /// Text node content.
    Text(String),
    /// CDATA section (`<!CDATA[...]]>`).
    CData(String),
    /// XML Comment (`<!-- ... -->`).
    Comment(String),
    /// Processing Instruction (`<?target data?>`).
    ProcessingInstruction {
        target: String,
        data: String,
    },
    /// DTD DOCTYPE Definition (`<!DOCTYPE name ...>`).
    DocTypeDefinition {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
        internal_subset: Option<String>,
    },
    /// Entity Reference (`&name;`).
    EntityReference(String),
}

impl NodeKind {
    /// Returns the element name or node kind label.
    pub fn name(&self) -> &str {
        match self {
            NodeKind::Element { name, .. } => name,
            NodeKind::ProcessingInstruction { target, .. } => target,
            NodeKind::DocTypeDefinition { name, .. } => name,
            NodeKind::EntityReference(name) => name,
            NodeKind::Declaration { .. } => "xml",
            NodeKind::CData(_) => "#cdata",
            NodeKind::Comment(_) => "#comment",
            NodeKind::Text(_) => "#text",
            NodeKind::Prolog => "#prolog",
            NodeKind::Root => "#document",
        }
    }
}

/// Node container storing node identity, parent link, children links, and node payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeData {
    /// Arena Node ID index.
    pub id: NodeId,
    /// Parent Node ID in arena if linked.
    pub parent: Option<NodeId>,
    /// List of child Node IDs in arena.
    pub children: Vec<NodeId>,
    /// Variant payload describing node type and data.
    pub kind: NodeKind,
}

impl NodeData {
    /// Instantiates a new [`NodeData`] node for a given ID and [`NodeKind`].
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            kind,
        }
    }
}
