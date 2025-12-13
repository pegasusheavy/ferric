//! Component system for Ferric.
//!
//! Components are the fundamental building blocks of Ferric applications.
//! Each component encapsulates its own template, styles, and logic.

mod context;
mod metadata;
mod registry;

pub use context::*;
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

    /// Render the component and return the resulting DOM.
    fn render(&self) -> Result<web_sys::Element, String>;

    /// Handle property changes from parent components.
    fn on_changes(&mut self, _changes: &HashMap<String, Box<dyn Any>>) {}
}

/// Macro to define a component with metadata.
#[macro_export]
macro_rules! component {
    (
        selector: $selector:literal,
        template: $template:literal,
        $(styles: $styles:literal,)?
        struct $name:ident {
            $($field:ident : $type:ty),* $(,)?
        }
    ) => {
        pub struct $name {
            $($field: $type,)*
        }

        impl $crate::Component for $name {
            fn selector(&self) -> &'static str {
                $selector
            }

            fn template(&self) -> &str {
                $template
            }

            $(
                fn styles(&self) -> Option<&str> {
                    Some($styles)
                }
            )?

            fn render(&self) -> Result<web_sys::Element, String> {
                // Default rendering implementation
                todo!("Implement rendering")
            }
        }
    };
}

