//! CSS selector matching for content projection.

use std::fmt;

/// Kinds of selectors supported for content projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorKind {
    /// Match any element (default slot).
    Any,
    /// Match by tag name: `div`, `span`, etc.
    Tag(String),
    /// Match by class: `.my-class`.
    Class(String),
    /// Match by attribute presence: `[my-attr]`.
    Attribute(String),
    /// Match by attribute value: `[my-attr="value"]`.
    AttributeValue(String, String),
    /// Combined tag and class: `div.my-class`.
    TagClass(String, String),
    /// Combined tag and attribute: `div[my-attr]`.
    TagAttribute(String, String),
    /// Multiple selectors (comma-separated).
    Multiple(Vec<SelectorKind>),
}

impl SelectorKind {
    /// Check if this is the default (any) selector.
    pub fn is_default(&self) -> bool {
        matches!(self, SelectorKind::Any)
    }
}

/// A parsed content selector.
#[derive(Debug, Clone)]
pub struct ContentSelector {
    /// The original selector string.
    pub raw: String,
    /// The parsed selector kind.
    pub kind: SelectorKind,
}

impl ContentSelector {
    /// Create a new content selector from a string.
    pub fn new(selector: &str) -> Self {
        let kind = parse_selector(selector);
        Self {
            raw: selector.to_string(),
            kind,
        }
    }

    /// Create a default (any) selector.
    pub fn default() -> Self {
        Self {
            raw: String::new(),
            kind: SelectorKind::Any,
        }
    }

    /// Check if this is the default selector.
    pub fn is_default(&self) -> bool {
        self.kind.is_default()
    }
}

impl fmt::Display for ContentSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw.is_empty() {
            write!(f, "*")
        } else {
            write!(f, "{}", self.raw)
        }
    }
}

/// Parse a selector string into a SelectorKind.
fn parse_selector(selector: &str) -> SelectorKind {
    let selector = selector.trim();

    if selector.is_empty() {
        return SelectorKind::Any;
    }

    // Handle comma-separated multiple selectors
    if selector.contains(',') {
        let parts: Vec<SelectorKind> = selector
            .split(',')
            .map(|s| parse_selector(s.trim()))
            .collect();
        return SelectorKind::Multiple(parts);
    }

    // Attribute selector: [attr] or [attr="value"]
    if selector.starts_with('[') && selector.ends_with(']') {
        let inner = &selector[1..selector.len() - 1];
        if let Some(eq_pos) = inner.find('=') {
            let attr = inner[..eq_pos].trim().to_string();
            let value = inner[eq_pos + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            return SelectorKind::AttributeValue(attr, value);
        }
        return SelectorKind::Attribute(inner.to_string());
    }

    // Class selector: .class
    if selector.starts_with('.') && !selector.contains('[') {
        return SelectorKind::Class(selector[1..].to_string());
    }

    // Tag with class: tag.class
    if let Some(dot_pos) = selector.find('.') {
        if !selector.contains('[') {
            let tag = selector[..dot_pos].to_string();
            let class = selector[dot_pos + 1..].to_string();
            return SelectorKind::TagClass(tag, class);
        }
    }

    // Tag with attribute: tag[attr]
    if let Some(bracket_pos) = selector.find('[') {
        let tag = selector[..bracket_pos].to_string();
        let attr_part = &selector[bracket_pos..];
        if attr_part.ends_with(']') {
            let attr = attr_part[1..attr_part.len() - 1].to_string();
            return SelectorKind::TagAttribute(tag, attr);
        }
    }

    // Simple tag selector
    SelectorKind::Tag(selector.to_string())
}

/// Check if an element matches a selector.
///
/// This is a simplified selector matching that works with element properties.
pub fn matches_selector(
    selector: &SelectorKind,
    tag_name: &str,
    classes: &[&str],
    attributes: &[(&str, Option<&str>)],
) -> bool {
    match selector {
        SelectorKind::Any => true,

        SelectorKind::Tag(tag) => tag.eq_ignore_ascii_case(tag_name),

        SelectorKind::Class(class) => classes.iter().any(|c| *c == class),

        SelectorKind::Attribute(attr) => {
            attributes.iter().any(|(name, _)| *name == attr)
        }

        SelectorKind::AttributeValue(attr, value) => {
            attributes.iter().any(|(name, val)| {
                *name == attr && val.map_or(false, |v| v == value)
            })
        }

        SelectorKind::TagClass(tag, class) => {
            tag.eq_ignore_ascii_case(tag_name) && classes.iter().any(|c| *c == class)
        }

        SelectorKind::TagAttribute(tag, attr) => {
            tag.eq_ignore_ascii_case(tag_name)
                && attributes.iter().any(|(name, _)| *name == attr)
        }

        SelectorKind::Multiple(selectors) => {
            selectors.iter().any(|s| matches_selector(s, tag_name, classes, attributes))
        }
    }
}

/// Check if an element matches a selector using web_sys::Element.
#[cfg(target_arch = "wasm32")]
pub fn matches_element(selector: &ContentSelector, element: &web_sys::Element) -> bool {
    // Use native matches() when available
    if !selector.raw.is_empty() {
        if let Ok(matches) = element.matches(&selector.raw) {
            return matches;
        }
    }

    // Fall back to manual matching
    let tag_name = element.tag_name();
    let class_list = element.class_list();
    let classes: Vec<String> = (0..class_list.length())
        .filter_map(|i| class_list.item(i))
        .collect();
    let class_refs: Vec<&str> = classes.iter().map(|s| s.as_str()).collect();

    // Collect attributes
    let attrs = element.attributes();
    let mut attributes = Vec::new();
    for i in 0..attrs.length() {
        if let Some(attr) = attrs.item(i) {
            attributes.push((attr.name(), Some(attr.value())));
        }
    }
    let attr_refs: Vec<(&str, Option<&str>)> = attributes
        .iter()
        .map(|(n, v)| (n.as_str(), Some(v.as_str())))
        .collect();

    matches_selector(&selector.kind, &tag_name, &class_refs, &attr_refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_any() {
        let selector = ContentSelector::new("");
        assert!(selector.is_default());
    }

    #[test]
    fn test_parse_tag() {
        let selector = ContentSelector::new("div");
        assert!(matches!(selector.kind, SelectorKind::Tag(ref t) if t == "div"));
    }

    #[test]
    fn test_parse_class() {
        let selector = ContentSelector::new(".my-class");
        assert!(matches!(selector.kind, SelectorKind::Class(ref c) if c == "my-class"));
    }

    #[test]
    fn test_parse_attribute() {
        let selector = ContentSelector::new("[card-header]");
        assert!(matches!(selector.kind, SelectorKind::Attribute(ref a) if a == "card-header"));
    }

    #[test]
    fn test_parse_attribute_value() {
        let selector = ContentSelector::new(r#"[type="submit"]"#);
        assert!(matches!(
            selector.kind,
            SelectorKind::AttributeValue(ref a, ref v) if a == "type" && v == "submit"
        ));
    }

    #[test]
    fn test_parse_tag_class() {
        let selector = ContentSelector::new("button.primary");
        assert!(matches!(
            selector.kind,
            SelectorKind::TagClass(ref t, ref c) if t == "button" && c == "primary"
        ));
    }

    #[test]
    fn test_parse_multiple() {
        let selector = ContentSelector::new("[header], .title");
        assert!(matches!(selector.kind, SelectorKind::Multiple(ref v) if v.len() == 2));
    }

    #[test]
    fn test_matches_tag() {
        let kind = SelectorKind::Tag("div".to_string());
        assert!(matches_selector(&kind, "div", &[], &[]));
        assert!(matches_selector(&kind, "DIV", &[], &[]));
        assert!(!matches_selector(&kind, "span", &[], &[]));
    }

    #[test]
    fn test_matches_class() {
        let kind = SelectorKind::Class("active".to_string());
        assert!(matches_selector(&kind, "div", &["active", "primary"], &[]));
        assert!(!matches_selector(&kind, "div", &["primary"], &[]));
    }

    #[test]
    fn test_matches_attribute() {
        let kind = SelectorKind::Attribute("card-header".to_string());
        assert!(matches_selector(&kind, "h2", &[], &[("card-header", None)]));
        assert!(!matches_selector(&kind, "h2", &[], &[("card-footer", None)]));
    }
}

