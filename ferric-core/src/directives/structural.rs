//! Structural directives that modify DOM structure.

use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::Element;

/// The *if directive - conditionally renders content.
pub struct IfDirective {
    condition: bool,
    template: Option<Element>,
    view_container: Option<Element>,
    current_view: Option<Element>,
}

impl IfDirective {
    /// Create a new *if directive.
    pub fn new() -> Self {
        Self {
            condition: false,
            template: None,
            view_container: None,
            current_view: None,
        }
    }

    /// Set the template to render.
    pub fn set_template(&mut self, template: Element) {
        self.template = Some(template);
    }

    /// Set the container where views will be rendered.
    pub fn set_container(&mut self, container: Element) {
        self.view_container = Some(container);
    }

    /// Set the condition.
    pub fn set_condition(&mut self, condition: bool) {
        if self.condition != condition {
            self.condition = condition;
            self.update();
        }
    }

    /// Get the current condition.
    pub fn condition(&self) -> bool {
        self.condition
    }

    /// Update the view based on the current condition.
    fn update(&mut self) {
        if let (Some(template), Some(container)) = (&self.template, &self.view_container) {
            if self.condition {
                // Show the template if not already shown
                if self.current_view.is_none() {
                    let view = clone_node(template);
                    let _ = container.append_child(&view);
                    self.current_view = Some(view);
                }
            } else {
                // Remove the view if shown
                if let Some(ref view) = self.current_view {
                    if let Some(parent) = view.parent_node() {
                        let _ = parent.remove_child(view);
                    }
                    self.current_view = None;
                }
            }
        }
    }

    /// Clear all views.
    pub fn clear(&mut self) {
        if let Some(ref view) = self.current_view
            && let Some(parent) = view.parent_node() {
                let _ = parent.remove_child(view);
            }
        self.current_view = None;
    }
}

impl Default for IfDirective {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IfDirective {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Context for a single item in a *for directive.
#[derive(Debug, Clone)]
pub struct ForContext<T> {
    /// The current item.
    pub item: T,
    /// Zero-based index of the item.
    pub index: usize,
    /// Total number of items.
    pub count: usize,
    /// Whether this is the first item.
    pub first: bool,
    /// Whether this is the last item.
    pub last: bool,
    /// Whether this item has an even index.
    pub even: bool,
    /// Whether this item has an odd index.
    pub odd: bool,
}

impl<T> ForContext<T> {
    fn new(item: T, index: usize, count: usize) -> Self {
        Self {
            item,
            index,
            count,
            first: index == 0,
            last: index == count - 1,
            even: index.is_multiple_of(2),
            odd: !index.is_multiple_of(2),
        }
    }
}

/// A view instance created by ForDirective.
struct ForView<T> {
    /// The DOM element representing this view.
    element: Element,
    /// The context for this view.
    context: ForContext<T>,
    /// The tracking key for this view.
    key: String,
}

/// The *for directive - renders a template for each item in a collection.
///
/// This directive efficiently renders lists by:
/// - Tracking items by identity to minimize DOM operations
/// - Reusing existing views when items haven't changed
/// - Only creating/destroying views for added/removed items
///
/// # Example
///
/// ```ignore
/// let mut directive = ForDirective::new();
/// directive.set_template(template_element);
/// directive.set_container(container_element);
/// directive.track_by(|item| item.id.to_string());
/// directive.set_items(vec![item1, item2, item3]);
/// ```
pub struct ForDirective<T: Clone> {
    items: Vec<T>,
    track_by: Option<Box<dyn Fn(&T) -> String>>,
    template: Option<Element>,
    view_container: Option<Element>,
    views: Vec<ForView<T>>,
    view_map: HashMap<String, usize>, // Maps tracking keys to view indices
}

impl<T: Clone> ForDirective<T> {
    /// Create a new *for directive.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            track_by: None,
            template: None,
            view_container: None,
            views: Vec::new(),
            view_map: HashMap::new(),
        }
    }

    /// Set the template to use for each item.
    pub fn set_template(&mut self, template: Element) {
        self.template = Some(template);
    }

    /// Set the container where views will be rendered.
    pub fn set_container(&mut self, container: Element) {
        self.view_container = Some(container);
    }

    /// Set the items to iterate over.
    ///
    /// This will trigger an update that efficiently diffs the old and new items,
    /// reusing views where possible.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.update();
    }

    /// Get the current items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Set the track-by function.
    ///
    /// This function is used to generate a unique key for each item, which allows
    /// the directive to efficiently track which items have been added, removed, or moved.
    ///
    /// If not set, items are tracked by index, which can be inefficient for dynamic lists.
    pub fn track_by<F>(&mut self, f: F)
    where
        F: Fn(&T) -> String + 'static,
    {
        self.track_by = Some(Box::new(f));
    }

    /// Get the tracking key for an item.
    fn get_key(&self, item: &T, index: usize) -> String {
        if let Some(ref track_fn) = self.track_by {
            track_fn(item)
        } else {
            // Default to index-based tracking
            index.to_string()
        }
    }

    /// Update the views based on the current items.
    ///
    /// This performs a diff between the old and new items and:
    /// 1. Reuses existing views for items that haven't changed
    /// 2. Creates new views for added items
    /// 3. Removes views for removed items
    /// 4. Updates the context (index, first, last, etc.) for all views
    fn update(&mut self) {
        let template = match &self.template {
            Some(t) => t,
            None => return, // No template, nothing to render
        };

        let container = match &self.view_container {
            Some(c) => c,
            None => return, // No container, nowhere to render
        };

        // Build a map of new items by their keys
        let count = self.items.len();
        let mut new_items: HashMap<String, (usize, T)> = HashMap::new();

        for (index, item) in self.items.iter().enumerate() {
            let key = self.get_key(item, index);
            new_items.insert(key, (index, item.clone()));
        }

        // Determine which views to keep, update, or remove
        let mut views_to_keep: Vec<Option<ForView<T>>> = (0..count).map(|_| None).collect();
        let mut old_views = std::mem::take(&mut self.views);

        // First pass: find views that can be reused
        for view in old_views.drain(..) {
            if let Some(&(new_index, ref item)) = new_items.get(&view.key) {
                // Item still exists, update its context
                let context = ForContext::new(item.clone(), new_index, count);
                views_to_keep[new_index] = Some(ForView {
                    element: view.element,
                    context,
                    key: view.key,
                });
            } else {
                // Item was removed, destroy the view
                if let Some(parent) = view.element.parent_node() {
                    let _ = parent.remove_child(&view.element);
                }
            }
        }

        // Second pass: create views for new items
        for (index, item) in self.items.iter().enumerate() {
            if views_to_keep[index].is_none() {
                let key = self.get_key(item, index);
                let context = ForContext::new(item.clone(), index, count);
                let element = self.create_view(template, &context);

                views_to_keep[index] = Some(ForView {
                    element,
                    context,
                    key,
                });
            }
        }

        // Third pass: insert views in the correct order
        // Clear the container first
        while let Some(child) = container.first_child() {
            let _ = container.remove_child(&child);
        }

        // Rebuild view_map and append views
        self.view_map.clear();
        self.views.clear();

        for (index, view_option) in views_to_keep.into_iter().enumerate() {
            if let Some(view) = view_option {
                let _ = container.append_child(&view.element);
                self.view_map.insert(view.key.clone(), index);
                self.views.push(view);
            }
        }
    }

    /// Create a view for an item by cloning the template.
    fn create_view(&self, template: &Element, _context: &ForContext<T>) -> Element {
        // Clone the template
        

        // In a full implementation, you would:
        // 1. Set up bindings for the context variables ($implicit, index, etc.)
        // 2. Initialize any child components
        // 3. Attach event listeners

        // For now, just return the cloned template
        clone_node(template)
    }

    /// Clear all views.
    pub fn clear(&mut self) {
        for view in &self.views {
            if let Some(parent) = view.element.parent_node() {
                let _ = parent.remove_child(&view.element);
            }
        }
        self.views.clear();
        self.view_map.clear();
    }

    /// Get the number of rendered views.
    pub fn view_count(&self) -> usize {
        self.views.len()
    }

    /// Get a reference to a view by index.
    pub fn get_view(&self, index: usize) -> Option<&Element> {
        self.views.get(index).map(|v| &v.element)
    }
}

impl<T: Clone> Default for ForDirective<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Drop for ForDirective<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// The *switch directive - conditionally renders one of several templates.
pub struct SwitchDirective<T: PartialEq + Clone> {
    value: Option<T>,
    cases: Vec<(T, Element)>,
    default_case: Option<Element>,
    view_container: Option<Element>,
    active_view: Option<Element>,
}

impl<T: PartialEq + Clone> SwitchDirective<T> {
    /// Create a new *switch directive.
    pub fn new() -> Self {
        Self {
            value: None,
            cases: Vec::new(),
            default_case: None,
            view_container: None,
            active_view: None,
        }
    }

    /// Set the container where views will be rendered.
    pub fn set_container(&mut self, container: Element) {
        self.view_container = Some(container);
    }

    /// Set the switch value.
    pub fn set_value(&mut self, value: T) {
        let should_update = match &self.value {
            Some(old_value) => old_value != &value,
            None => true,
        };

        self.value = Some(value);

        if should_update {
            self.update();
        }
    }

    /// Get the current value.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Add a case.
    pub fn add_case(&mut self, value: T, template: Element) {
        self.cases.push((value, template));
    }

    /// Set the default case.
    pub fn set_default(&mut self, template: Element) {
        self.default_case = Some(template);
    }

    /// Update the active view based on the current value.
    fn update(&mut self) {
        if let Some(ref value) = self.value {
            // Find matching case
            for (case_value, template) in &self.cases {
                if case_value == value {
                    self.activate_view(template.clone());
                    return;
                }
            }

            // Fall back to default
            if let Some(ref default) = self.default_case {
                self.activate_view(default.clone());
            } else {
                self.clear_active_view();
            }
        }
    }

    fn activate_view(&mut self, template: Element) {
        // Deactivate current view
        self.clear_active_view();

        // Clone and activate new view
        let view = clone_node(&template);

        if let Some(ref container) = self.view_container {
            let _ = container.append_child(&view);
        }

        self.active_view = Some(view);
    }

    fn clear_active_view(&mut self) {
        if let Some(ref active) = self.active_view
            && let Some(parent) = active.parent_node() {
                let _ = parent.remove_child(active);
            }
        self.active_view = None;
    }

    /// Clear the active view.
    pub fn clear(&mut self) {
        self.clear_active_view();
    }
}

impl<T: PartialEq + Clone> Default for SwitchDirective<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq + Clone> Drop for SwitchDirective<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Helper function to clone a DOM node deeply.
fn clone_node(node: &Element) -> Element {
    node.clone_node_with_deep(true)
        .expect("Failed to clone node")
        .dyn_into::<Element>()
        .expect("Cloned node is not an Element")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would need a browser environment or wasm-bindgen-test
    // For now, they serve as documentation of the intended behavior

    #[test]
    fn test_if_directive_structure() {
        let directive = IfDirective::new();
        assert!(!directive.condition());
    }

    #[test]
    fn test_for_directive_structure() {
        let directive: ForDirective<i32> = ForDirective::new();
        assert_eq!(directive.items().len(), 0);
        assert_eq!(directive.view_count(), 0);
    }

    #[test]
    fn test_for_context() {
        let ctx = ForContext::new(42, 0, 3);
        assert_eq!(ctx.item, 42);
        assert_eq!(ctx.index, 0);
        assert_eq!(ctx.count, 3);
        assert!(ctx.first);
        assert!(!ctx.last);
        assert!(ctx.even);
        assert!(!ctx.odd);

        let ctx = ForContext::new(42, 2, 3);
        assert!(ctx.last);
        assert!(ctx.even);
    }

    #[test]
    fn test_switch_directive_structure() {
        let directive: SwitchDirective<i32> = SwitchDirective::new();
        assert!(directive.value().is_none());
    }
}
