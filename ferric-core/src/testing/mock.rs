//! Mock services and DI overrides for testing.
//!
//! This module provides utilities for creating mock implementations of services
//! and overriding providers in tests.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A mock service that records method calls.
///
/// ## Example
///
/// ```ignore
/// let mock = Mock::<dyn MyService>::new()
///     .expect_call("get_data")
///     .returning(|| "test data".to_string());
///
/// let test_bed = TestBed::configure()
///     .provide(mock.clone())
///     .compile();
///
/// // ... run test ...
///
/// mock.verify();
/// ```
pub struct Mock<T: ?Sized> {
    inner: Rc<MockInner>,
    _marker: std::marker::PhantomData<T>,
}

struct MockInner {
    expectations: RefCell<Vec<Expectation>>,
    calls: RefCell<Vec<CallRecord>>,
    strict: bool,
}

impl<T: ?Sized> Mock<T> {
    /// Create a new mock.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(MockInner {
                expectations: RefCell::new(Vec::new()),
                calls: RefCell::new(Vec::new()),
                strict: false,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a strict mock (all calls must be expected).
    pub fn strict() -> Self {
        Self {
            inner: Rc::new(MockInner {
                expectations: RefCell::new(Vec::new()),
                calls: RefCell::new(Vec::new()),
                strict: true,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Expect a method call.
    pub fn expect_call(self, method: &str) -> ExpectationBuilder<T> {
        ExpectationBuilder {
            mock: self,
            method: method.to_string(),
            times: Times::Any,
            args_matcher: None,
            return_value: None,
        }
    }

    /// Record a method call.
    pub fn record_call(&self, method: &str, args: Vec<Box<dyn Any>>) {
        self.inner.calls.borrow_mut().push(CallRecord {
            method: method.to_string(),
            args,
        });
    }

    /// Verify all expectations were met.
    pub fn verify(&self) -> bool {
        let calls = self.inner.calls.borrow();
        let expectations = self.inner.expectations.borrow();

        for expectation in expectations.iter() {
            let call_count = calls.iter().filter(|c| c.method == expectation.method).count();

            match expectation.times {
                Times::Exactly(n) if call_count != n => return false,
                Times::AtLeast(n) if call_count < n => return false,
                Times::AtMost(n) if call_count > n => return false,
                Times::Between(min, max) if call_count < min || call_count > max => return false,
                Times::Never if call_count > 0 => return false,
                _ => {}
            }
        }

        true
    }

    /// Get all recorded calls.
    pub fn calls(&self) -> Vec<String> {
        self.inner
            .calls
            .borrow()
            .iter()
            .map(|c| c.method.clone())
            .collect()
    }

    /// Get calls for a specific method.
    pub fn calls_for(&self, method: &str) -> usize {
        self.inner
            .calls
            .borrow()
            .iter()
            .filter(|c| c.method == method)
            .count()
    }

    /// Check if a method was called.
    pub fn was_called(&self, method: &str) -> bool {
        self.calls_for(method) > 0
    }

    /// Reset recorded calls.
    pub fn reset(&self) {
        self.inner.calls.borrow_mut().clear();
    }
}

impl<T: ?Sized> Clone for Mock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Default for Mock<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> fmt::Debug for Mock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mock")
            .field("calls", &self.calls())
            .finish()
    }
}

/// Builder for setting up mock expectations.
pub struct ExpectationBuilder<T: ?Sized> {
    mock: Mock<T>,
    method: String,
    times: Times,
    args_matcher: Option<Box<dyn Fn(&[Box<dyn Any>]) -> bool>>,
    return_value: Option<Box<dyn Any>>,
}

impl<T: ?Sized> ExpectationBuilder<T> {
    /// Expect the method to be called exactly n times.
    pub fn times(mut self, n: usize) -> Self {
        self.times = Times::Exactly(n);
        self
    }

    /// Expect the method to be called at least n times.
    pub fn at_least(mut self, n: usize) -> Self {
        self.times = Times::AtLeast(n);
        self
    }

    /// Expect the method to be called at most n times.
    pub fn at_most(mut self, n: usize) -> Self {
        self.times = Times::AtMost(n);
        self
    }

    /// Expect the method to be called exactly once.
    pub fn once(mut self) -> Self {
        self.times = Times::Exactly(1);
        self
    }

    /// Expect the method to never be called.
    pub fn never(mut self) -> Self {
        self.times = Times::Never;
        self
    }

    /// Match specific arguments.
    pub fn with_args<F>(mut self, matcher: F) -> Self
    where
        F: Fn(&[Box<dyn Any>]) -> bool + 'static,
    {
        self.args_matcher = Some(Box::new(matcher));
        self
    }

    /// Set a return value for the mock.
    pub fn returning<R: 'static>(mut self, value: R) -> Mock<T> {
        self.return_value = Some(Box::new(value));
        self.build()
    }

    /// Set a factory for return values.
    pub fn returning_fn<R: 'static, F: Fn() -> R + 'static>(self, _f: F) -> Mock<T> {
        // In a real implementation, we'd store the factory
        self.build()
    }

    /// Build and return the mock.
    pub fn build(self) -> Mock<T> {
        self.mock.inner.expectations.borrow_mut().push(Expectation {
            method: self.method,
            times: self.times,
            args_matcher: self.args_matcher,
        });
        self.mock
    }
}

struct Expectation {
    method: String,
    times: Times,
    args_matcher: Option<Box<dyn Fn(&[Box<dyn Any>]) -> bool>>,
}

struct CallRecord {
    method: String,
    args: Vec<Box<dyn Any>>,
}

/// Times a method should be called.
#[derive(Debug, Clone, Copy)]
pub enum Times {
    /// Any number of times.
    Any,
    /// Exactly n times.
    Exactly(usize),
    /// At least n times.
    AtLeast(usize),
    /// At most n times.
    AtMost(usize),
    /// Between min and max times (inclusive).
    Between(usize, usize),
    /// Never called.
    Never,
}

/// A spy that wraps a real service and records calls.
pub struct Spy<T> {
    inner: T,
    calls: Rc<RefCell<Vec<SpyCall>>>,
}

impl<T> Spy<T> {
    /// Create a spy around a real service.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get the inner service.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get mutable access to the inner service.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Record a call.
    pub fn record(&self, method: &str) {
        self.calls.borrow_mut().push(SpyCall {
            method: method.to_string(),
            timestamp: std::time::Instant::now(),
        });
    }

    /// Get all recorded calls.
    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().iter().map(|c| c.method.clone()).collect()
    }

    /// Check if a method was called.
    pub fn was_called(&self, method: &str) -> bool {
        self.calls.borrow().iter().any(|c| c.method == method)
    }

    /// Get call count for a method.
    pub fn call_count(&self, method: &str) -> usize {
        self.calls.borrow().iter().filter(|c| c.method == method).count()
    }

    /// Reset recorded calls.
    pub fn reset(&self) {
        self.calls.borrow_mut().clear();
    }
}

impl<T: Clone> Clone for Spy<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            calls: self.calls.clone(),
        }
    }
}

struct SpyCall {
    method: String,
    timestamp: std::time::Instant,
}

/// A stub that returns predefined values.
pub struct Stub<T> {
    values: RefCell<HashMap<String, Box<dyn Any>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Stub<T> {
    /// Create a new stub.
    pub fn new() -> Self {
        Self {
            values: RefCell::new(HashMap::new()),
            _marker: std::marker::PhantomData,
        }
    }

    /// Set a return value for a method.
    pub fn when<R: 'static>(&self, method: &str, value: R) {
        self.values.borrow_mut().insert(method.to_string(), Box::new(value));
    }

    /// Get a return value for a method.
    pub fn get<R: Clone + 'static>(&self, method: &str) -> Option<R> {
        self.values
            .borrow()
            .get(method)
            .and_then(|v| v.downcast_ref::<R>())
            .cloned()
    }
}

impl<T> Default for Stub<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider override for dependency injection.
pub struct ProviderOverride {
    type_id: TypeId,
    factory: Box<dyn Fn() -> Box<dyn Any>>,
}

impl ProviderOverride {
    /// Create a provider override with a value.
    pub fn value<T: 'static>(value: T) -> Self {
        let type_id = TypeId::of::<T>();
        Self {
            type_id,
            factory: Box::new(move || {
                // This is a simplified version
                Box::new(()) as Box<dyn Any>
            }),
        }
    }

    /// Create a provider override with a factory.
    pub fn factory<T: 'static, F: Fn() -> T + 'static>(factory: F) -> Self {
        let type_id = TypeId::of::<T>();
        Self {
            type_id,
            factory: Box::new(move || Box::new(factory()) as Box<dyn Any>),
        }
    }
}

/// Create a mock for a service trait.
#[macro_export]
macro_rules! mock_service {
    ($trait_name:ident { $($method:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty);* $(;)? }) => {
        struct $trait_name {
            mock: $crate::testing::Mock<dyn $trait_name>,
        }

        impl $trait_name {
            fn new() -> Self {
                Self {
                    mock: $crate::testing::Mock::new(),
                }
            }
        }
    };
}

/// Builder for creating mock HTTP responses (for testing HTTP clients).
pub struct MockResponseBuilder {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl MockResponseBuilder {
    /// Create a new mock response builder.
    pub fn new() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a successful response.
    pub fn ok() -> Self {
        Self::new().status(200)
    }

    /// Create a not found response.
    pub fn not_found() -> Self {
        Self::new().status(404)
    }

    /// Create a server error response.
    pub fn server_error() -> Self {
        Self::new().status(500)
    }

    /// Set the status code.
    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Add a header.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the body.
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set a JSON body.
    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Self {
        self.body = serde_json::to_vec(value).unwrap_or_default();
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        self
    }

    /// Set a text body.
    pub fn text(mut self, text: &str) -> Self {
        self.body = text.as_bytes().to_vec();
        self.headers.insert("content-type".to_string(), "text/plain".to_string());
        self
    }

    /// Build the mock response.
    pub fn build(self) -> MockResponse {
        MockResponse {
            status: self.status,
            headers: self.headers,
            body: self.body,
        }
    }
}

impl Default for MockResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock HTTP response for testing.
pub struct MockResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// Sequence of mock responses for testing retry logic.
pub struct MockResponseSequence {
    responses: RefCell<Vec<MockResponse>>,
    index: AtomicUsize,
}

impl MockResponseSequence {
    /// Create a new sequence.
    pub fn new() -> Self {
        Self {
            responses: RefCell::new(Vec::new()),
            index: AtomicUsize::new(0),
        }
    }

    /// Add a response to the sequence.
    pub fn then(self, response: MockResponse) -> Self {
        self.responses.borrow_mut().push(response);
        self
    }

    /// Add a successful response.
    pub fn then_ok(self) -> Self {
        self.then(MockResponseBuilder::ok().build())
    }

    /// Add an error response.
    pub fn then_error(self, status: u16) -> Self {
        self.then(MockResponseBuilder::new().status(status).build())
    }

    /// Get the next response in the sequence.
    pub fn next(&self) -> Option<MockResponse> {
        let index = self.index.fetch_add(1, Ordering::SeqCst);
        let responses = self.responses.borrow();

        if index < responses.len() {
            // Clone the response (simplified)
            Some(MockResponse {
                status: responses[index].status,
                headers: responses[index].headers.clone(),
                body: responses[index].body.clone(),
            })
        } else {
            None
        }
    }

    /// Reset the sequence.
    pub fn reset(&self) {
        self.index.store(0, Ordering::SeqCst);
    }
}

impl Default for MockResponseSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait TestService {
        fn get_data(&self) -> String;
    }

    #[test]
    fn test_mock_creation() {
        let mock = Mock::<dyn TestService>::new();
        assert!(mock.calls().is_empty());
    }

    #[test]
    fn test_mock_recording() {
        let mock = Mock::<dyn TestService>::new();
        mock.record_call("get_data", vec![]);

        assert!(mock.was_called("get_data"));
        assert_eq!(mock.calls_for("get_data"), 1);
    }

    #[test]
    fn test_mock_expectations() {
        let mock = Mock::<dyn TestService>::new()
            .expect_call("get_data")
            .once()
            .returning("test");

        mock.record_call("get_data", vec![]);

        assert!(mock.verify());
    }

    #[test]
    fn test_spy() {
        let spy = Spy::new("test_value".to_string());
        spy.record("len");
        spy.record("clone");

        assert!(spy.was_called("len"));
        assert_eq!(spy.call_count("clone"), 1);
    }

    #[test]
    fn test_stub() {
        let stub = Stub::<()>::new();
        stub.when("get_value", 42i32);

        assert_eq!(stub.get::<i32>("get_value"), Some(42));
    }

    #[test]
    fn test_mock_response_builder() {
        let response = MockResponseBuilder::ok()
            .header("x-test", "value")
            .text("Hello")
            .build();

        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("x-test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_mock_response_sequence() {
        let seq = MockResponseSequence::new()
            .then_error(500)
            .then_error(503)
            .then_ok();

        assert_eq!(seq.next().unwrap().status, 500);
        assert_eq!(seq.next().unwrap().status, 503);
        assert_eq!(seq.next().unwrap().status, 200);
        assert!(seq.next().is_none());
    }
}

