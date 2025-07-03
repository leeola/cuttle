use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use chumsky::span::SimpleSpan;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidNumber {
        span: SimpleSpan,
        found: String,
        expected: String,
    },
    InvalidVector {
        span: SimpleSpan,
        found_components: usize,
        expected_components: usize,
    },
    InvalidColor {
        span: SimpleSpan,
        found_components: usize,
        expected_components: usize,
    },
    UnexpectedToken {
        span: SimpleSpan,
        found: Option<char>,
        expected: Vec<String>,
    },
    UnexpectedEndOfInput {
        span: SimpleSpan,
        expected: Vec<String>,
    },
    InvalidNodeType {
        span: SimpleSpan,
        found: String,
        valid_types: Vec<String>,
    },
    MissingRequiredField {
        span: SimpleSpan,
        field: String,
        node_type: String,
    },
    InvalidFieldValue {
        span: SimpleSpan,
        field: String,
        found: String,
        expected: String,
    },
}

impl ParseError {
    pub fn span(&self) -> SimpleSpan {
        match self {
            ParseError::InvalidNumber { span, .. }
            | ParseError::InvalidVector { span, .. }
            | ParseError::InvalidColor { span, .. }
            | ParseError::UnexpectedToken { span, .. }
            | ParseError::UnexpectedEndOfInput { span, .. }
            | ParseError::InvalidNodeType { span, .. }
            | ParseError::MissingRequiredField { span, .. }
            | ParseError::InvalidFieldValue { span, .. } => *span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ParseError::InvalidNumber { expected, .. } => {
                format!("Invalid number format, expected {expected}")
            }
            ParseError::InvalidVector {
                found_components,
                expected_components,
                ..
            } => {
                format!(
                    "Invalid vector: found {found_components} components, expected {expected_components}"
                )
            }
            ParseError::InvalidColor {
                found_components,
                expected_components,
                ..
            } => {
                format!(
                    "Invalid color: found {found_components} components, expected {expected_components}"
                )
            }
            ParseError::UnexpectedToken { expected, .. } => {
                if expected.is_empty() {
                    "Unexpected token".to_string()
                } else {
                    format!("Expected one of: {}", expected.join(", "))
                }
            }
            ParseError::UnexpectedEndOfInput { expected, .. } => {
                if expected.is_empty() {
                    "Unexpected end of input".to_string()
                } else {
                    format!("Unexpected end of input, expected: {}", expected.join(", "))
                }
            }
            ParseError::InvalidNodeType {
                found, valid_types, ..
            } => {
                format!(
                    "Invalid node type '{found}', valid types: {}",
                    valid_types.join(", ")
                )
            }
            ParseError::MissingRequiredField {
                field, node_type, ..
            } => {
                format!("Missing required field '{field}' for {node_type} node")
            }
            ParseError::InvalidFieldValue {
                field,
                found,
                expected,
                ..
            } => {
                format!("Invalid value '{found}' for field '{field}', expected {expected}")
            }
        }
    }

    pub fn label_message(&self) -> String {
        match self {
            ParseError::InvalidNumber { found, .. } => {
                format!("'{found}' is not a valid number")
            }
            ParseError::InvalidVector {
                found_components, ..
            } => {
                format!("Vector has {found_components} components")
            }
            ParseError::InvalidColor {
                found_components, ..
            } => {
                format!("Color has {found_components} components")
            }
            ParseError::UnexpectedToken { found, .. } => match found {
                Some(ch) => format!("Found '{ch}' here"),
                None => "Found end of input here".to_string(),
            },
            ParseError::UnexpectedEndOfInput { .. } => "Input ended here".to_string(),
            ParseError::InvalidNodeType { found, .. } => {
                format!("'{found}' is not a valid node type")
            }
            ParseError::MissingRequiredField { field, .. } => {
                format!("'{field}' field is required")
            }
            ParseError::InvalidFieldValue { found, .. } => {
                format!("'{found}' is not valid here")
            }
        }
    }

    pub fn from_rich(rich_error: Rich<'_, char>) -> Self {
        let span = *rich_error.span();
        let found = rich_error.found().copied();
        let expected = rich_error
            .expected()
            .map(|exp| match exp {
                chumsky::error::RichPattern::Token(ch) => format!("'{ch:?}'"),
                chumsky::error::RichPattern::Label(label) => label.to_string(),
                chumsky::error::RichPattern::EndOfInput => "end of input".to_string(),
                chumsky::error::RichPattern::Identifier(id) => format!("identifier '{id}'"),
                chumsky::error::RichPattern::Any => "any character".to_string(),
                chumsky::error::RichPattern::SomethingElse => "something else".to_string(),
            })
            .collect::<Vec<_>>();

        if expected.is_empty() {
            ParseError::UnexpectedToken {
                span,
                found,
                expected,
            }
        } else {
            match found {
                Some(_) => ParseError::UnexpectedToken {
                    span,
                    found,
                    expected,
                },
                None => ParseError::UnexpectedEndOfInput { span, expected },
            }
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ParseError {}

pub struct ErrorReporter {
    color_generator: ColorGenerator,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            color_generator: ColorGenerator::new(),
        }
    }

    pub fn report_error(&mut self, error: &ParseError, source: &str, filename: &str) -> String {
        let mut output = Vec::new();
        let color = self.color_generator.next();

        let span = error.span();
        let report = Report::build(ReportKind::Error, filename, span.start)
            .with_message(error.message())
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(error.label_message())
                    .with_color(color),
            );

        let report = match error {
            ParseError::InvalidVector {
                expected_components,
                ..
            } => report.with_help(format!(
                "Vectors must have exactly {expected_components} components: (x, y, z)"
            )),
            ParseError::InvalidColor {
                expected_components,
                ..
            } => report.with_help(format!(
                "Colors must have exactly {expected_components} components: (r, g, b, a)"
            )),
            ParseError::InvalidNodeType { valid_types, .. } => {
                report.with_help(format!("Available node types: {}", valid_types.join(", ")))
            }
            ParseError::MissingRequiredField {
                field, node_type, ..
            } => report.with_help(format!("Add the '{field}' field to your {node_type} node")),
            _ => report,
        };

        report
            .finish()
            .write((filename, Source::from(source)), &mut output)
            .expect("Failed to write error report");

        String::from_utf8(output).expect("Error report contains invalid UTF-8")
    }

    pub fn report_errors(&mut self, errors: &[ParseError], source: &str, filename: &str) -> String {
        let mut output = Vec::new();

        for error in errors {
            let color = self.color_generator.next();

            let span = error.span();
            let report = Report::build(ReportKind::Error, filename, span.start)
                .with_message(error.message())
                .with_label(
                    Label::new((filename, span.start..span.end))
                        .with_message(error.label_message())
                        .with_color(color),
                );

            let report = match error {
                ParseError::InvalidVector {
                    expected_components,
                    ..
                } => report.with_help(format!(
                    "Vectors must have exactly {expected_components} components: (x, y, z)"
                )),
                ParseError::InvalidColor {
                    expected_components,
                    ..
                } => report.with_help(format!(
                    "Colors must have exactly {expected_components} components: (r, g, b, a)"
                )),
                ParseError::InvalidNodeType { valid_types, .. } => {
                    report.with_help(format!("Available node types: {}", valid_types.join(", ")))
                }
                ParseError::MissingRequiredField {
                    field, node_type, ..
                } => report.with_help(format!("Add the '{field}' field to your {node_type} node")),
                _ => report,
            };

            report
                .finish()
                .write((filename, Source::from(source)), &mut output)
                .expect("Failed to write error report");
        }

        String::from_utf8(output).expect("Error report contains invalid UTF-8")
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

pub type ParseResult<T> = Result<T, Vec<ParseError>>;

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::span::SimpleSpan;

    #[test]
    fn parse_error_span() {
        let error = ParseError::InvalidNumber {
            span: SimpleSpan::from(5..10),
            found: "abc".to_string(),
            expected: "number".to_string(),
        };
        assert_eq!(error.span(), SimpleSpan::from(5..10));
    }

    #[test]
    fn parse_error_message() {
        let error = ParseError::InvalidNumber {
            span: SimpleSpan::from(0..3),
            found: "abc".to_string(),
            expected: "number".to_string(),
        };
        assert_eq!(error.message(), "Invalid number format, expected number");
    }

    #[test]
    fn parse_error_label_message() {
        let error = ParseError::InvalidNumber {
            span: SimpleSpan::from(0..3),
            found: "abc".to_string(),
            expected: "number".to_string(),
        };
        assert_eq!(error.label_message(), "'abc' is not a valid number");
    }

    #[test]
    fn parse_error_display() {
        let error = ParseError::InvalidVector {
            span: SimpleSpan::from(0..7),
            found_components: 2,
            expected_components: 3,
        };
        assert_eq!(
            error.to_string(),
            "Invalid vector: found 2 components, expected 3"
        );
    }

    #[test]
    fn error_reporter_creates_colored_output() {
        let mut reporter = ErrorReporter::new();
        let error = ParseError::InvalidNumber {
            span: SimpleSpan::from(6..9),
            found: "abc".to_string(),
            expected: "number".to_string(),
        };
        let source = "value abc";
        let report = reporter.report_error(&error, source, "test.txt");

        assert!(report.contains("Error"));
        assert!(report.contains("Invalid number format"));
        assert!(report.contains("test.txt"));
        assert!(report.contains("abc"));
    }

    #[test]
    fn error_reporter_handles_multiple_errors() {
        let mut reporter = ErrorReporter::new();
        let errors = vec![
            ParseError::InvalidNumber {
                span: SimpleSpan::from(6..9),
                found: "abc".to_string(),
                expected: "number".to_string(),
            },
            ParseError::InvalidVector {
                span: SimpleSpan::from(0..7),
                found_components: 2,
                expected_components: 3,
            },
        ];
        let source = "value abc";
        let report = reporter.report_errors(&errors, source, "test.txt");

        assert!(report.contains("Error"));
        assert!(report.contains("Invalid number format"));
        assert!(report.contains("Invalid vector"));
    }

    #[test]
    fn error_reporter_includes_help_messages() {
        let mut reporter = ErrorReporter::new();
        let error = ParseError::InvalidVector {
            span: SimpleSpan::from(0..7),
            found_components: 2,
            expected_components: 3,
        };
        let source = "value (1, 2)";
        let report = reporter.report_error(&error, source, "test.txt");

        assert!(report.contains("Vectors must have exactly 3 components: (x, y, z)"));
    }

    #[test]
    fn rich_error_conversion() {
        let span = SimpleSpan::from(0..5);
        let rich_error = Rich::custom(span, "test error");
        let parse_error = ParseError::from_rich(rich_error);

        match parse_error {
            ParseError::UnexpectedToken {
                span: error_span, ..
            } => {
                assert_eq!(error_span, span);
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
}
