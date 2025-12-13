//! Async pipe for subscribing to promises and async values.

use super::transform::Pipe;
use super::transform::{PipeArgs, PipeValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ASYNC_ID: AtomicU64 = AtomicU64::new(1);

fn next_async_id() -> u64 {
    NEXT_ASYNC_ID.fetch_add(1, Ordering::SeqCst)
}

/// State of an async value.
#[derive(Debug, Clone, PartialEq)]
pub enum AsyncState<T> {
    /// Initial state, no value yet.
    Initial,
    /// Loading/pending state.
    Pending,
    /// Successfully resolved with a value.
    Resolved(T),
    /// Failed with an error.
    Error(String),
}

impl<T> AsyncState<T> {
    /// Check if the state is resolved.
    pub fn is_resolved(&self) -> bool {
        matches!(self, AsyncState::Resolved(_))
    }

    /// Check if the state is pending.
    pub fn is_pending(&self) -> bool {
        matches!(self, AsyncState::Pending)
    }

    /// Check if the state is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, AsyncState::Error(_))
    }

    /// Get the resolved value, if any.
    pub fn value(&self) -> Option<&T> {
        match self {
            AsyncState::Resolved(v) => Some(v),
            _ => None,
        }
    }

    /// Get the error message, if any.
    pub fn error(&self) -> Option<&str> {
        match self {
            AsyncState::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Map the resolved value.
    pub fn map<U, F: FnOnce(&T) -> U>(&self, f: F) -> AsyncState<U> {
        match self {
            AsyncState::Initial => AsyncState::Initial,
            AsyncState::Pending => AsyncState::Pending,
            AsyncState::Resolved(v) => AsyncState::Resolved(f(v)),
            AsyncState::Error(e) => AsyncState::Error(e.clone()),
        }
    }
}

impl<T: Default> Default for AsyncState<T> {
    fn default() -> Self {
        AsyncState::Initial
    }
}

impl<T: ToString> AsyncState<T> {
    /// Convert to a display string with optional loading text.
    pub fn to_display(&self, loading_text: &str) -> String {
        match self {
            AsyncState::Initial => String::new(),
            AsyncState::Pending => loading_text.to_string(),
            AsyncState::Resolved(v) => v.to_string(),
            AsyncState::Error(e) => format!("Error: {}", e),
        }
    }
}

/// A reactive async value that can be subscribed to.
pub struct AsyncValue<T> {
    id: u64,
    state: RefCell<AsyncState<T>>,
    subscribers: RefCell<Vec<Box<dyn Fn(&AsyncState<T>)>>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for AsyncValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncValue")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("subscribers", &format!("[{} subscribers]", self.subscribers.borrow().len()))
            .finish()
    }
}

impl<T> AsyncValue<T> {
    /// Create a new async value in initial state.
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            id: next_async_id(),
            state: RefCell::new(AsyncState::Initial),
            subscribers: RefCell::new(Vec::new()),
        })
    }

    /// Create with an initial pending state.
    pub fn pending() -> Rc<Self> {
        Rc::new(Self {
            id: next_async_id(),
            state: RefCell::new(AsyncState::Pending),
            subscribers: RefCell::new(Vec::new()),
        })
    }

    /// Create with an already resolved value.
    pub fn resolved(value: T) -> Rc<Self> {
        Rc::new(Self {
            id: next_async_id(),
            state: RefCell::new(AsyncState::Resolved(value)),
            subscribers: RefCell::new(Vec::new()),
        })
    }

    /// Get the unique ID of this async value.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the current state.
    pub fn state(&self) -> AsyncState<T>
    where
        T: Clone,
    {
        self.state.borrow().clone()
    }

    /// Set to pending state.
    pub fn set_pending(&self) {
        *self.state.borrow_mut() = AsyncState::Pending;
        self.notify_subscribers();
    }

    /// Set a resolved value.
    pub fn set_value(&self, value: T) {
        *self.state.borrow_mut() = AsyncState::Resolved(value);
        self.notify_subscribers();
    }

    /// Set an error state.
    pub fn set_error(&self, error: String) {
        *self.state.borrow_mut() = AsyncState::Error(error);
        self.notify_subscribers();
    }

    /// Subscribe to state changes.
    pub fn subscribe<F: Fn(&AsyncState<T>) + 'static>(&self, callback: F) {
        self.subscribers.borrow_mut().push(Box::new(callback));
    }

    /// Notify all subscribers of a state change.
    fn notify_subscribers(&self) {
        let state = self.state.borrow();
        for subscriber in self.subscribers.borrow().iter() {
            subscriber(&state);
        }
    }
}

impl<T> Default for AsyncValue<T> {
    fn default() -> Self {
        Self {
            id: next_async_id(),
            state: RefCell::new(AsyncState::Initial),
            subscribers: RefCell::new(Vec::new()),
        }
    }
}

/// Registry for tracking async pipe subscriptions.
pub struct AsyncPipeRegistry {
    /// Map from async value ID to its current string representation.
    cache: std::sync::RwLock<HashMap<String, String>>,
}

impl Default for AsyncPipeRegistry {
    fn default() -> Self {
        Self {
            cache: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl AsyncPipeRegistry {
    /// Get or create a global registry.
    pub fn global() -> &'static Self {
        use std::sync::OnceLock;
        static REGISTRY: OnceLock<AsyncPipeRegistry> = OnceLock::new();
        REGISTRY.get_or_init(AsyncPipeRegistry::default)
    }

    /// Update the cached value for an async source.
    pub fn update(&self, key: &str, value: String) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key.to_string(), value);
        }
    }

    /// Get the cached value for an async source.
    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.read().ok()?.get(key).cloned()
    }

    /// Remove a cached value.
    pub fn remove(&self, key: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key);
        }
    }

    /// Clear all cached values.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

/// The async pipe - subscribes to async values and returns latest result.
pub struct AsyncPipe;

impl Pipe for AsyncPipe {
    fn name(&self) -> &'static str {
        "async"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let loading_text = args.get(0).unwrap_or("");
        let error_prefix = args.get(1).unwrap_or("Error: ");

        if let Some(cached) = AsyncPipeRegistry::global().get(value) {
            return cached;
        }

        if let Ok(state) = serde_json::from_str::<serde_json::Value>(value) {
            if let Some(obj) = state.as_object() {
                if let Some(state_type) = obj.get("state").and_then(|s| s.as_str()) {
                    match state_type {
                        "pending" | "loading" => return loading_text.to_string(),
                        "error" => {
                            let error = obj
                                .get("error")
                                .and_then(|e| e.as_str())
                                .unwrap_or("Unknown error");
                            return format!("{}{}", error_prefix, error);
                        }
                        "resolved" | "success" => {
                            if let Some(val) = obj.get("value") {
                                return match val {
                                    serde_json::Value::String(s) => s.clone(),
                                    _ => val.to_string(),
                                };
                            }
                        }
                        _ => {}
                    }
                }

                if obj.get("loading").and_then(|l| l.as_bool()).unwrap_or(false) {
                    return loading_text.to_string();
                }

                if let Some(error) = obj.get("error").and_then(|e| e.as_str()) {
                    return format!("{}{}", error_prefix, error);
                }

                if let Some(data) = obj.get("data") {
                    return match data {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => loading_text.to_string(),
                        _ => data.to_string(),
                    };
                }

                if let Some(val) = obj.get("value") {
                    return match val {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => loading_text.to_string(),
                        _ => val.to_string(),
                    };
                }
            }
        }

        value.to_string()
    }

    fn transform_value(&self, value: &PipeValue, args: &PipeArgs) -> PipeValue {
        PipeValue::String(self.transform(&value.to_string_value(), args))
    }

    fn is_pure(&self) -> bool {
        false
    }
}

/// Helper function to create an async state JSON for use in templates.
pub fn async_pending() -> String {
    r#"{"state":"pending"}"#.to_string()
}

/// Helper function to create a resolved async state JSON.
pub fn async_resolved<T: serde::Serialize>(value: &T) -> String {
    serde_json::json!({
        "state": "resolved",
        "value": value
    })
    .to_string()
}

/// Helper function to create an error async state JSON.
pub fn async_error(error: &str) -> String {
    serde_json::json!({
        "state": "error",
        "error": error
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_state_resolved() {
        let state: AsyncState<String> = AsyncState::Resolved("Hello".to_string());
        assert!(state.is_resolved());
        assert_eq!(state.value(), Some(&"Hello".to_string()));
    }

    #[test]
    fn test_async_state_pending() {
        let state: AsyncState<String> = AsyncState::Pending;
        assert!(state.is_pending());
        assert_eq!(state.to_display("Loading..."), "Loading...");
    }

    #[test]
    fn test_async_state_error() {
        let state: AsyncState<String> = AsyncState::Error("Network error".to_string());
        assert!(state.is_error());
        assert_eq!(state.error(), Some("Network error"));
    }

    #[test]
    fn test_async_pipe_pending() {
        let pipe = AsyncPipe;
        let input = r#"{"state":"pending"}"#;
        let result = pipe.transform(input, &PipeArgs::from_vec(vec!["Loading...".to_string()]));
        assert_eq!(result, "Loading...");
    }

    #[test]
    fn test_async_pipe_resolved() {
        let pipe = AsyncPipe;
        let input = r#"{"state":"resolved","value":"Hello World"}"#;
        let result = pipe.transform(input, &PipeArgs::new());
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_async_pipe_error() {
        let pipe = AsyncPipe;
        let input = r#"{"state":"error","error":"Failed to fetch"}"#;
        let result = pipe.transform(input, &PipeArgs::new());
        assert_eq!(result, "Error: Failed to fetch");
    }

    #[test]
    fn test_async_value() {
        let value = AsyncValue::<String>::new();

        assert!(matches!(value.state(), AsyncState::Initial));

        value.set_pending();
        assert!(value.state().is_pending());

        value.set_value("Done!".to_string());
        assert_eq!(value.state().value(), Some(&"Done!".to_string()));
    }

    #[test]
    fn test_async_helpers() {
        let pending: serde_json::Value = serde_json::from_str(&async_pending()).unwrap();
        assert_eq!(pending["state"], "pending");

        let resolved: serde_json::Value = serde_json::from_str(&async_resolved(&"Hello")).unwrap();
        assert_eq!(resolved["state"], "resolved");
        assert_eq!(resolved["value"], "Hello");

        let error: serde_json::Value = serde_json::from_str(&async_error("Oops")).unwrap();
        assert_eq!(error["state"], "error");
        assert_eq!(error["error"], "Oops");
    }
}
