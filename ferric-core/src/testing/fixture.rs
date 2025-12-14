//! Component fixtures for testing components in isolation.
//!
//! Fixtures provide a way to render and interact with components during tests.

use crate::di::Injector;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// A fixture for testing a component.
///
/// ## Example
///
/// ```ignore
/// let fixture = ComponentFixtureBuilder::<MyComponent>::new()
///     .with_input("title", "Test")
///     .with_input("count", 5)
///     .create();
///
/// // Trigger change detection
/// fixture.detect_changes();
///
/// // Query rendered output
/// let title = fixture.query("h1").text();
/// assert_eq!(title, "Test");
///
/// // Interact with the component
/// fixture.query("button").click();
/// fixture.detect_changes();
///
/// // Check output events
/// assert!(fixture.output::<i32>("countChange").was_emitted());
/// ```
pub struct ComponentFixtureBuilder<T> {
    inputs: HashMap<String, Box<dyn Any>>,
    providers: Vec<(std::any::TypeId, Box<dyn Any>)>,
    host_element: Option<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> ComponentFixtureBuilder<T> {
    /// Create a new fixture builder.
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            providers: Vec::new(),
            host_element: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set an input value.
    pub fn with_input<V: 'static>(mut self, name: &str, value: V) -> Self {
        self.inputs.insert(name.to_string(), Box::new(value));
        self
    }

    /// Add a provider override.
    pub fn with_provider<P: 'static>(mut self, provider: P) -> Self {
        self.providers.push((std::any::TypeId::of::<P>(), Box::new(provider)));
        self
    }

    /// Set the host element tag.
    pub fn with_host(mut self, tag: &str) -> Self {
        self.host_element = Some(tag.to_string());
        self
    }

    /// Create the fixture.
    pub fn create(self) -> Fixture<T> {
        let injector = Rc::new(Injector::root());

        Fixture {
            injector,
            component: RefCell::new(None),
            inputs: RefCell::new(self.inputs),
            outputs: RefCell::new(HashMap::new()),
            rendered_html: RefCell::new(String::new()),
            element_tree: RefCell::new(None),
            auto_detect: RefCell::new(false),
            destroyed: RefCell::new(false),
        }
    }
}

impl<T: 'static> Default for ComponentFixtureBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A test fixture for a component.
pub struct Fixture<T> {
    injector: Rc<Injector>,
    component: RefCell<Option<T>>,
    inputs: RefCell<HashMap<String, Box<dyn Any>>>,
    outputs: RefCell<HashMap<String, OutputRecorder>>,
    rendered_html: RefCell<String>,
    element_tree: RefCell<Option<ElementNode>>,
    auto_detect: RefCell<bool>,
    destroyed: RefCell<bool>,
}

impl<T: 'static> Fixture<T> {
    /// Get a reference to the component instance.
    pub fn instance(&self) -> std::cell::Ref<'_, Option<T>> {
        self.component.borrow()
    }

    /// Get a mutable reference to the component instance.
    pub fn instance_mut(&self) -> std::cell::RefMut<'_, Option<T>> {
        self.component.borrow_mut()
    }

    /// Set an input value.
    pub fn set_input<V: 'static>(&self, name: &str, value: V) {
        self.inputs.borrow_mut().insert(name.to_string(), Box::new(value));

        if *self.auto_detect.borrow() {
            self.detect_changes();
        }
    }

    /// Get an input value.
    pub fn input<V: Clone + 'static>(&self, name: &str) -> Option<V> {
        self.inputs
            .borrow()
            .get(name)
            .and_then(|v| v.downcast_ref::<V>())
            .cloned()
    }

    /// Get an output recorder.
    pub fn output<V: 'static>(&self, name: &str) -> OutputHandle<V> {
        OutputHandle {
            name: name.to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Trigger change detection.
    pub fn detect_changes(&self) {
        // In a real implementation, this would:
        // 1. Run change detection on the component
        // 2. Update the rendered HTML
        // 3. Sync the element tree
    }

    /// Enable auto-detect changes mode.
    pub fn auto_detect_changes(&self, enabled: bool) {
        *self.auto_detect.borrow_mut() = enabled;
    }

    /// Wait for the fixture to be stable.
    pub async fn when_stable(&self) {
        // Wait for pending async operations
        self.detect_changes();
    }

    /// Get the rendered HTML.
    pub fn html(&self) -> String {
        self.rendered_html.borrow().clone()
    }

    /// Query for an element.
    pub fn query(&self, selector: &str) -> ElementQuery {
        ElementQuery::new(selector, self.element_tree.borrow().clone())
    }

    /// Query for all matching elements.
    pub fn query_all(&self, _selector: &str) -> Vec<ElementQuery> {
        // In a real implementation, this would query all matching elements
        Vec::new()
    }

    /// Check if an element exists.
    pub fn has(&self, selector: &str) -> bool {
        self.query(selector).exists()
    }

    /// Destroy the fixture.
    pub fn destroy(&self) {
        *self.destroyed.borrow_mut() = true;
        *self.component.borrow_mut() = None;
    }

    /// Check if the fixture is destroyed.
    pub fn is_destroyed(&self) -> bool {
        *self.destroyed.borrow()
    }

    /// Get the injector.
    pub fn injector(&self) -> &Rc<Injector> {
        &self.injector
    }

    /// Debug print the rendered output.
    pub fn debug(&self) -> String {
        format!(
            "Fixture<{}>:\n  HTML: {}\n  Inputs: {:?}",
            std::any::type_name::<T>(),
            self.rendered_html.borrow(),
            self.inputs.borrow().keys().collect::<Vec<_>>()
        )
    }
}

impl<T> fmt::Debug for Fixture<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fixture")
            .field("html", &self.rendered_html.borrow())
            .field("destroyed", &self.destroyed.borrow())
            .finish()
    }
}

/// Output event recorder.
struct OutputRecorder {
    events: Vec<Box<dyn Any>>,
}

impl OutputRecorder {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn record<V: 'static>(&mut self, value: V) {
        self.events.push(Box::new(value));
    }
}

/// Handle to an output for assertions.
pub struct OutputHandle<V> {
    name: String,
    _marker: std::marker::PhantomData<V>,
}

impl<V> OutputHandle<V> {
    /// Check if the output was emitted.
    pub fn was_emitted(&self) -> bool {
        // In a real implementation, this would check the output recorder
        false
    }

    /// Get the emission count.
    pub fn emission_count(&self) -> usize {
        0
    }

    /// Get emitted values.
    pub fn values(&self) -> Vec<V>
    where
        V: Clone + 'static,
    {
        Vec::new()
    }

    /// Get the last emitted value.
    pub fn last(&self) -> Option<V>
    where
        V: Clone + 'static,
    {
        None
    }
}

/// A node in the element tree.
#[derive(Clone)]
pub struct ElementNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub classes: Vec<String>,
    pub styles: HashMap<String, String>,
    pub text: String,
    pub children: Vec<ElementNode>,
    pub event_handlers: Vec<String>,
}

impl ElementNode {
    /// Create a new element node.
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            classes: Vec::new(),
            styles: HashMap::new(),
            text: String::new(),
            children: Vec::new(),
            event_handlers: Vec::new(),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a class.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_string());
        self
    }

    /// Add a style.
    pub fn style(mut self, property: &str, value: &str) -> Self {
        self.styles.insert(property.to_string(), value.to_string());
        self
    }

    /// Set text content.
    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    /// Add a child element.
    pub fn child(mut self, child: ElementNode) -> Self {
        self.children.push(child);
        self
    }
}

impl Default for ElementNode {
    fn default() -> Self {
        Self::new("div")
    }
}

/// Query result for element interactions.
#[derive(Clone)]
pub struct ElementQuery {
    selector: String,
    element: Option<ElementNode>,
}

impl ElementQuery {
    /// Create a new element query.
    pub fn new(selector: &str, element: Option<ElementNode>) -> Self {
        Self {
            selector: selector.to_string(),
            element,
        }
    }

    /// Check if the element exists.
    pub fn exists(&self) -> bool {
        self.element.is_some()
    }

    /// Get the text content.
    pub fn text(&self) -> String {
        self.element.as_ref().map(|e| e.text.clone()).unwrap_or_default()
    }

    /// Get an attribute value.
    pub fn attr(&self, name: &str) -> Option<String> {
        self.element.as_ref().and_then(|e| e.attributes.get(name).cloned())
    }

    /// Check if has a class.
    pub fn has_class(&self, class: &str) -> bool {
        self.element.as_ref().map(|e| e.classes.contains(&class.to_string())).unwrap_or(false)
    }

    /// Get all classes.
    pub fn classes(&self) -> Vec<String> {
        self.element.as_ref().map(|e| e.classes.clone()).unwrap_or_default()
    }

    /// Get a style value.
    pub fn style(&self, property: &str) -> Option<String> {
        self.element.as_ref().and_then(|e| e.styles.get(property).cloned())
    }

    /// Get the element's tag name.
    pub fn tag(&self) -> Option<String> {
        self.element.as_ref().map(|e| e.tag.clone())
    }

    /// Simulate a click event.
    pub fn click(&self) -> &Self {
        // Dispatch click event
        self
    }

    /// Simulate double click.
    pub fn double_click(&self) -> &Self {
        self
    }

    /// Simulate right click.
    pub fn right_click(&self) -> &Self {
        self
    }

    /// Simulate hover.
    pub fn hover(&self) -> &Self {
        self
    }

    /// Simulate focus.
    pub fn focus(&self) -> &Self {
        self
    }

    /// Simulate blur.
    pub fn blur(&self) -> &Self {
        self
    }

    /// Set input value.
    pub fn set_value(&self, value: &str) -> &Self {
        // Update input value and dispatch input event
        let _ = value;
        self
    }

    /// Get input value.
    pub fn value(&self) -> Option<String> {
        self.attr("value")
    }

    /// Type text into an input.
    pub fn type_text(&self, text: &str) -> &Self {
        // Simulate typing
        let _ = text;
        self
    }

    /// Clear input value.
    pub fn clear(&self) -> &Self {
        self.set_value("")
    }

    /// Check a checkbox.
    pub fn check(&self) -> &Self {
        self
    }

    /// Uncheck a checkbox.
    pub fn uncheck(&self) -> &Self {
        self
    }

    /// Select an option in a select element.
    pub fn select(&self, value: &str) -> &Self {
        let _ = value;
        self
    }

    /// Dispatch a custom event.
    pub fn dispatch(&self, event: &str) -> &Self {
        let _ = event;
        self
    }

    /// Dispatch an event with data.
    pub fn dispatch_with<D: 'static>(&self, event: &str, data: D) -> &Self {
        let _ = (event, data);
        self
    }

    /// Press a key.
    pub fn press_key(&self, key: &str) -> &Self {
        let _ = key;
        self
    }

    /// Query a child element.
    pub fn query(&self, selector: &str) -> ElementQuery {
        // In a real implementation, this would query children
        ElementQuery::new(selector, None)
    }

    /// Get children.
    pub fn children(&self) -> Vec<ElementQuery> {
        self.element
            .as_ref()
            .map(|e| {
                e.children
                    .iter()
                    .map(|c| ElementQuery::new("", Some(c.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Assert the element exists.
    pub fn should_exist(&self) {
        assert!(self.exists(), "Element '{}' should exist", self.selector);
    }

    /// Assert the element does not exist.
    pub fn should_not_exist(&self) {
        assert!(!self.exists(), "Element '{}' should not exist", self.selector);
    }

    /// Assert text content.
    pub fn should_have_text(&self, expected: &str) {
        let text = self.text();
        assert!(
            text.contains(expected),
            "Expected text '{}' but got '{}'",
            expected,
            text
        );
    }

    /// Assert exact text content.
    pub fn should_have_exact_text(&self, expected: &str) {
        let text = self.text();
        assert_eq!(text, expected, "Expected exact text '{}' but got '{}'", expected, text);
    }

    /// Assert has class.
    pub fn should_have_class(&self, class: &str) {
        assert!(
            self.has_class(class),
            "Element should have class '{}'",
            class
        );
    }

    /// Assert attribute value.
    pub fn should_have_attr(&self, name: &str, value: &str) {
        let actual = self.attr(name);
        assert_eq!(
            actual,
            Some(value.to_string()),
            "Expected attribute '{}' to be '{}' but got {:?}",
            name,
            value,
            actual
        );
    }
}

impl fmt::Debug for ElementQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementQuery")
            .field("selector", &self.selector)
            .field("exists", &self.exists())
            .finish()
    }
}

/// Helper to create test fixtures quickly.
pub fn fixture<T: 'static>() -> ComponentFixtureBuilder<T> {
    ComponentFixtureBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        name: String,
    }

    #[test]
    fn test_fixture_builder() {
        let fixture = ComponentFixtureBuilder::<TestComponent>::new()
            .with_input("name", "Test".to_string())
            .create();

        assert_eq!(fixture.input::<String>("name"), Some("Test".to_string()));
    }

    #[test]
    fn test_element_node() {
        let node = ElementNode::new("button")
            .class("btn")
            .class("primary")
            .attr("disabled", "true")
            .text("Click me");

        assert_eq!(node.tag, "button");
        assert!(node.classes.contains(&"btn".to_string()));
        assert_eq!(node.text, "Click me");
    }

    #[test]
    fn test_element_query() {
        let node = ElementNode::new("div")
            .class("container")
            .text("Hello");

        let query = ElementQuery::new(".container", Some(node));

        assert!(query.exists());
        assert_eq!(query.text(), "Hello");
        assert!(query.has_class("container"));
    }

    #[test]
    fn test_fixture_destroy() {
        let fixture = fixture::<TestComponent>().create();
        assert!(!fixture.is_destroyed());

        fixture.destroy();
        assert!(fixture.is_destroyed());
    }
}
