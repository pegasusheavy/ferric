//! Main application component with external template and styles.
//!
//! This demonstrates using `include_str!()` to inline external HTML and CSS files
//! at compile time, similar to Angular's `templateUrl` and `styleUrls`.

use crate::template::{html, inject_styles, query};
use web_sys::Element;

/// CSS styles loaded from external file at compile time.
/// The file path is relative to this source file.
const STYLES: &str = include_str!("todo_app.css");

/// HTML template loaded from external file at compile time.
/// The file path is relative to this source file.
const TEMPLATE: &str = include_str!("todo_app.html");

/// The main application component.
///
/// # Example
///
/// ```ignore
/// // Component with external template and styles
/// const STYLES: &str = include_str!("my_component.css");
/// const TEMPLATE: &str = include_str!("my_component.html");
///
/// pub struct MyComponent {
///     element: Option<Element>,
/// }
///
/// impl MyComponent {
///     pub fn render(&mut self) -> Element {
///         inject_styles("my-component", STYLES);
///         let element = html(TEMPLATE);
///         self.element = Some(element.clone());
///         element
///     }
/// }
/// ```
pub struct TodoAppComponent {
    element: Option<Element>,
}

impl TodoAppComponent {
    pub fn new() -> Self {
        Self { element: None }
    }

    /// Render and return the root element.
    pub fn render(&mut self) -> Element {
        // Inject styles (idempotent - won't duplicate)
        inject_styles("todo-app", STYLES);

        // Parse template to DOM
        let element = html(TEMPLATE);
        self.element = Some(element.clone());
        element
    }

    /// Get the todo list container.
    pub fn todo_list(&self) -> Option<Element> {
        self.element.as_ref().and_then(|el| query(el, ".todo-list"))
    }

    /// Get the input element.
    pub fn input(&self) -> Option<Element> {
        self.element.as_ref().and_then(|el| query(el, ".new-todo"))
    }

    /// Get the footer element.
    pub fn footer(&self) -> Option<Element> {
        self.element.as_ref().and_then(|el| query(el, ".todo-footer"))
    }

    /// Get the toggle-all checkbox.
    pub fn toggle_all(&self) -> Option<Element> {
        self.element.as_ref().and_then(|el| query(el, ".toggle-all"))
    }

    /// Get the count element.
    pub fn count_element(&self) -> Option<Element> {
        self.element.as_ref().and_then(|el| query(el, ".todo-count"))
    }

    /// Get the clear completed button.
    pub fn clear_button(&self) -> Option<Element> {
        self.element.as_ref().and_then(|el| query(el, ".clear-completed"))
    }

    /// Get filter links.
    pub fn filter_links(&self) -> Vec<Element> {
        self.element
            .as_ref()
            .map(|el| crate::template::query_all(el, ".filters a"))
            .unwrap_or_default()
    }
}

impl Default for TodoAppComponent {
    fn default() -> Self {
        Self::new()
    }
}
