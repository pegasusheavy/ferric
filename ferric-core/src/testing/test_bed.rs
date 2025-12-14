//! TestBed for configuring and creating test modules.
//!
//! The TestBed is the primary API for setting up tests in Ferric applications.

use crate::di::Injector;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The main testing utility for configuring test modules.
///
/// ## Example
///
/// ```ignore
/// let test_bed = TestBed::configure()
///     .component::<MyComponent>()
///     .provide::<dyn MyService>(mock_service)
///     .compile();
/// ```
pub struct TestBed {
    injector: Rc<Injector>,
    providers: HashMap<TypeId, Box<dyn Any>>,
    components: Vec<TypeId>,
    compiled: bool,
}

impl TestBed {
    /// Start configuring a new test module.
    pub fn configure() -> TestBedBuilder {
        TestBedBuilder::new()
    }

    /// Create the TestBed with default configuration.
    pub fn new() -> Self {
        Self {
            injector: Rc::new(Injector::root()),
            providers: HashMap::new(),
            components: Vec::new(),
            compiled: false,
        }
    }

    /// Get the test injector.
    pub fn injector(&self) -> &Rc<Injector> {
        &self.injector
    }

    /// Resolve a service from the test injector.
    pub fn get<T: 'static>(&self) -> Option<Rc<T>> {
        self.injector.resolve::<T>()
    }

    /// Resolve an optional service (same as get).
    pub fn get_optional<T: 'static>(&self) -> Option<Rc<T>> {
        self.injector.resolve::<T>()
    }

    /// Resolve a required service (panics if not found).
    pub fn get_required<T: 'static>(&self) -> Rc<T> {
        self.injector.resolve_required::<T>()
    }

    /// Create a component fixture for testing.
    pub fn create_component<T: 'static>(&self) -> ComponentFixture<T> {
        ComponentFixture::new(self.injector.clone())
    }

    /// Reset the TestBed state.
    pub fn reset(&mut self) {
        self.providers.clear();
        self.components.clear();
        self.compiled = false;
    }

    /// Override a provider for testing.
    pub fn override_provider<T: 'static>(&mut self, provider: T) {
        self.providers.insert(TypeId::of::<T>(), Box::new(provider));
    }
}

impl Default for TestBed {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring a TestBed.
pub struct TestBedBuilder {
    providers: Vec<ProviderOverride>,
    components: Vec<TypeId>,
    imports: Vec<Box<dyn Any>>,
    declarations: Vec<TypeId>,
    schemas: Vec<TestSchema>,
}

impl TestBedBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            components: Vec::new(),
            imports: Vec::new(),
            declarations: Vec::new(),
            schemas: Vec::new(),
        }
    }

    /// Declare a component for testing.
    pub fn component<T: 'static>(mut self) -> Self {
        self.components.push(TypeId::of::<T>());
        self
    }

    /// Add a provider to the test module.
    pub fn provide<T: 'static>(mut self, value: T) -> Self {
        self.providers.push(ProviderOverride {
            type_id: TypeId::of::<T>(),
            value: Box::new(value),
        });
        self
    }

    /// Add a provider using a factory function.
    pub fn provide_factory<T: 'static, F: Fn() -> T + 'static>(mut self, factory: F) -> Self {
        self.providers.push(ProviderOverride {
            type_id: TypeId::of::<T>(),
            value: Box::new(FactoryProvider { factory }),
        });
        self
    }

    /// Use a mock value for a service.
    pub fn use_mock<T: 'static>(self, mock: T) -> Self {
        self.provide(mock)
    }

    /// Import a module configuration.
    pub fn import<M: 'static>(mut self, module: M) -> Self {
        self.imports.push(Box::new(module));
        self
    }

    /// Declare multiple components.
    pub fn declarations<T: ComponentList>(mut self) -> Self {
        self.declarations.extend(T::type_ids());
        self
    }

    /// Add a schema for custom element validation.
    pub fn schema(mut self, schema: TestSchema) -> Self {
        self.schemas.push(schema);
        self
    }

    /// Compile the test module and return a TestBed.
    pub fn compile(self) -> TestBed {
        let injector = Rc::new(Injector::root());

        // Register providers
        for _provider in &self.providers {
            // In a real implementation, we'd register these with the injector
        }

        TestBed {
            injector,
            providers: self.providers.into_iter().map(|p| (p.type_id, p.value)).collect(),
            components: self.components,
            compiled: true,
        }
    }

    /// Compile asynchronously (for modules with async providers).
    pub async fn compile_async(self) -> TestBed {
        // For now, same as sync
        self.compile()
    }
}

impl Default for TestBedBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct ProviderOverride {
    type_id: TypeId,
    value: Box<dyn Any>,
}

struct FactoryProvider<F> {
    factory: F,
}

/// Trait for listing component types.
pub trait ComponentList {
    fn type_ids() -> Vec<TypeId>;
}

// Implement for tuples
impl ComponentList for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
}

impl<A: 'static> ComponentList for (A,) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }
}

impl<A: 'static, B: 'static> ComponentList for (A, B) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }
}

impl<A: 'static, B: 'static, C: 'static> ComponentList for (A, B, C) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()]
    }
}

/// Schema for validating custom elements in tests.
#[derive(Debug, Clone, Copy)]
pub enum TestSchema {
    /// Allow all custom elements.
    CustomElements,
    /// Strict HTML validation.
    StrictHtml,
    /// No validation.
    NoErrors,
}

/// Component fixture reference.
pub struct ComponentFixture<T> {
    injector: Rc<Injector>,
    component: RefCell<Option<T>>,
    element: RefCell<Option<TestElement>>,
    change_detector: RefCell<ChangeDetectorState>,
    inputs: RefCell<HashMap<String, Box<dyn Any>>>,
    outputs: RefCell<HashMap<String, Vec<Box<dyn Any>>>>,
}

impl<T: 'static> ComponentFixture<T> {
    /// Create a new fixture.
    pub(crate) fn new(injector: Rc<Injector>) -> Self {
        Self {
            injector,
            component: RefCell::new(None),
            element: RefCell::new(None),
            change_detector: RefCell::new(ChangeDetectorState::default()),
            inputs: RefCell::new(HashMap::new()),
            outputs: RefCell::new(HashMap::new()),
        }
    }

    /// Get a reference to the component instance.
    pub fn component(&self) -> std::cell::Ref<'_, Option<T>> {
        self.component.borrow()
    }

    /// Get a mutable reference to the component instance.
    pub fn component_mut(&self) -> std::cell::RefMut<'_, Option<T>> {
        self.component.borrow_mut()
    }

    /// Get the native element.
    pub fn native_element(&self) -> Option<TestElement> {
        self.element.borrow().clone()
    }

    /// Get the debug element for querying.
    pub fn debug_element(&self) -> DebugElement {
        DebugElement::new(self.element.borrow().clone())
    }

    /// Set an input property on the component.
    pub fn set_input<V: 'static>(&self, name: &str, value: V) {
        self.inputs.borrow_mut().insert(name.to_string(), Box::new(value));
    }

    /// Get an input value.
    pub fn get_input<V: Clone + 'static>(&self, name: &str) -> Option<V> {
        self.inputs
            .borrow()
            .get(name)
            .and_then(|v| v.downcast_ref::<V>())
            .cloned()
    }

    /// Subscribe to an output event.
    pub fn subscribe_output<V: 'static>(&self, name: &str) -> OutputSubscription<V> {
        OutputSubscription {
            name: name.to_string(),
            events: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Trigger change detection.
    pub fn detect_changes(&self) {
        self.change_detector.borrow_mut().detect_changes();
    }

    /// Trigger change detection and wait for stability.
    pub async fn when_stable(&self) {
        // Wait for async operations to complete
        self.detect_changes();
    }

    /// Query for an element by selector.
    pub fn query(&self, selector: &str) -> QueryResult {
        QueryResult::new(selector, self.element.borrow().clone())
    }

    /// Query for all elements matching a selector.
    pub fn query_all(&self, _selector: &str) -> Vec<QueryResult> {
        // In a real implementation, this would query the DOM
        Vec::new()
    }

    /// Check if the fixture is stable (no pending async operations).
    pub fn is_stable(&self) -> bool {
        self.change_detector.borrow().is_stable()
    }

    /// Destroy the fixture and clean up.
    pub fn destroy(&self) {
        *self.component.borrow_mut() = None;
        *self.element.borrow_mut() = None;
    }

    /// Auto-detect changes on input modifications.
    pub fn auto_detect_changes(&self, enabled: bool) {
        self.change_detector.borrow_mut().auto_detect = enabled;
    }
}

/// Output subscription for testing events.
pub struct OutputSubscription<V> {
    name: String,
    events: Vec<V>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: Clone> OutputSubscription<V> {
    /// Get all emitted values.
    pub fn values(&self) -> &[V] {
        &self.events
    }

    /// Get the last emitted value.
    pub fn last(&self) -> Option<&V> {
        self.events.last()
    }

    /// Clear recorded events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Check if any events were emitted.
    pub fn was_called(&self) -> bool {
        !self.events.is_empty()
    }

    /// Get the call count.
    pub fn call_count(&self) -> usize {
        self.events.len()
    }
}

#[derive(Default)]
struct ChangeDetectorState {
    auto_detect: bool,
    stable: bool,
    pending_tasks: usize,
}

impl ChangeDetectorState {
    fn detect_changes(&mut self) {
        // Trigger change detection cycle
        self.stable = self.pending_tasks == 0;
    }

    fn is_stable(&self) -> bool {
        self.stable
    }
}

/// Test element wrapper.
#[derive(Clone)]
pub struct TestElement {
    tag_name: String,
    attributes: HashMap<String, String>,
    children: Vec<TestElement>,
    text_content: String,
    classes: Vec<String>,
}

impl TestElement {
    /// Create a new test element.
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            text_content: String::new(),
            classes: Vec::new(),
        }
    }

    /// Get the tag name.
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Get an attribute value.
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set an attribute.
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Get text content.
    pub fn text_content(&self) -> &str {
        &self.text_content
    }

    /// Check if has a class.
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.contains(&class.to_string())
    }

    /// Get all classes.
    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    /// Get children.
    pub fn children(&self) -> &[TestElement] {
        &self.children
    }
}

impl Default for TestElement {
    fn default() -> Self {
        Self::new("div")
    }
}

/// Debug element for testing queries.
#[derive(Clone)]
pub struct DebugElement {
    element: Option<TestElement>,
}

impl DebugElement {
    /// Create a new debug element.
    pub fn new(element: Option<TestElement>) -> Self {
        Self { element }
    }

    /// Query for a child element.
    pub fn query(&self, selector: &str) -> QueryResult {
        QueryResult::new(selector, self.element.clone())
    }

    /// Query for all matching child elements.
    pub fn query_all(&self, _selector: &str) -> Vec<QueryResult> {
        Vec::new()
    }

    /// Get the native element.
    pub fn native_element(&self) -> Option<&TestElement> {
        self.element.as_ref()
    }

    /// Get children debug elements.
    pub fn children(&self) -> Vec<DebugElement> {
        self.element
            .as_ref()
            .map(|e| {
                e.children()
                    .iter()
                    .map(|c| DebugElement::new(Some(c.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Query result for element assertions.
#[derive(Clone)]
pub struct QueryResult {
    selector: String,
    element: Option<TestElement>,
}

impl QueryResult {
    /// Create a new query result.
    pub fn new(selector: &str, element: Option<TestElement>) -> Self {
        Self {
            selector: selector.to_string(),
            element,
        }
    }

    /// Check if element exists.
    pub fn exists(&self) -> bool {
        self.element.is_some()
    }

    /// Get text content.
    pub fn text(&self) -> String {
        self.element
            .as_ref()
            .map(|e| e.text_content().to_string())
            .unwrap_or_default()
    }

    /// Get an attribute value.
    pub fn attribute(&self, name: &str) -> Option<String> {
        self.element.as_ref().and_then(|e| e.get_attribute(name).cloned())
    }

    /// Check if has a class.
    pub fn has_class(&self, class: &str) -> bool {
        self.element.as_ref().map(|e| e.has_class(class)).unwrap_or(false)
    }

    /// Get the native element.
    pub fn native(&self) -> Option<&TestElement> {
        self.element.as_ref()
    }

    /// Trigger a click event.
    pub fn click(&self) {
        // Dispatch click event
    }

    /// Set input value.
    pub fn set_value(&self, _value: &str) {
        // Set input value and dispatch input event
    }

    /// Get input value.
    pub fn value(&self) -> Option<String> {
        self.attribute("value")
    }

    /// Focus the element.
    pub fn focus(&self) {
        // Dispatch focus event
    }

    /// Blur the element.
    pub fn blur(&self) {
        // Dispatch blur event
    }

    /// Dispatch a custom event.
    pub fn dispatch_event(&self, _event_name: &str) {
        // Dispatch event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_bed_creation() {
        let test_bed = TestBed::new();
        assert!(!test_bed.compiled);
    }

    #[test]
    fn test_test_bed_builder() {
        let test_bed = TestBed::configure()
            .provide(42i32)
            .schema(TestSchema::CustomElements)
            .compile();

        assert!(test_bed.compiled);
    }

    #[test]
    fn test_query_result() {
        let element = TestElement::new("div");
        let result = QueryResult::new(".test", Some(element));

        assert!(result.exists());
    }
}
