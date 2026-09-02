//! # Node Data Types
//!
//! Defines memory-optimized node kinds, boxed attributes, arena node indices, and node metadata.

/// Unique index identifier for a node within the arena [`Vec<NodeData>`].
pub type NodeId = usize;

/// Key-value attribute pair on an XML Element tag stored with compact boxed strings (`Box<str>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Attribute name/key (e.g., `id`, `xmlns:xs`, `category`).
    pub name: Box<str>,
    /// Attribute string value.
    pub value: Box<str>,
}

impl Attribute {
    /// Instantiates a new [`Attribute`] from name and value.
    pub fn new(name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Enum representing all supported XML node variants stored with memory-compact boxed types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Top-level Prolog virtual container.
    Prolog,
    /// XML Declaration (`<?xml version="..." encoding="..." standalone="..."?>`).
    Declaration {
        version: Box<str>,
        encoding: Option<Box<str>>,
        standalone: Option<bool>,
    },
    /// Top-level Root virtual container.
    Root,
    /// XML Element tag (`<name attr="val">...</name>`).
    Element {
        name: Box<str>,
        attributes: Vec<Attribute>,
    },
    /// Text node content.
    Text(Box<str>),
    /// CDATA section (`<!CDATA[...]]>`).
    CData(Box<str>),
    /// XML Comment (`<!-- ... -->`).
    Comment(Box<str>),
    /// Processing Instruction (`<?target data?>`).
    ProcessingInstruction {
        target: Box<str>,
        data: Box<str>,
    },
    /// DTD DOCTYPE Definition (`<!DOCTYPE name ...>`).
    DocTypeDefinition {
        name: Box<str>,
        public_id: Option<Box<str>>,
        system_id: Option<Box<str>>,
        internal_subset: Option<Box<str>>,
    },
    /// Entity Reference (`&name;`).
    EntityReference(Box<str>),
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
