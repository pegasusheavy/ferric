//! Attribute directives that modify element behavior.

use super::AttributeDirective;
use wasm_bindgen::JsCast;

/// A directive that adds/removes a CSS class based on a condition.
pub struct ClassDirective {
    class_name: String,
    condition: bool,
}

impl ClassDirective {
    /// Create a new class directive.
    pub fn new(class_name: &str) -> Self {
        Self {
            class_name: class_name.to_string(),
            condition: false,
        }
    }

    /// Set the condition.
    pub fn set_condition(&mut self, condition: bool) {
        self.condition = condition;
    }
}

impl AttributeDirective for ClassDirective {
    fn selector(&self) -> &'static str {
        "[class]"
    }

    fn on_init(&mut self, element: &web_sys::Element) {
        self.on_changes(element);
    }

    fn on_changes(&mut self, element: &web_sys::Element) {
        let class_list = element.class_list();
        if self.condition {
            let _ = class_list.add_1(&self.class_name);
        } else {
            let _ = class_list.remove_1(&self.class_name);
        }
    }

    fn on_destroy(&mut self, element: &web_sys::Element) {
        let _ = element.class_list().remove_1(&self.class_name);
    }
}

/// A directive that sets inline styles.
pub struct StyleDirective {
    property: String,
    value: String,
}

impl StyleDirective {
    /// Create a new style directive.
    pub fn new(property: &str) -> Self {
        Self {
            property: property.to_string(),
            value: String::new(),
        }
    }

    /// Set the style value.
    pub fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }
}

impl AttributeDirective for StyleDirective {
    fn selector(&self) -> &'static str {
        "[style]"
    }

    fn on_init(&mut self, element: &web_sys::Element) {
        self.on_changes(element);
    }

    fn on_changes(&mut self, element: &web_sys::Element) {
        if let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
            let _ = html_element.style().set_property(&self.property, &self.value);
        }
    }

    fn on_destroy(&mut self, element: &web_sys::Element) {
        if let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
            let _ = html_element.style().remove_property(&self.property);
        }
    }
}

/// A directive that disables form elements.
pub struct DisabledDirective {
    disabled: bool,
}

impl DisabledDirective {
    /// Create a new disabled directive.
    pub fn new() -> Self {
        Self { disabled: false }
    }

    /// Set the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

impl Default for DisabledDirective {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeDirective for DisabledDirective {
    fn selector(&self) -> &'static str {
        "[disabled]"
    }

    fn on_init(&mut self, element: &web_sys::Element) {
        self.on_changes(element);
    }

    fn on_changes(&mut self, element: &web_sys::Element) {
        if self.disabled {
            let _ = element.set_attribute("disabled", "");
        } else {
            let _ = element.remove_attribute("disabled");
        }
    }

    fn on_destroy(&mut self, element: &web_sys::Element) {
        let _ = element.remove_attribute("disabled");
    }
}

/// A directive that manages element visibility.
pub struct HiddenDirective {
    hidden: bool,
}

impl HiddenDirective {
    /// Create a new hidden directive.
    pub fn new() -> Self {
        Self { hidden: false }
    }

    /// Set the hidden state.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }
}

impl Default for HiddenDirective {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeDirective for HiddenDirective {
    fn selector(&self) -> &'static str {
        "[hidden]"
    }

    fn on_init(&mut self, element: &web_sys::Element) {
        self.on_changes(element);
    }

    fn on_changes(&mut self, element: &web_sys::Element) {
        if self.hidden {
            let _ = element.set_attribute("hidden", "");
        } else {
            let _ = element.remove_attribute("hidden");
        }
    }

    fn on_destroy(&mut self, element: &web_sys::Element) {
        let _ = element.remove_attribute("hidden");
    }
}

