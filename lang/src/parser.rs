use crate::{ErrorReporter, Node, NodeGraph, NodeId, ParseError, ParseResult, Value};
use chumsky::error::Rich;
use chumsky::primitive::{choice, end, just};
use chumsky::{IterParser, Parser, extra, text};

#[derive(Clone, Debug)]
pub enum ParsedNode {
    Cube { size: Option<Value> },
    Value(Value),
}

fn number_parser<'src>() -> impl Parser<'src, &'src str, f64, extra::Err<Rich<'src, char>>> {
    text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<f64>()
                .map_err(|_| Rich::custom(span, format!("'{s}' is not a valid number")))
        })
}

fn value_parser<'src>() -> impl Parser<'src, &'src str, Value, extra::Err<Rich<'src, char>>> {
    let float = text::int(10)
        .then(just('.').then(text::digits(10)))
        .to_slice()
        .try_map(|s: &str, span| {
            s.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| Rich::custom(span, format!("'{s}' is not a valid float")))
        });

    let integer = text::int(10).to_slice().try_map(|s: &str, span| {
        s.parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| Rich::custom(span, format!("'{s}' is not a valid integer")))
    });

    let boolean = just("true")
        .to(Value::Boolean(true))
        .or(just("false").to(Value::Boolean(false)));

    let vector = just('(')
        .ignore_then(
            number_parser()
                .separated_by(just(',').padded())
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(')'))
        .try_map(|coords, span| {
            if coords.len() == 3 {
                Ok(Value::Vector(coords[0], coords[1], coords[2]))
            } else {
                Err(Rich::custom(
                    span,
                    format!(
                        "Vector must have exactly 3 components, found {}",
                        coords.len()
                    ),
                ))
            }
        });

    let color = just('(')
        .ignore_then(
            number_parser()
                .separated_by(just(',').padded())
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(')'))
        .try_map(|coords, span| {
            if coords.len() == 4 {
                Ok(Value::Color(coords[0], coords[1], coords[2], coords[3]))
            } else {
                Err(Rich::custom(
                    span,
                    format!(
                        "Color must have exactly 4 components, found {}",
                        coords.len()
                    ),
                ))
            }
        });

    choice((float, integer, boolean, vector, color))
}

fn cube_parser<'src>() -> impl Parser<'src, &'src str, ParsedNode, extra::Err<Rich<'src, char>>> {
    let with_braces = just("cube")
        .ignore_then(just('{').padded())
        .ignore_then(just("size:").padded().ignore_then(value_parser()))
        .then_ignore(just('}').padded())
        .map(|size| ParsedNode::Cube { size: Some(size) });

    let without_braces = just("cube").map(|_| ParsedNode::Cube { size: None });

    choice((with_braces, without_braces))
}

fn value_node_parser<'src>()
-> impl Parser<'src, &'src str, ParsedNode, extra::Err<Rich<'src, char>>> {
    just("value")
        .ignore_then(value_parser().padded())
        .map(ParsedNode::Value)
}

fn node_parser<'src>() -> impl Parser<'src, &'src str, ParsedNode, extra::Err<Rich<'src, char>>> {
    choice((cube_parser(), value_node_parser())).padded()
}

pub fn parse_geometry_nodes(input: &str) -> ParseResult<NodeGraph> {
    let parser = node_parser().then_ignore(end());

    let (parsed_node, errors) = parser.parse(input).into_output_errors();

    if !errors.is_empty() {
        let parse_errors = errors
            .into_iter()
            .map(ParseError::from_rich)
            .collect::<Vec<_>>();
        return Err(parse_errors);
    }

    if let Some(parsed_node) = parsed_node {
        let mut graph = NodeGraph::new();
        let node_counter = 0;

        let node = match parsed_node {
            ParsedNode::Cube { size } => {
                let size_value = size.unwrap_or(Value::Float(2.0));
                Node::Cube {
                    id: NodeId(format!("cube_{node_counter}")),
                    size: size_value,
                }
            }
            ParsedNode::Value(value) => Node::Value {
                id: NodeId(format!("value_{node_counter}")),
                value,
            },
        };

        graph.add_node(node);
        Ok(graph)
    } else {
        Err(vec![ParseError::UnexpectedEndOfInput {
            span: (0..input.len()).into(),
            expected: vec!["cube".to_string(), "value".to_string()],
        }])
    }
}

pub fn parse_geometry_nodes_with_errors(input: &str) -> Result<NodeGraph, String> {
    match parse_geometry_nodes(input) {
        Ok(graph) => Ok(graph),
        Err(errors) => {
            let mut reporter = ErrorReporter::new();
            let error_report = reporter.report_errors(&errors, input, "<input>");
            Err(error_report)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_cube() {
        let input = "cube { size: 2.0 }";
        let result = parse_geometry_nodes(input);
        assert!(result.is_ok());
        let graph = result.expect("Failed to parse cube");
        assert_eq!(graph.nodes.len(), 1);
        match &graph.nodes[0] {
            Node::Cube { size, .. } => {
                assert_eq!(size, &Value::Float(2.0));
            }
            _ => panic!("Expected Cube node"),
        }
    }

    #[test]
    fn parse_value_node() {
        let input = "value 42";
        let result = parse_geometry_nodes(input);
        assert!(result.is_ok());
        let graph = result.expect("Failed to parse value node");
        assert_eq!(graph.nodes.len(), 1);
        match &graph.nodes[0] {
            Node::Value { value, .. } => {
                assert_eq!(value, &Value::Integer(42));
            }
            _ => panic!("Expected Value node"),
        }
    }

    #[test]
    fn parse_value_float() {
        let input = "value 42.5";
        let result = parse_geometry_nodes(input);
        assert!(result.is_ok());
        let graph = result.expect("Failed to parse float value");
        assert_eq!(graph.nodes.len(), 1);
        match &graph.nodes[0] {
            Node::Value { value, .. } => {
                assert_eq!(value, &Value::Float(42.5));
            }
            _ => panic!("Expected Value node"),
        }
    }

    #[test]
    fn parse_invalid_input() {
        let input = "invalid syntax";
        let result = parse_geometry_nodes(input);
        assert!(result.is_err());
        let errors = result.expect_err("Expected parse error");
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_invalid_vector() {
        let input = "value (1, 2)";
        let result = parse_geometry_nodes(input);
        assert!(result.is_err());
        let errors = result.expect_err("Expected parse error");
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_invalid_color() {
        let input = "value (1, 2, 3, 4, 5)";
        let result = parse_geometry_nodes(input);
        assert!(result.is_err());
        let errors = result.expect_err("Expected parse error");
        assert!(!errors.is_empty());
    }

    #[test]
    fn error_formatting() {
        let input = "invalid syntax";
        let result = parse_geometry_nodes_with_errors(input);
        assert!(result.is_err());
        let error_msg = result.expect_err("Expected parse error");
        assert!(error_msg.contains("Error"));
        assert!(error_msg.contains("<input>"));
        assert!(error_msg.contains("Found 'i' here"));
    }
}
