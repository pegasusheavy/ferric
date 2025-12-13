//! Core pipe transformation types and traits.

use std::fmt;

/// Arguments passed to a pipe.
#[derive(Debug, Clone, Default)]
pub struct PipeArgs {
    args: Vec<String>,
}

impl PipeArgs {
    /// Create new empty args.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create args from a vector of strings.
    pub fn from_vec(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Get argument at index.
    pub fn get(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    /// Get argument at index or default.
    pub fn get_or<'a>(&'a self, index: usize, default: &'a str) -> &'a str {
        self.get(index).unwrap_or(default)
    }

    /// Get argument as i64.
    pub fn get_i64(&self, index: usize) -> Option<i64> {
        self.get(index).and_then(|s| s.parse().ok())
    }

    /// Get argument as f64.
    pub fn get_f64(&self, index: usize) -> Option<f64> {
        self.get(index).and_then(|s| s.parse().ok())
    }

    /// Get argument as bool.
    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.get(index).and_then(|s| s.parse().ok())
    }

    /// Get number of arguments.
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Iterate over arguments.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.args.iter().map(|s| s.as_str())
    }
}

/// Value that can be piped.
#[derive(Debug, Clone)]
pub enum PipeValue {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Null value.
    Null,
    /// Array of values.
    Array(Vec<PipeValue>),
    /// Object/Map (JSON-like).
    Object(std::collections::HashMap<String, PipeValue>),
}

impl PipeValue {
    /// Convert to string representation.
    pub fn to_string_value(&self) -> String {
        match self {
            PipeValue::String(s) => s.clone(),
            PipeValue::Int(i) => i.to_string(),
            PipeValue::Float(f) => f.to_string(),
            PipeValue::Bool(b) => b.to_string(),
            PipeValue::Null => "null".to_string(),
            PipeValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string_value()).collect();
                format!("[{}]", items.join(", "))
            }
            PipeValue::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_string_value()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }

    /// Try to get as string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PipeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as i64.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            PipeValue::Int(i) => Some(*i),
            PipeValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Try to get as f64.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PipeValue::Float(f) => Some(*f),
            PipeValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Check if null.
    pub fn is_null(&self) -> bool {
        matches!(self, PipeValue::Null)
    }
}

impl fmt::Display for PipeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_value())
    }
}

impl From<&str> for PipeValue {
    fn from(s: &str) -> Self {
        PipeValue::String(s.to_string())
    }
}

impl From<String> for PipeValue {
    fn from(s: String) -> Self {
        PipeValue::String(s)
    }
}

impl From<i64> for PipeValue {
    fn from(i: i64) -> Self {
        PipeValue::Int(i)
    }
}

impl From<i32> for PipeValue {
    fn from(i: i32) -> Self {
        PipeValue::Int(i as i64)
    }
}

impl From<f64> for PipeValue {
    fn from(f: f64) -> Self {
        PipeValue::Float(f)
    }
}

impl From<bool> for PipeValue {
    fn from(b: bool) -> Self {
        PipeValue::Bool(b)
    }
}

impl<T: Into<PipeValue>> From<Option<T>> for PipeValue {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => PipeValue::Null,
        }
    }
}

impl<T: Into<PipeValue> + Clone> From<Vec<T>> for PipeValue {
    fn from(v: Vec<T>) -> Self {
        PipeValue::Array(v.into_iter().map(|x| x.into()).collect())
    }
}

/// Trait that all pipes must implement.
pub trait Pipe: Send + Sync {
    /// The name of the pipe (used in templates).
    fn name(&self) -> &'static str;

    /// Transform a string value.
    fn transform(&self, value: &str, args: &PipeArgs) -> String;

    /// Transform a PipeValue (default implementation converts to string).
    fn transform_value(&self, value: &PipeValue, args: &PipeArgs) -> PipeValue {
        PipeValue::String(self.transform(&value.to_string_value(), args))
    }

    /// Check if this pipe is pure (same input always gives same output).
    fn is_pure(&self) -> bool {
        true
    }
}

/// Apply a single pipe to a value.
pub fn apply_pipe(value: &str, pipe_name: &str, args: &PipeArgs) -> Result<String, String> {
    use super::registry::get_pipe;

    let pipe = get_pipe(pipe_name).ok_or_else(|| format!("Unknown pipe: {}", pipe_name))?;
    Ok(pipe.transform(value, args))
}

/// Apply multiple pipes to a value (chained).
pub fn apply_pipes(value: &str, pipes: &[(String, PipeArgs)]) -> Result<String, String> {
    let mut result = value.to_string();

    for (pipe_name, args) in pipes {
        result = apply_pipe(&result, pipe_name, args)?;
    }

    Ok(result)
}

/// A parsed pipe expression (pipe name + arguments).
#[derive(Debug, Clone)]
pub struct ParsedPipe {
    /// Pipe name.
    pub name: String,
    /// Pipe arguments.
    pub args: PipeArgs,
}

/// Parse a pipe expression like "date:'short'" or "slice:0:10".
pub fn parse_pipe_expression(expr: &str) -> ParsedPipe {
    let expr = expr.trim();

    // Split on first ':'
    if let Some(colon_pos) = expr.find(':') {
        let name = expr[..colon_pos].trim().to_string();
        let args_str = &expr[colon_pos + 1..];

        // Parse arguments (colon-separated)
        let args: Vec<String> = parse_pipe_args(args_str);

        ParsedPipe {
            name,
            args: PipeArgs::from_vec(args),
        }
    } else {
        ParsedPipe {
            name: expr.to_string(),
            args: PipeArgs::new(),
        }
    }
}

/// Parse pipe arguments, handling quoted strings.
fn parse_pipe_args(args_str: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for c in args_str.chars() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if c == quote_char && in_quotes => {
                in_quotes = false;
            }
            ':' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.trim().to_string());
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current.trim().to_string());
    }

    args
}

/// Parse a full template expression with pipes.
/// Returns (base_expression, list of pipes).
pub fn parse_piped_expression(expr: &str) -> (String, Vec<ParsedPipe>) {
    let parts: Vec<&str> = expr.split('|').collect();

    if parts.is_empty() {
        return (String::new(), Vec::new());
    }

    let base = parts[0].trim().to_string();
    let pipes: Vec<ParsedPipe> = parts[1..]
        .iter()
        .map(|p| parse_pipe_expression(p))
        .collect();

    (base, pipes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pipe_expression_simple() {
        let parsed = parse_pipe_expression("uppercase");
        assert_eq!(parsed.name, "uppercase");
        assert!(parsed.args.is_empty());
    }

    #[test]
    fn test_parse_pipe_expression_with_args() {
        let parsed = parse_pipe_expression("date:'short'");
        assert_eq!(parsed.name, "date");
        assert_eq!(parsed.args.get(0), Some("short"));
    }

    #[test]
    fn test_parse_pipe_expression_multiple_args() {
        let parsed = parse_pipe_expression("slice:0:10");
        assert_eq!(parsed.name, "slice");
        assert_eq!(parsed.args.get(0), Some("0"));
        assert_eq!(parsed.args.get(1), Some("10"));
    }

    #[test]
    fn test_parse_piped_expression() {
        let (base, pipes) = parse_piped_expression("name | lowercase | slice:0:5");
        assert_eq!(base, "name");
        assert_eq!(pipes.len(), 2);
        assert_eq!(pipes[0].name, "lowercase");
        assert_eq!(pipes[1].name, "slice");
    }

    #[test]
    fn test_pipe_args() {
        let args = PipeArgs::from_vec(vec!["10".to_string(), "hello".to_string()]);
        assert_eq!(args.get_i64(0), Some(10));
        assert_eq!(args.get(1), Some("hello"));
        assert_eq!(args.get(2), None);
    }
}
