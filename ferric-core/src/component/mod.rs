//! Component system for Ferric.
//!
//! Components are the fundamental building blocks of Ferric applications.
//! Each component encapsulates its own template, styles, and logic.
//!
//! ## View Encapsulation
//!
//! Components support three encapsulation modes:
//!
//! - **Emulated** (default): Styles are scoped using attribute selectors
//! - **ShadowDom**: Uses native Shadow DOM for style isolation
//! - **None**: Styles are global, no scoping
//!
//! ```ignore
//! #[component(
//!     selector = "app-card",
//!     template = "<div class=\"card\"><fe-content></fe-content></div>",
//!     styles = ".card { padding: 1rem; }",
//!     encapsulation = "Emulated"
//! )]
//! pub struct CardComponent;
//! ```

mod context;
mod encapsulation;
mod metadata;
mod registry;

pub use context::*;
pub use encapsulation::*;
pub use metadata::*;
pub use registry::*;

use crate::lifecycle::Lifecycle;
use std::any::Any;
use std::collections::HashMap;

/// Trait that all Ferric components must implement.
pub trait Component: Lifecycle {
    /// Returns the component's selector (e.g., "app-root", "my-button").
    fn selector(&self) -> &'static str;

    /// Returns the component's template as a string.
    fn template(&self) -> &str;

    /// Returns the component's styles (optional).
    fn styles(&self) -> Option<&str> {
        None
    }

    /// Returns the view encapsulation mode for this component.
    fn encapsulation(&self) -> ViewEncapsulation {
        ViewEncapsulation::Emulated
    }

    /// Render the component and return the resulting DOM.
    fn render(&self) -> Result<web_sys::Element, String>;

    /// Render with encapsulation applied.
    fn render_encapsulated(&self) -> Result<(web_sys::Element, String), String> {
        let encapsulator = ViewEncapsulator::new(self.encapsulation());

        let template = self.template();
        let styles = self.styles().unwrap_or("");

        let (encapsulated_template, encapsulated_styles) = encapsulator.encapsulate(template, styles);

        // Store the encapsulated template for rendering
        // The actual DOM creation happens in the render method
        let element = self.render()?;

        // Add host attribute if using emulated encapsulation
        if self.encapsulation() == ViewEncapsulation::Emulated {
            let _ = element.set_attribute(encapsulator.host_attr(), "");
        }

        Ok((element, encapsulated_styles))
    }

    /// Handle property changes from parent components.
    fn on_changes(&mut self, _changes: &HashMap<String, Box<dyn Any>>) {}
}

/// Helper trait for components with encapsulation support.
pub trait EncapsulatedComponent: Component {
    /// Get the view encapsulator for this component.
    fn view_encapsulator(&self) -> ViewEncapsulator {
        ViewEncapsulator::new(self.encapsulation())
    }

    /// Get encapsulated styles.
    fn encapsulated_styles(&self) -> String {
        let encapsulator = self.view_encapsulator();
        encapsulator.style.encapsulate_styles(self.styles().unwrap_or(""))
    }

    /// Get encapsulated template.
    fn encapsulated_template(&self) -> String {
        let encapsulator = self.view_encapsulator();
        encapsulator.template.encapsulate_template(self.template())
    }
}

// Blanket implementation for all components
impl<T: Component> EncapsulatedComponent for T {}

// The #[component] attribute macro from ferric_macros is used instead
// of a declarative macro for better Angular-like syntax.
