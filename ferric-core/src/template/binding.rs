//! Data binding types for templates.

/// Types of data bindings supported in templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    /// One-way binding from component to view: {{ value }}
    Interpolation,
    /// Property binding: [property]="expression"
    Property,
    /// Event binding: (event)="handler()"
    Event,
    /// Two-way binding: [(property)]="value"
    TwoWay,
    /// Attribute binding: [attr.name]="value"
    Attribute,
    /// Class binding: [class.name]="condition"
    Class,
    /// Style binding: [style.property]="value"
    Style,
}

/// A parsed binding from a template.
#[derive(Debug, Clone)]
pub struct Binding {
    /// The type of binding.
    pub binding_type: BindingType,
    /// The target property, event, or attribute name.
    pub target: String,
    /// The expression or handler.
    pub expression: String,
}

impl Binding {
    /// Create a new binding.
    pub fn new(binding_type: BindingType, target: &str, expression: &str) -> Self {
        Self {
            binding_type,
            target: target.to_string(),
            expression: expression.to_string(),
        }
    }

    /// Create an interpolation binding.
    pub fn interpolation(expression: &str) -> Self {
        Self::new(BindingType::Interpolation, "", expression)
    }

    /// Create a property binding.
    pub fn property(name: &str, expression: &str) -> Self {
        // Check for special prefixes
        if name.starts_with("attr.") {
            return Self::attribute(&name[5..], expression);
        }
        if name.starts_with("class.") {
            return Self::class(&name[6..], expression);
        }
        if name.starts_with("style.") {
            return Self::style(&name[6..], expression);
        }
        Self::new(BindingType::Property, name, expression)
    }

    /// Create an event binding.
    pub fn event(name: &str, handler: &str) -> Self {
        Self::new(BindingType::Event, name, handler)
    }

    /// Create a two-way binding.
    pub fn two_way(name: &str, expression: &str) -> Self {
        Self::new(BindingType::TwoWay, name, expression)
    }

    /// Create an attribute binding.
    pub fn attribute(name: &str, expression: &str) -> Self {
        Self::new(BindingType::Attribute, name, expression)
    }

    /// Create a class binding.
    pub fn class(class_name: &str, condition: &str) -> Self {
        Self::new(BindingType::Class, class_name, condition)
    }

    /// Create a style binding.
    pub fn style(property: &str, expression: &str) -> Self {
        Self::new(BindingType::Style, property, expression)
    }
}

/// Input binding configuration.
#[derive(Debug, Clone)]
pub struct InputBinding {
    /// The property name on the component.
    pub property: String,
    /// The bound expression.
    pub expression: String,
    /// Whether this is a required input.
    pub required: bool,
}

impl InputBinding {
    /// Create a new input binding.
    pub fn new(property: &str, expression: &str) -> Self {
        Self {
            property: property.to_string(),
            expression: expression.to_string(),
            required: false,
        }
    }

    /// Mark as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Output binding configuration.
#[derive(Debug, Clone)]
pub struct OutputBinding {
    /// The event name on the component.
    pub event: String,
    /// The handler expression.
    pub handler: String,
}

impl OutputBinding {
    /// Create a new output binding.
    pub fn new(event: &str, handler: &str) -> Self {
        Self {
            event: event.to_string(),
            handler: handler.to_string(),
        }
    }
}

/// Parse a binding from an attribute.
///
/// # Examples
///
/// ```ignore
/// parse_binding_attribute("[value]", "name") -> Some(Binding::property("value", "name"))
/// parse_binding_attribute("(click)", "onClick()") -> Some(Binding::event("click", "onClick()"))
/// parse_binding_attribute("[(ngModel)]", "value") -> Some(Binding::two_way("ngModel", "value"))
/// parse_binding_attribute("[attr.disabled]", "isDisabled") -> Some(Binding::attribute("disabled", "isDisabled"))
/// parse_binding_attribute("[class.active]", "isActive") -> Some(Binding::class("active", "isActive"))
/// parse_binding_attribute("[style.color]", "textColor") -> Some(Binding::style("color", "textColor"))
/// ```
pub fn parse_binding_attribute(attr_name: &str, attr_value: &str) -> Option<Binding> {
    let name = attr_name.trim();
    let value = attr_value.trim();

    // Two-way binding: [(property)]="value"
    if name.starts_with("[(") && name.ends_with(")]") {
        let prop = &name[2..name.len() - 2];
        return Some(Binding::two_way(prop, value));
    }

    // Property binding: [property]="expression"
    if name.starts_with('[') && name.ends_with(']') {
        let prop = &name[1..name.len() - 1];
        return Some(Binding::property(prop, value));
    }

    // Event binding: (event)="handler"
    if name.starts_with('(') && name.ends_with(')') {
        let event = &name[1..name.len() - 1];
        return Some(Binding::event(event, value));
    }

    None
}

/// Check if a text contains interpolation expressions.
pub fn contains_interpolation(text: &str) -> bool {
    text.contains("{{") && text.contains("}}")
}

/// Extract interpolation expressions from text.
///
/// Returns a list of (start_index, end_index, expression) tuples.
pub fn extract_interpolations(text: &str) -> Vec<(usize, usize, String)> {
    let mut results = Vec::new();
    let mut pos = 0;

    while let Some(start) = text[pos..].find("{{") {
        let start_idx = pos + start;
        if let Some(end) = text[start_idx..].find("}}") {
            let end_idx = start_idx + end + 2;
            let expr = text[start_idx + 2..start_idx + end].trim().to_string();
            results.push((start_idx, end_idx, expr));
            pos = end_idx;
        } else {
            break;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_property_binding() {
        let binding = parse_binding_attribute("[value]", "name").unwrap();
        assert_eq!(binding.binding_type, BindingType::Property);
        assert_eq!(binding.target, "value");
        assert_eq!(binding.expression, "name");
    }

    #[test]
    fn test_parse_event_binding() {
        let binding = parse_binding_attribute("(click)", "onClick()").unwrap();
        assert_eq!(binding.binding_type, BindingType::Event);
        assert_eq!(binding.target, "click");
        assert_eq!(binding.expression, "onClick()");
    }

    #[test]
    fn test_parse_two_way_binding() {
        let binding = parse_binding_attribute("[(ngModel)]", "value").unwrap();
        assert_eq!(binding.binding_type, BindingType::TwoWay);
        assert_eq!(binding.target, "ngModel");
        assert_eq!(binding.expression, "value");
    }

    #[test]
    fn test_parse_attribute_binding() {
        let binding = parse_binding_attribute("[attr.disabled]", "isDisabled").unwrap();
        assert_eq!(binding.binding_type, BindingType::Attribute);
        assert_eq!(binding.target, "disabled");
    }

    #[test]
    fn test_parse_class_binding() {
        let binding = parse_binding_attribute("[class.active]", "isActive").unwrap();
        assert_eq!(binding.binding_type, BindingType::Class);
        assert_eq!(binding.target, "active");
    }

    #[test]
    fn test_parse_style_binding() {
        let binding = parse_binding_attribute("[style.color]", "textColor").unwrap();
        assert_eq!(binding.binding_type, BindingType::Style);
        assert_eq!(binding.target, "color");
    }

    #[test]
    fn test_extract_interpolations() {
        let text = "Hello {{ name }}, you have {{ count }} messages";
        let interps = extract_interpolations(text);

        assert_eq!(interps.len(), 2);
        assert_eq!(interps[0].2, "name");
        assert_eq!(interps[1].2, "count");
    }
}
