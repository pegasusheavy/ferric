//! Template parser for converting HTML strings into an AST.

use super::{Binding, INTERPOLATION_START, INTERPOLATION_END};

/// A node in the template AST.
#[derive(Debug, Clone)]
pub enum TemplateNode {
    /// An HTML element.
    Element(ElementNode),
    /// A text node.
    Text(TextNode),
    /// An interpolation expression.
    Interpolation(InterpolationNode),
    /// A component reference.
    Component(ComponentNode),
}

/// An HTML element node.
#[derive(Debug, Clone)]
pub struct ElementNode {
    /// The tag name.
    pub tag: String,
    /// Static attributes.
    pub attributes: Vec<(String, String)>,
    /// Data bindings.
    pub bindings: Vec<Binding>,
    /// Child nodes.
    pub children: Vec<TemplateNode>,
    /// Template reference variable (e.g., #myRef).
    pub reference: Option<String>,
    /// Structural directive (e.g., *ngIf, *ngFor).
    pub structural_directive: Option<StructuralDirective>,
}

/// A text node.
#[derive(Debug, Clone)]
pub struct TextNode {
    /// The text content.
    pub content: String,
}

/// An interpolation expression node.
#[derive(Debug, Clone)]
pub struct InterpolationNode {
    /// The expression to evaluate.
    pub expression: String,
}

/// A component node.
#[derive(Debug, Clone)]
pub struct ComponentNode {
    /// The component selector.
    pub selector: String,
    /// Input bindings.
    pub inputs: Vec<Binding>,
    /// Output bindings.
    pub outputs: Vec<Binding>,
    /// Child content (for content projection).
    pub children: Vec<TemplateNode>,
    /// Template reference variable.
    pub reference: Option<String>,
}

/// A structural directive.
#[derive(Debug, Clone)]
pub struct StructuralDirective {
    /// The directive name (e.g., "if", "for").
    pub name: String,
    /// The directive expression.
    pub expression: String,
}

/// Parse a template string into an AST.
pub fn parse_template(template: &str) -> Result<Vec<TemplateNode>, ParseError> {
    let mut parser = TemplateParser::new(template);
    parser.parse()
}

/// Template parser state.
struct TemplateParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> TemplateParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse(&mut self) -> Result<Vec<TemplateNode>, ParseError> {
        let mut nodes = Vec::new();

        while self.position < self.input.len() {
            if self.peek_str(INTERPOLATION_START) {
                nodes.push(self.parse_interpolation()?);
            } else if self.peek_char() == Some('<') {
                if self.peek_str("</") {
                    break; // End tag - return to parent
                }
                nodes.push(self.parse_element()?);
            } else {
                nodes.push(self.parse_text()?);
            }
        }

        Ok(nodes)
    }

    fn parse_interpolation(&mut self) -> Result<TemplateNode, ParseError> {
        self.expect_str(INTERPOLATION_START)?;
        self.skip_whitespace();

        let start = self.position;
        while !self.peek_str(INTERPOLATION_END) && self.position < self.input.len() {
            self.position += 1;
        }
        let expression = self.input[start..self.position].trim().to_string();

        self.expect_str(INTERPOLATION_END)?;

        Ok(TemplateNode::Interpolation(InterpolationNode { expression }))
    }

    fn parse_element(&mut self) -> Result<TemplateNode, ParseError> {
        self.expect_char('<')?;
        let tag = self.parse_identifier()?;

        let mut attributes = Vec::new();
        let mut bindings = Vec::new();
        let mut reference = None;
        let mut structural_directive = None;

        self.skip_whitespace();

        // Parse attributes and bindings
        while self.peek_char() != Some('>') && !self.peek_str("/>") {
            self.skip_whitespace();

            if let Some(c) = self.peek_char() {
                match c {
                    '#' => {
                        self.position += 1;
                        reference = Some(self.parse_identifier()?);
                    }
                    '*' => {
                        self.position += 1;
                        let name = self.parse_identifier()?;
                        self.expect_char('=')?;
                        self.expect_char('"')?;
                        let expr = self.parse_until('"')?;
                        self.expect_char('"')?;
                        structural_directive = Some(StructuralDirective {
                            name,
                            expression: expr,
                        });
                    }
                    '[' => {
                        bindings.push(self.parse_property_binding()?);
                    }
                    '(' => {
                        bindings.push(self.parse_event_binding()?);
                    }
                    _ => {
                        let (key, value) = self.parse_attribute()?;
                        attributes.push((key, value));
                    }
                }
            }

            self.skip_whitespace();
        }

        // Check for self-closing tag
        let is_self_closing = self.peek_str("/>");
        if is_self_closing {
            self.position += 2;
            return Ok(TemplateNode::Element(ElementNode {
                tag,
                attributes,
                bindings,
                children: Vec::new(),
                reference,
                structural_directive,
            }));
        }

        self.expect_char('>')?;

        // Parse children
        let children = self.parse()?;

        // Parse closing tag
        self.expect_str("</")?;
        let closing_tag = self.parse_identifier()?;
        if closing_tag != tag {
            return Err(ParseError::MismatchedTags(tag, closing_tag));
        }
        self.expect_char('>')?;

        Ok(TemplateNode::Element(ElementNode {
            tag,
            attributes,
            bindings,
            children,
            reference,
            structural_directive,
        }))
    }

    fn parse_property_binding(&mut self) -> Result<Binding, ParseError> {
        self.expect_char('[')?;

        // Check for two-way binding
        let is_two_way = self.peek_char() == Some('(');
        if is_two_way {
            self.expect_char('(')?;
        }

        let name = self.parse_identifier()?;

        if is_two_way {
            self.expect_char(')')?;
        }
        self.expect_char(']')?;
        self.expect_char('=')?;
        self.expect_char('"')?;
        let expression = self.parse_until('"')?;
        self.expect_char('"')?;

        if is_two_way {
            Ok(Binding::two_way(&name, &expression))
        } else {
            Ok(Binding::property(&name, &expression))
        }
    }

    fn parse_event_binding(&mut self) -> Result<Binding, ParseError> {
        self.expect_char('(')?;
        let name = self.parse_identifier()?;
        self.expect_char(')')?;
        self.expect_char('=')?;
        self.expect_char('"')?;
        let handler = self.parse_until('"')?;
        self.expect_char('"')?;

        Ok(Binding::event(&name, &handler))
    }

    fn parse_attribute(&mut self) -> Result<(String, String), ParseError> {
        let name = self.parse_identifier()?;

        if self.peek_char() == Some('=') {
            self.position += 1;
            self.expect_char('"')?;
            let value = self.parse_until('"')?;
            self.expect_char('"')?;
            Ok((name, value))
        } else {
            Ok((name, String::new()))
        }
    }

    fn parse_text(&mut self) -> Result<TemplateNode, ParseError> {
        let start = self.position;

        while self.position < self.input.len() {
            if self.peek_char() == Some('<') || self.peek_str(INTERPOLATION_START) {
                break;
            }
            self.position += 1;
        }

        let content = self.input[start..self.position].to_string();
        Ok(TemplateNode::Text(TextNode { content }))
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.position;

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                self.position += 1;
            } else {
                break;
            }
        }

        if start == self.position {
            return Err(ParseError::ExpectedIdentifier);
        }

        Ok(self.input[start..self.position].to_string())
    }

    fn parse_until(&mut self, end: char) -> Result<String, ParseError> {
        let start = self.position;

        while self.peek_char() != Some(end) && self.position < self.input.len() {
            self.position += 1;
        }

        Ok(self.input[start..self.position].to_string())
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn peek_str(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        if self.peek_char() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(ParseError::ExpectedChar(expected))
        }
    }

    fn expect_str(&mut self, expected: &str) -> Result<(), ParseError> {
        if self.peek_str(expected) {
            self.position += expected.len();
            Ok(())
        } else {
            Err(ParseError::ExpectedStr(expected.to_string()))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }
}

/// Errors that can occur during template parsing.
#[derive(Debug, Clone)]
pub enum ParseError {
    ExpectedChar(char),
    ExpectedStr(String),
    ExpectedIdentifier,
    MismatchedTags(String, String),
    UnexpectedEof,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedChar(c) => write!(f, "Expected character: '{}'", c),
            Self::ExpectedStr(s) => write!(f, "Expected string: '{}'", s),
            Self::ExpectedIdentifier => write!(f, "Expected identifier"),
            Self::MismatchedTags(open, close) => {
                write!(f, "Mismatched tags: <{}> and </{}>", open, close)
            }
            Self::UnexpectedEof => write!(f, "Unexpected end of input"),
        }
    }
}

impl std::error::Error for ParseError {}

