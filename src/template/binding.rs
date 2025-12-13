//! Data binding types for templates.

/// Types of data bindings supported in templates.
#[derive(Debug, Clone, PartialEq)]
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

/// Output binding configuration.
#[derive(Debug, Clone)]
pub struct OutputBinding {
    /// The event name on the component.
    pub event: String,
    /// The handler expression.
    pub handler: String,
}

