//! Expression evaluator for template bindings.
//!
//! Supports a subset of JavaScript-like expressions for template bindings.

use super::context::TemplateContext;

/// Result of evaluating an expression.
#[derive(Debug, Clone)]
pub enum ExprValue {
    /// String value.
    String(String),
    /// Number value (f64 for simplicity).
    Number(f64),
    /// Boolean value.
    Bool(bool),
    /// Null/undefined value.
    Null,
    /// Array of values.
    Array(Vec<ExprValue>),
}

impl ExprValue {
    /// Convert to string.
    pub fn to_string(&self) -> String {
        match self {
            ExprValue::String(s) => s.clone(),
            ExprValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            ExprValue::Bool(b) => b.to_string(),
            ExprValue::Null => String::new(),
            ExprValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                items.join(", ")
            }
        }
    }

    /// Convert to boolean (truthy/falsy).
    pub fn to_bool(&self) -> bool {
        match self {
            ExprValue::String(s) => !s.is_empty(),
            ExprValue::Number(n) => *n != 0.0,
            ExprValue::Bool(b) => *b,
            ExprValue::Null => false,
            ExprValue::Array(arr) => !arr.is_empty(),
        }
    }

    /// Convert to number.
    pub fn to_number(&self) -> f64 {
        match self {
            ExprValue::String(s) => s.parse().unwrap_or(0.0),
            ExprValue::Number(n) => *n,
            ExprValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            ExprValue::Null => 0.0,
            ExprValue::Array(_) => f64::NAN,
        }
    }
}

impl From<String> for ExprValue {
    fn from(s: String) -> Self {
        ExprValue::String(s)
    }
}

impl From<&str> for ExprValue {
    fn from(s: &str) -> Self {
        ExprValue::String(s.to_string())
    }
}

impl From<i32> for ExprValue {
    fn from(n: i32) -> Self {
        ExprValue::Number(n as f64)
    }
}

impl From<f64> for ExprValue {
    fn from(n: f64) -> Self {
        ExprValue::Number(n)
    }
}

impl From<bool> for ExprValue {
    fn from(b: bool) -> Self {
        ExprValue::Bool(b)
    }
}

/// Expression evaluator.
pub struct ExpressionEvaluator<'a> {
    context: &'a TemplateContext,
}

impl<'a> ExpressionEvaluator<'a> {
    /// Create a new evaluator with a context.
    pub fn new(context: &'a TemplateContext) -> Self {
        Self { context }
    }

    /// Evaluate an expression string.
    pub fn evaluate(&self, expr: &str) -> ExprValue {
        let expr = expr.trim();

        // Handle empty expression
        if expr.is_empty() {
            return ExprValue::Null;
        }

        // Try parsing as a literal first
        if let Some(val) = self.parse_literal(expr) {
            return val;
        }

        // Handle ternary operator
        if let Some(val) = self.evaluate_ternary(expr) {
            return val;
        }

        // Handle logical operators
        if let Some(val) = self.evaluate_logical(expr) {
            return val;
        }

        // Handle comparison operators
        if let Some(val) = self.evaluate_comparison(expr) {
            return val;
        }

        // Handle arithmetic
        if let Some(val) = self.evaluate_arithmetic(expr) {
            return val;
        }

        // Handle negation
        if expr.starts_with('!') {
            let inner = self.evaluate(&expr[1..]);
            return ExprValue::Bool(!inner.to_bool());
        }

        // Handle property access (e.g., "user.name")
        self.evaluate_property_access(expr)
    }

    /// Parse a literal value.
    fn parse_literal(&self, expr: &str) -> Option<ExprValue> {
        // String literals
        if (expr.starts_with('"') && expr.ends_with('"')) ||
           (expr.starts_with('\'') && expr.ends_with('\'')) {
            return Some(ExprValue::String(expr[1..expr.len()-1].to_string()));
        }

        // Boolean literals
        if expr == "true" {
            return Some(ExprValue::Bool(true));
        }
        if expr == "false" {
            return Some(ExprValue::Bool(false));
        }

        // Null/undefined
        if expr == "null" || expr == "undefined" {
            return Some(ExprValue::Null);
        }

        // Number literals
        if let Ok(n) = expr.parse::<f64>() {
            return Some(ExprValue::Number(n));
        }

        None
    }

    /// Evaluate a ternary expression.
    fn evaluate_ternary(&self, expr: &str) -> Option<ExprValue> {
        // Find the ? outside of strings and parentheses
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut question_pos = None;

        for (i, c) in expr.chars().enumerate() {
            if in_string {
                if c == string_char {
                    in_string = false;
                }
                continue;
            }

            match c {
                '"' | '\'' => {
                    in_string = true;
                    string_char = c;
                }
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                '?' if depth == 0 => {
                    question_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        let question_pos = question_pos?;

        // Find the colon
        let rest = &expr[question_pos + 1..];
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut colon_pos = None;

        for (i, c) in rest.chars().enumerate() {
            if in_string {
                if c == string_char {
                    in_string = false;
                }
                continue;
            }

            match c {
                '"' | '\'' => {
                    in_string = true;
                    string_char = c;
                }
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                ':' if depth == 0 => {
                    colon_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        let colon_pos = colon_pos?;

        let condition = &expr[..question_pos];
        let true_expr = &rest[..colon_pos];
        let false_expr = &rest[colon_pos + 1..];

        let cond_val = self.evaluate(condition);
        if cond_val.to_bool() {
            Some(self.evaluate(true_expr))
        } else {
            Some(self.evaluate(false_expr))
        }
    }

    /// Evaluate logical operators (&&, ||).
    fn evaluate_logical(&self, expr: &str) -> Option<ExprValue> {
        // Find && or || outside strings and parentheses
        let ops = ["||", "&&"];

        for op in ops {
            if let Some((left, right)) = self.split_at_operator(expr, op) {
                let left_val = self.evaluate(left);

                match op {
                    "&&" => {
                        if !left_val.to_bool() {
                            return Some(left_val);
                        }
                        return Some(self.evaluate(right));
                    }
                    "||" => {
                        if left_val.to_bool() {
                            return Some(left_val);
                        }
                        return Some(self.evaluate(right));
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Evaluate comparison operators.
    fn evaluate_comparison(&self, expr: &str) -> Option<ExprValue> {
        let ops = ["===", "!==", "==", "!=", "<=", ">=", "<", ">"];

        for op in ops {
            if let Some((left, right)) = self.split_at_operator(expr, op) {
                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);

                let result = match op {
                    "===" | "==" => self.strict_equals(&left_val, &right_val),
                    "!==" | "!=" => !self.strict_equals(&left_val, &right_val),
                    "<" => left_val.to_number() < right_val.to_number(),
                    ">" => left_val.to_number() > right_val.to_number(),
                    "<=" => left_val.to_number() <= right_val.to_number(),
                    ">=" => left_val.to_number() >= right_val.to_number(),
                    _ => false,
                };

                return Some(ExprValue::Bool(result));
            }
        }

        None
    }

    /// Evaluate arithmetic operators.
    fn evaluate_arithmetic(&self, expr: &str) -> Option<ExprValue> {
        // Handle +, -, *, /, % with proper precedence
        // First try + and - (lower precedence)
        for op in ["+", "-"] {
            if let Some((left, right)) = self.split_at_operator_rtl(expr, op) {
                // Don't split if this is a unary minus
                if left.is_empty() && op == "-" {
                    continue;
                }

                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);

                // String concatenation for +
                if op == "+"
                    && (matches!(&left_val, ExprValue::String(_)) ||
                       matches!(&right_val, ExprValue::String(_))) {
                        return Some(ExprValue::String(
                            format!("{}{}", left_val.to_string(), right_val.to_string())
                        ));
                    }

                let result = match op {
                    "+" => left_val.to_number() + right_val.to_number(),
                    "-" => left_val.to_number() - right_val.to_number(),
                    _ => return None,
                };

                return Some(ExprValue::Number(result));
            }
        }

        // Then try *, /, % (higher precedence)
        for op in ["*", "/", "%"] {
            if let Some((left, right)) = self.split_at_operator_rtl(expr, op) {
                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);

                let result = match op {
                    "*" => left_val.to_number() * right_val.to_number(),
                    "/" => left_val.to_number() / right_val.to_number(),
                    "%" => left_val.to_number() % right_val.to_number(),
                    _ => return None,
                };

                return Some(ExprValue::Number(result));
            }
        }

        None
    }

    /// Evaluate property access (e.g., "user.name", "items[0]").
    fn evaluate_property_access(&self, expr: &str) -> ExprValue {
        let expr = expr.trim();

        // Handle method calls (e.g., "items.length")
        if let Some(dot_pos) = expr.find('.') {
            let obj_expr = &expr[..dot_pos];
            let prop = &expr[dot_pos + 1..];

            // Special case for built-in properties
            let obj_val = self.evaluate(obj_expr);
            if prop == "length" {
                match &obj_val {
                    ExprValue::String(s) => return ExprValue::Number(s.len() as f64),
                    ExprValue::Array(arr) => return ExprValue::Number(arr.len() as f64),
                    _ => {}
                }
            }
        }

        // Handle array access (e.g., "items[0]")
        if let Some(bracket_pos) = expr.find('[')
            && expr.ends_with(']') {
                let _array_expr = &expr[..bracket_pos];
                let _index_expr = &expr[bracket_pos + 1..expr.len() - 1];
                // TODO: Implement array indexing
            }

        // Simple variable lookup - try to preserve the original type
        if let Some(ctx_ref) = self.context.get(expr) {
            // Try to get as specific types first
            if let Some(b) = ctx_ref.get::<bool>() {
                return ExprValue::Bool(b);
            }
            if let Some(n) = ctx_ref.get::<i32>() {
                return ExprValue::Number(n as f64);
            }
            if let Some(n) = ctx_ref.get::<i64>() {
                return ExprValue::Number(n as f64);
            }
            if let Some(n) = ctx_ref.get::<f64>() {
                return ExprValue::Number(n);
            }
            if let Some(n) = ctx_ref.get::<f32>() {
                return ExprValue::Number(n as f64);
            }
            if let Some(s) = ctx_ref.get::<String>() {
                return ExprValue::String(s);
            }
            // Fallback to string representation
            return ExprValue::String(ctx_ref.to_string());
        }

        ExprValue::Null
    }

    /// Split expression at an operator (left to right).
    fn split_at_operator<'b>(&self, expr: &'b str, op: &str) -> Option<(&'b str, &'b str)> {
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let op_len = op.len();
        let chars: Vec<char> = expr.chars().collect();

        for i in 0..chars.len() {
            if in_string {
                if chars[i] == string_char {
                    in_string = false;
                }
                continue;
            }

            match chars[i] {
                '"' | '\'' => {
                    in_string = true;
                    string_char = chars[i];
                }
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ if depth == 0 => {
                    // Check if we match the operator
                    if i + op_len <= chars.len() {
                        let slice: String = chars[i..i + op_len].iter().collect();
                        if slice == op {
                            let left = &expr[..i];
                            let right = &expr[i + op_len..];
                            if !left.is_empty() && !right.is_empty() {
                                return Some((left.trim(), right.trim()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Split expression at an operator (right to left for proper precedence).
    fn split_at_operator_rtl<'b>(&self, expr: &'b str, op: &str) -> Option<(&'b str, &'b str)> {
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let op_len = op.len();
        let chars: Vec<char> = expr.chars().collect();
        let mut last_match: Option<usize> = None;

        for i in 0..chars.len() {
            if in_string {
                if chars[i] == string_char {
                    in_string = false;
                }
                continue;
            }

            match chars[i] {
                '"' | '\'' => {
                    in_string = true;
                    string_char = chars[i];
                }
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ if depth == 0 => {
                    if i + op_len <= chars.len() {
                        let slice: String = chars[i..i + op_len].iter().collect();
                        if slice == op {
                            last_match = Some(i);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(i) = last_match {
            let left = &expr[..i];
            let right = &expr[i + op_len..];
            if !left.is_empty() && !right.is_empty() {
                return Some((left.trim(), right.trim()));
            }
        }

        None
    }

    /// Check strict equality.
    fn strict_equals(&self, left: &ExprValue, right: &ExprValue) -> bool {
        match (left, right) {
            (ExprValue::String(a), ExprValue::String(b)) => a == b,
            (ExprValue::Number(a), ExprValue::Number(b)) => (a - b).abs() < f64::EPSILON,
            (ExprValue::Bool(a), ExprValue::Bool(b)) => a == b,
            (ExprValue::Null, ExprValue::Null) => true,
            _ => false,
        }
    }
}

/// Evaluate an expression with a context.
pub fn evaluate(expr: &str, context: &TemplateContext) -> ExprValue {
    ExpressionEvaluator::new(context).evaluate(expr)
}

/// Evaluate an expression and return as string.
pub fn evaluate_to_string(expr: &str, context: &TemplateContext) -> String {
    evaluate(expr, context).to_string()
}

/// Evaluate an expression and return as boolean.
pub fn evaluate_to_bool(expr: &str, context: &TemplateContext) -> bool {
    evaluate(expr, context).to_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literals() {
        let ctx = TemplateContext::new();

        assert_eq!(evaluate("true", &ctx).to_bool(), true);
        assert_eq!(evaluate("false", &ctx).to_bool(), false);
        assert_eq!(evaluate("42", &ctx).to_number(), 42.0);
        assert_eq!(evaluate("'hello'", &ctx).to_string(), "hello");
        assert_eq!(evaluate("\"world\"", &ctx).to_string(), "world");
    }

    #[test]
    fn test_variables() {
        let ctx = TemplateContext::new();
        ctx.set("name", "Ferric".to_string());
        ctx.set("count", 10i32);

        assert_eq!(evaluate("name", &ctx).to_string(), "Ferric");
        assert_eq!(evaluate("count", &ctx).to_number(), 10.0);
    }

    #[test]
    fn test_comparison() {
        let ctx = TemplateContext::new();
        ctx.set("x", 10i32);

        assert_eq!(evaluate("x > 5", &ctx).to_bool(), true);
        assert_eq!(evaluate("x < 5", &ctx).to_bool(), false);
        assert_eq!(evaluate("x == 10", &ctx).to_bool(), true);
    }

    #[test]
    fn test_ternary() {
        let ctx = TemplateContext::new();
        ctx.set("isActive", true);

        assert_eq!(evaluate("isActive ? 'yes' : 'no'", &ctx).to_string(), "yes");
    }

    #[test]
    fn test_logical() {
        let ctx = TemplateContext::new();
        ctx.set("a", true);
        ctx.set("b", false);

        assert_eq!(evaluate("a && b", &ctx).to_bool(), false);
        assert_eq!(evaluate("a || b", &ctx).to_bool(), true);
        assert_eq!(evaluate("!b", &ctx).to_bool(), true);
    }
}
