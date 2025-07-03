use serde::{Deserialize, Serialize};

pub mod ast;
pub mod blender;
pub mod error;
pub mod parser;

pub use ast::*;
pub use blender::*;
pub use error::*;
pub use parser::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Vector(f64, f64, f64),
    Color(f64, f64, f64, f64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: NodeId,
    pub from_output: String,
    pub to_node: NodeId,
    pub to_input: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Value { id: NodeId, value: Value },
    Cube { id: NodeId, size: Value },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraph {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

impl Node {
    pub fn id(&self) -> &NodeId {
        match self {
            Node::Value { id, .. } => id,
            Node::Cube { id, .. } => id,
        }
    }
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    pub fn find_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id() == id)
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_parse_and_convert_cube() {
        let input = "cube { size: 2.0 }";
        let graph = parse_geometry_nodes(input).expect("Failed to parse cube in test");

        // Test parsing
        assert_eq!(graph.nodes.len(), 1);
        match &graph.nodes[0] {
            Node::Cube { size, .. } => {
                assert_eq!(size, &Value::Float(2.0));
            }
            _ => panic!("Expected Cube node"),
        }

        // Test conversion to Blender format
        let blender_graph: BlenderNodeGraph = graph.into();
        assert_eq!(blender_graph.nodes.len(), 1);
        assert_eq!(blender_graph.nodes[0].node_type, "GeometryNodeMeshCube");
        assert_eq!(blender_graph.nodes[0].inputs[0].name, "Size");
    }

    #[test]
    fn test_parse_and_convert_value() {
        let input = "value 42";
        let graph = parse_geometry_nodes(input).expect("Failed to parse cube in test");

        // Test parsing
        assert_eq!(graph.nodes.len(), 1);
        match &graph.nodes[0] {
            Node::Value { value, .. } => {
                assert_eq!(value, &Value::Integer(42));
            }
            _ => panic!("Expected Value node"),
        }

        // Test conversion to Blender format
        let blender_graph: BlenderNodeGraph = graph.into();
        assert_eq!(blender_graph.nodes.len(), 1);
        assert_eq!(blender_graph.nodes[0].node_type, "ShaderNodeValue");
        assert_eq!(blender_graph.nodes[0].outputs[0].name, "Value");
    }

    #[test]
    fn test_bidirectional_value_conversion() {
        let original_value = Value::Float(std::f64::consts::PI);
        let blender_value: BlenderValue = original_value.clone().into();
        let converted_back: Value = blender_value.into();
        assert_eq!(original_value, converted_back);
    }

    #[test]
    fn test_serialization() {
        let graph = NodeGraph {
            nodes: vec![Node::Cube {
                id: NodeId("cube_0".to_string()),
                size: Value::Float(2.0),
            }],
            connections: vec![],
        };

        // Test JSON serialization
        let json = serde_json::to_string(&graph).expect("Failed to serialize");
        let deserialized: NodeGraph = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(graph, deserialized);
    }
}
