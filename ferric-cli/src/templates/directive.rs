//! Directive templates

/// Generate directive Rust file
pub fn directive_rs(name: &str) -> String {
    format!(
        r#"//! {name} directive

use wasm_bindgen::prelude::*;
use web_sys::Element;

/// {name} directive
///
/// Directives are used to add behavior to existing DOM elements.
pub struct {name} {{
    element: Element,
}}

impl {name} {{
    /// Apply the directive to an element
    pub fn apply(element: Element) -> Result<Self, JsValue> {{
        let directive = Self {{ element }};
        directive.init()?;
        Ok(directive)
    }}

    fn init(&self) -> Result<(), JsValue> {{
        // Initialize directive behavior
        // Example: Add event listeners, modify attributes, etc.

        Ok(())
    }}

    /// Get the element this directive is applied to
    pub fn element(&self) -> &Element {{
        &self.element
    }}
}}

impl Drop for {name} {{
    fn drop(&mut self) {{
        // Cleanup: remove event listeners, etc.
    }}
}}
"#,
        name = name
    )
}
