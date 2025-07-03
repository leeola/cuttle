use crate::{Node, NodeGraph, NodeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub position: Option<Position>,
    pub selected: bool,
    pub expanded: bool,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            position: None,
            selected: false,
            expanded: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeWithMetadata {
    pub node: Node,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphWithMetadata {
    pub graph: NodeGraph,
    pub node_metadata: std::collections::HashMap<NodeId, NodeMetadata>,
}

impl NodeGraphWithMetadata {
    pub fn new() -> Self {
        Self {
            graph: NodeGraph::new(),
            node_metadata: std::collections::HashMap::new(),
        }
    }

    pub fn add_node_with_metadata(&mut self, node: Node, metadata: NodeMetadata) {
        let id = node.id().clone();
        self.graph.add_node(node);
        self.node_metadata.insert(id, metadata);
    }

    pub fn add_node(&mut self, node: Node) {
        let id = node.id().clone();
        self.graph.add_node(node);
        self.node_metadata.insert(id, NodeMetadata::default());
    }
}

impl Default for NodeGraphWithMetadata {
    fn default() -> Self {
        Self::new()
    }
}
