//! Switch directive implementation.
//!
//! Provides `[feSwitch]`, `*feSwitchCase`, and `*feSwitchDefault` directives
//! for conditional rendering based on a switch expression.

/// A value that can be matched in a switch statement.
#[derive(Debug, Clone, PartialEq)]
pub enum SwitchValue {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Null/None value.
    Null,
}

impl SwitchValue {
    /// Create from a string.
    pub fn string(s: impl Into<String>) -> Self {
        SwitchValue::String(s.into())
    }

    /// Create from an integer.
    pub fn int(i: i64) -> Self {
        SwitchValue::Int(i)
    }

    /// Create from a float.
    pub fn float(f: f64) -> Self {
        SwitchValue::Float(f)
    }

    /// Create from a boolean.
    pub fn bool(b: bool) -> Self {
        SwitchValue::Bool(b)
    }

    /// Create a null value.
    pub fn null() -> Self {
        SwitchValue::Null
    }

    /// Parse from a string representation.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();

        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            return SwitchValue::String(s[1..s.len() - 1].to_string());
        }

        if s == "true" {
            return SwitchValue::Bool(true);
        }
        if s == "false" {
            return SwitchValue::Bool(false);
        }

        if s == "null" || s == "undefined" {
            return SwitchValue::Null;
        }

        if let Ok(i) = s.parse::<i64>() {
            return SwitchValue::Int(i);
        }

        if let Ok(f) = s.parse::<f64>() {
            return SwitchValue::Float(f);
        }

        SwitchValue::String(s.to_string())
    }
}

impl From<&str> for SwitchValue {
    fn from(s: &str) -> Self {
        SwitchValue::String(s.to_string())
    }
}

impl From<String> for SwitchValue {
    fn from(s: String) -> Self {
        SwitchValue::String(s)
    }
}

impl From<i32> for SwitchValue {
    fn from(i: i32) -> Self {
        SwitchValue::Int(i as i64)
    }
}

impl From<i64> for SwitchValue {
    fn from(i: i64) -> Self {
        SwitchValue::Int(i)
    }
}

impl From<f64> for SwitchValue {
    fn from(f: f64) -> Self {
        SwitchValue::Float(f)
    }
}

impl From<bool> for SwitchValue {
    fn from(b: bool) -> Self {
        SwitchValue::Bool(b)
    }
}

impl<T> From<Option<T>> for SwitchValue
where
    T: Into<SwitchValue>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => SwitchValue::Null,
        }
    }
}

/// Represents a switch case with its value and template.
#[derive(Debug, Clone)]
pub struct SwitchCase {
    /// The value(s) this case matches.
    pub values: Vec<SwitchValue>,
    /// The template HTML to render.
    pub template: String,
    /// Whether this is the default case.
    pub is_default: bool,
}

impl SwitchCase {
    /// Create a new case with a single value.
    pub fn new(value: impl Into<SwitchValue>, template: impl Into<String>) -> Self {
        Self {
            values: vec![value.into()],
            template: template.into(),
            is_default: false,
        }
    }

    /// Create a case matching multiple values.
    pub fn multi(values: Vec<SwitchValue>, template: impl Into<String>) -> Self {
        Self {
            values,
            template: template.into(),
            is_default: false,
        }
    }

    /// Create the default case.
    pub fn default(template: impl Into<String>) -> Self {
        Self {
            values: Vec::new(),
            template: template.into(),
            is_default: true,
        }
    }

    /// Check if this case matches a value.
    pub fn matches(&self, value: &SwitchValue) -> bool {
        if self.is_default {
            return false;
        }
        self.values.iter().any(|v| v == value)
    }
}

/// The main switch container that manages cases and rendering.
#[derive(Debug, Clone)]
pub struct FeSwitchContainer {
    /// Current switch expression value.
    value: Option<SwitchValue>,
    /// Registered cases.
    cases: Vec<SwitchCase>,
    /// Currently active case index.
    active_case: Option<usize>,
}

impl FeSwitchContainer {
    /// Create a new switch container.
    pub fn new() -> Self {
        Self {
            value: None,
            cases: Vec::new(),
            active_case: None,
        }
    }

    /// Set the switch expression value.
    pub fn set_value(&mut self, value: impl Into<SwitchValue>) {
        self.value = Some(value.into());
        self.update_active_case();
    }

    /// Clear the switch value.
    pub fn clear_value(&mut self) {
        self.value = None;
        self.active_case = None;
    }

    /// Add a case.
    pub fn add_case(&mut self, case: SwitchCase) {
        self.cases.push(case);
        if self.value.is_some() {
            self.update_active_case();
        }
    }

    /// Get all cases.
    pub fn cases(&self) -> &[SwitchCase] {
        &self.cases
    }

    /// Get the currently active case.
    pub fn active_case(&self) -> Option<&SwitchCase> {
        self.active_case.and_then(|i| self.cases.get(i))
    }

    /// Get the rendered content for the active case.
    pub fn render(&self) -> Option<&str> {
        self.active_case().map(|c| c.template.as_str())
    }

    /// Update which case is active based on current value.
    fn update_active_case(&mut self) {
        self.active_case = None;

        if let Some(ref value) = self.value {
            for (i, case) in self.cases.iter().enumerate() {
                if case.matches(value) {
                    self.active_case = Some(i);
                    return;
                }
            }

            for (i, case) in self.cases.iter().enumerate() {
                if case.is_default {
                    self.active_case = Some(i);
                    return;
                }
            }
        }
    }

    /// Check if any case is active.
    pub fn has_active_case(&self) -> bool {
        self.active_case.is_some()
    }
}

impl Default for FeSwitchContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed switch block.
#[derive(Debug, Clone)]
pub struct SwitchBlock {
    /// The switch expression.
    pub expression: String,
    /// The cases in this switch.
    pub cases: Vec<ParsedCase>,
    /// Start position in template.
    pub start: usize,
    /// End position in template.
    pub end: usize,
}

/// A parsed case.
#[derive(Debug, Clone)]
pub struct ParsedCase {
    /// The case value (None for default).
    pub value: Option<String>,
    /// The template content.
    pub template: String,
    /// Whether this is the default case.
    pub is_default: bool,
}

/// Result of parsing switch directives.
#[derive(Debug, Clone, Default)]
pub struct ParsedSwitch {
    /// All switch blocks found.
    pub switches: Vec<SwitchBlock>,
}

impl ParsedSwitch {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any switches were found.
    pub fn has_switches(&self) -> bool {
        !self.switches.is_empty()
    }
}

/// Parse switch directives from a template.
pub fn parse_switch_template(template: &str) -> ParsedSwitch {
    let mut result = ParsedSwitch::new();

    let mut search_start = 0;
    while let Some(switch_start) = template[search_start..].find("[feSwitch]") {
        let abs_start = search_start + switch_start;

        let element_start = template[..abs_start].rfind('<').unwrap_or(0);
        let element_end = find_element_end(&template[element_start..])
            .map(|e| element_start + e)
            .unwrap_or(template.len());

        if let Some(expr) = extract_attribute_value(&template[abs_start..], "[feSwitch]") {
            let switch_content = &template[element_start..element_end];
            let cases = parse_switch_cases(switch_content);

            result.switches.push(SwitchBlock {
                expression: expr,
                cases,
                start: element_start,
                end: element_end,
            });
        }

        search_start = element_end;
    }

    result
}

/// Parse switch cases from within a switch container.
fn parse_switch_cases(content: &str) -> Vec<ParsedCase> {
    let mut cases = Vec::new();

    let mut search_start = 0;
    while let Some(case_start) = content[search_start..].find("*feSwitchCase") {
        let abs_start = search_start + case_start;

        if let Some(value) = extract_attribute_value(&content[abs_start..], "*feSwitchCase") {
            let element_start = content[..abs_start].rfind('<').unwrap_or(0);
            if let Some((tag_end, content_end)) = find_element_content(&content[element_start..]) {
                let template = content[element_start + tag_end..element_start + content_end].to_string();

                cases.push(ParsedCase {
                    value: Some(value),
                    template,
                    is_default: false,
                });
            }
        }

        search_start = abs_start + 13;
    }

    if let Some(default_start) = content.find("*feSwitchDefault") {
        let element_start = content[..default_start].rfind('<').unwrap_or(0);
        if let Some((tag_end, content_end)) = find_element_content(&content[element_start..]) {
            let template = content[element_start + tag_end..element_start + content_end].to_string();

            cases.push(ParsedCase {
                value: None,
                template,
                is_default: true,
            });
        }
    }

    cases
}

/// Extract an attribute value from HTML.
fn extract_attribute_value(html: &str, attr: &str) -> Option<String> {
    let attr_start = html.find(attr)?;
    let after_attr = &html[attr_start + attr.len()..];

    let trimmed = after_attr.trim_start();
    if !trimmed.starts_with('=') {
        return None;
    }
    let after_eq = trimmed[1..].trim_start();

    let (quote, start) = if after_eq.starts_with('"') {
        ('"', 1)
    } else if after_eq.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };

    let value_start = start;
    let value_end = after_eq[value_start..].find(quote)?;

    Some(after_eq[value_start..value_start + value_end].to_string())
}

/// Find the end of an element (including nested elements).
fn find_element_end(html: &str) -> Option<usize> {
    if !html.starts_with('<') {
        return None;
    }

    let tag_end = html[1..].find(|c: char| c.is_whitespace() || c == '>' || c == '/')?;
    let tag_name = &html[1..1 + tag_end];

    if let Some(close) = html.find("/>")
        && html[..close].find('>').is_none_or(|g| g > close) {
            return Some(close + 2);
        }

    let open_end = html.find('>')?;

    let close_tag = format!("</{}>", tag_name);
    let mut depth = 1;
    let mut search_pos = open_end + 1;

    while depth > 0 && search_pos < html.len() {
        let next_open = html[search_pos..].find(&format!("<{}", tag_name));
        let next_close = html[search_pos..].find(&close_tag);

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                let after_tag = search_pos + o + 1 + tag_name.len();
                if after_tag < html.len() {
                    let next_char = html.chars().nth(after_tag)?;
                    if next_char.is_whitespace() || next_char == '>' || next_char == '/' {
                        depth += 1;
                    }
                }
                search_pos += o + 1;
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(search_pos + c + close_tag.len());
                }
                search_pos += c + 1;
            }
            (Some(o), None) => {
                search_pos += o + 1;
            }
            (None, None) => break,
        }
    }

    None
}

/// Find element content boundaries.
fn find_element_content(html: &str) -> Option<(usize, usize)> {
    let tag_end = html.find('>')?;
    let element_end = find_element_end(html)?;

    let close_tag_start = html[..element_end].rfind("</")?;

    Some((tag_end + 1, close_tag_start))
}

/// Transform a template by processing switch directives.
pub fn transform_switch_template(template: &str, context: &dyn Fn(&str) -> SwitchValue) -> String {
    let parsed = parse_switch_template(template);

    if !parsed.has_switches() {
        return template.to_string();
    }

    let mut result = template.to_string();

    for switch in parsed.switches.iter().rev() {
        let value = context(&switch.expression);

        let mut matched_template = None;
        let mut default_template = None;

        for case in &switch.cases {
            if case.is_default {
                default_template = Some(&case.template);
            } else if let Some(ref case_value) = case.value {
                let case_switch_value = SwitchValue::parse(case_value);
                if case_switch_value == value {
                    matched_template = Some(&case.template);
                    break;
                }
            }
        }

        let replacement = matched_template
            .or(default_template)
            .map(|s| s.as_str())
            .unwrap_or("");

        result.replace_range(switch.start..switch.end, replacement);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_value_parse() {
        assert_eq!(SwitchValue::parse("'hello'"), SwitchValue::String("hello".to_string()));
        assert_eq!(SwitchValue::parse("\"world\""), SwitchValue::String("world".to_string()));
        assert_eq!(SwitchValue::parse("42"), SwitchValue::Int(42));
        assert_eq!(SwitchValue::parse("3.14"), SwitchValue::Float(3.14));
        assert_eq!(SwitchValue::parse("true"), SwitchValue::Bool(true));
        assert_eq!(SwitchValue::parse("false"), SwitchValue::Bool(false));
        assert_eq!(SwitchValue::parse("null"), SwitchValue::Null);
    }

    #[test]
    fn test_switch_container() {
        let mut container = FeSwitchContainer::new();

        container.add_case(SwitchCase::new("red", "<p>Red</p>"));
        container.add_case(SwitchCase::new("blue", "<p>Blue</p>"));
        container.add_case(SwitchCase::default("<p>Unknown</p>"));

        container.set_value("red");
        assert_eq!(container.render(), Some("<p>Red</p>"));

        container.set_value("blue");
        assert_eq!(container.render(), Some("<p>Blue</p>"));

        container.set_value("green");
        assert_eq!(container.render(), Some("<p>Unknown</p>"));
    }

    #[test]
    fn test_switch_with_integers() {
        let mut container = FeSwitchContainer::new();

        container.add_case(SwitchCase::new(1, "<p>One</p>"));
        container.add_case(SwitchCase::new(2, "<p>Two</p>"));
        container.add_case(SwitchCase::new(3, "<p>Three</p>"));

        container.set_value(2);
        assert_eq!(container.render(), Some("<p>Two</p>"));
    }

    #[test]
    fn test_case_matching() {
        let case = SwitchCase::new("test", "template");
        assert!(case.matches(&SwitchValue::String("test".to_string())));
        assert!(!case.matches(&SwitchValue::String("other".to_string())));
    }

    #[test]
    fn test_default_case() {
        let default = SwitchCase::default("default template");
        assert!(default.is_default);
        assert!(!default.matches(&SwitchValue::String("anything".to_string())));
    }

    #[test]
    fn test_extract_attribute_value() {
        let html = r#"<div [feSwitch]="color" class="container">"#;
        assert_eq!(extract_attribute_value(html, "[feSwitch]"), Some("color".to_string()));
    }
}

