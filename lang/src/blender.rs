use crate::{Node, NodeGraph, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlenderNode {
    pub node_type: String,
    pub location: (f64, f64),
    pub inputs: Vec<BlenderSocket>,
    pub outputs: Vec<BlenderSocket>,
    pub parameters: std::collections::HashMap<String, BlenderValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlenderSocket {
    pub name: String,
    pub socket_type: String,
    pub default_value: Option<BlenderValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlenderValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Vector(f64, f64, f64),
    Color(f64, f64, f64, f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlenderNodeGraph {
    pub nodes: Vec<BlenderNode>,
    pub links: Vec<BlenderLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlenderLink {
    pub from_node: usize,
    pub from_socket: String,
    pub to_node: usize,
    pub to_socket: String,
}

impl From<Value> for BlenderValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Integer(i) => BlenderValue::Integer(i),
            Value::Float(f) => BlenderValue::Float(f),
            Value::Boolean(b) => BlenderValue::Boolean(b),
            Value::Vector(x, y, z) => BlenderValue::Vector(x, y, z),
            Value::Color(r, g, b, a) => BlenderValue::Color(r, g, b, a),
        }
    }
}

impl From<BlenderValue> for Value {
    fn from(value: BlenderValue) -> Self {
        match value {
            BlenderValue::Integer(i) => Value::Integer(i),
            BlenderValue::Float(f) => Value::Float(f),
            BlenderValue::Boolean(b) => Value::Boolean(b),
            BlenderValue::Vector(x, y, z) => Value::Vector(x, y, z),
            BlenderValue::Color(r, g, b, a) => Value::Color(r, g, b, a),
            BlenderValue::String(_) => Value::Boolean(false), // fallback
        }
    }
}

impl From<Node> for BlenderNode {
    fn from(node: Node) -> Self {
        match node {
            Node::Value { value, .. } => BlenderNode {
                node_type: "ShaderNodeValue".to_string(),
                location: (0.0, 0.0),
                inputs: vec![],
                outputs: vec![BlenderSocket {
                    name: "Value".to_string(),
                    socket_type: "NodeSocketFloat".to_string(),
                    default_value: Some(value.into()),
                }],
                parameters: std::collections::HashMap::new(),
            },
            Node::Cube { size, .. } => {
                let mut parameters = std::collections::HashMap::new();
                parameters.insert("size".to_string(), size.clone().into());
                BlenderNode {
                    node_type: "GeometryNodeMeshCube".to_string(),
                    location: (0.0, 0.0),
                    inputs: vec![BlenderSocket {
                        name: "Size".to_string(),
                        socket_type: "NodeSocketVector".to_string(),
                        default_value: Some(size.into()),
                    }],
                    outputs: vec![BlenderSocket {
                        name: "Mesh".to_string(),
                        socket_type: "NodeSocketGeometry".to_string(),
                        default_value: None,
                    }],
                    parameters,
                }
            }
        }
    }
}

impl From<NodeGraph> for BlenderNodeGraph {
    fn from(graph: NodeGraph) -> Self {
        let blender_nodes: Vec<BlenderNode> = graph.nodes.into_iter().map(|n| n.into()).collect();

        BlenderNodeGraph {
            nodes: blender_nodes,
            links: vec![], // TODO: Convert connections
        }
    }
}
