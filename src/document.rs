use crate::error::{Result, XmlError};
use crate::node::{NodeData, NodeId, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    nodes: Vec<NodeData>,
    root_id: Option<NodeId>,
    prolog_id: Option<NodeId>,
    declaration_id: Option<NodeId>,
    dtd_id: Option<NodeId>,
}

impl Document {
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

    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(NodeData::new(id, kind));
        id
    }

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

    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(id)
    }

    pub fn root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    pub fn prolog_id(&self) -> Option<NodeId> {
        self.prolog_id
    }

    pub fn declaration_id(&self) -> Option<NodeId> {
        self.declaration_id
    }

    pub fn set_declaration_id(&mut self, id: NodeId) {
        self.declaration_id = Some(id);
    }

    pub fn dtd_id(&self) -> Option<NodeId> {
        self.dtd_id
    }

    pub fn set_dtd_id(&mut self, id: NodeId) {
        self.dtd_id = Some(id);
    }

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

    pub fn nodes(&self) -> &[NodeData] {
        &self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
