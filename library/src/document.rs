//! # Document (DOM Arena Model)
//!
//! Provides the primary [`Document`] container representing an XML DOM tree stored in a flat arena vector.

use crate::error::{Result, XmlError};
use crate::node::{NodeData, NodeId, NodeKind};

/// In-memory representation of an XML Document stored in a flat arena [`Vec<NodeData>`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    /// Arena vector storing all nodes indexed by [`NodeId`].
    nodes: Vec<NodeData>,
    /// Optional Node ID of top-level virtual Root container.
    root_id: Option<NodeId>,
    /// Optional Node ID of Prolog container.
    prolog_id: Option<NodeId>,
    /// Optional Node ID of XML Declaration node.
    declaration_id: Option<NodeId>,
    /// Optional Node ID of DTD DOCTYPE node.
    dtd_id: Option<NodeId>,
}

impl Document {
    /// Creates a new empty `Document` with initialized Prolog and Root virtual container nodes.
    pub fn new() -> Self {
        let mut doc = Self {
            nodes: Vec::new(),
            root_id: None,
            prolog_id: None,
            declaration_id: None,
            dtd_id: None,
        };
        // Create top-level Prolog container node
        let prolog_id = doc.add_node(NodeKind::Prolog);
        doc.prolog_id = Some(prolog_id);
        
        // Create root virtual container node
        let root_id = doc.add_node(NodeKind::Root);
        doc.root_id = Some(root_id);
        
        doc
    }

    /// Appends a new node kind to the arena and returns its unique [`NodeId`].
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(NodeData::new(id, kind));
        id
    }

    /// Links a child node to a parent node within the arena.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<()> {
        if parent_id >= self.nodes.len() || child_id >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        self.nodes[child_id].parent = Some(parent_id);
        if !self.nodes[parent_id].children.contains(&child_id) {
            self.nodes[parent_id].children.push(child_id);
        }
        Ok(())
    }

    /// Immutable lookup for a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(id)
    }

    /// Mutable lookup for a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(id)
    }

    /// Returns the ID of the root virtual container node.
    pub fn root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    /// Returns the ID of the Prolog container node.
    pub fn prolog_id(&self) -> Option<NodeId> {
        self.prolog_id
    }

    /// Returns the ID of the XML Declaration node if present.
    pub fn declaration_id(&self) -> Option<NodeId> {
        self.declaration_id
    }

    /// Sets the XML Declaration node ID.
    pub fn set_declaration_id(&mut self, id: NodeId) {
        self.declaration_id = Some(id);
    }

    /// Returns the ID of the DTD DOCTYPE node if present.
    pub fn dtd_id(&self) -> Option<NodeId> {
        self.dtd_id
    }

    /// Sets the DTD DOCTYPE node ID.
    pub fn set_dtd_id(&mut self, id: NodeId) {
        self.dtd_id = Some(id);
    }

    /// Returns the ID of the primary document element node (first child Element of Root).
    pub fn root_element_id(&self) -> Option<NodeId> {
        let root_id = self.root_id?;
        let root_node = self.get_node(root_id)?;
        for &child_id in &root_node.children {
            if let Some(child) = self.get_node(child_id) {
                if matches!(child.kind, NodeKind::Element { .. }) {
                    return Some(child_id);
                }
            }
        }
        None
    }

    /// Returns a list of child node IDs for a given parent node ID.
    pub fn get_children(&self, id: NodeId) -> Vec<NodeId> {
        self.get_node(id).map(|n| n.children.clone()).unwrap_or_default()
    }

    /// Returns the parent node ID for a given node ID.
    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id).and_then(|n| n.parent)
    }

    /// Recursively extracts concatenated text content for a given node and its descendants.
    pub fn get_text_content(&self, id: NodeId) -> String {
        let mut text = String::new();
        if let Some(node) = self.get_node(id) {
            match &node.kind {
                NodeKind::Text(t) => text.push_str(t),
                _ => {
                    for &child_id in &node.children {
                        if let Some(child) = self.get_node(child_id) {
                            match &child.kind {
                                NodeKind::Text(t) => text.push_str(t),
                                NodeKind::Element { .. } => text.push_str(&self.get_text_content(child_id)),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        text
    }

    /// Returns a slice of all nodes stored in the arena.
    pub fn nodes(&self) -> &[NodeData] {
        &self.nodes
    }

    /// Returns the total number of nodes stored in the arena.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the arena contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
