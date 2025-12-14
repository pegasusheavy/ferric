//! Pipe registry for storing and retrieving pipes.

use super::async_pipe::AsyncPipe;
use super::builtin::*;
use super::transform::Pipe;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Thread-local pipe registry.
thread_local! {
    static PIPE_REGISTRY: RefCell<HashMap<String, Arc<dyn Pipe>>> = RefCell::new(HashMap::new());
    static INITIALIZED: RefCell<bool> = const { RefCell::new(false) };
}

/// Pipe registry for managing available pipes.
pub struct PipeRegistry;

impl PipeRegistry {
    /// Register a pipe.
    pub fn register(pipe: Arc<dyn Pipe>) {
        PIPE_REGISTRY.with(|registry| {
            registry.borrow_mut().insert(pipe.name().to_string(), pipe);
        });
    }

    /// Get a pipe by name.
    pub fn get(name: &str) -> Option<Arc<dyn Pipe>> {
        ensure_initialized();
        PIPE_REGISTRY.with(|registry| registry.borrow().get(name).cloned())
    }

    /// Check if a pipe exists.
    pub fn has(name: &str) -> bool {
        ensure_initialized();
        PIPE_REGISTRY.with(|registry| registry.borrow().contains_key(name))
    }

    /// Get all registered pipe names.
    pub fn names() -> Vec<String> {
        ensure_initialized();
        PIPE_REGISTRY.with(|registry| registry.borrow().keys().cloned().collect())
    }

    /// Clear all pipes (mainly for testing).
    pub fn clear() {
        PIPE_REGISTRY.with(|registry| registry.borrow_mut().clear());
        INITIALIZED.with(|init| *init.borrow_mut() = false);
    }
}

/// Register a pipe globally.
pub fn register_pipe(pipe: Arc<dyn Pipe>) {
    PipeRegistry::register(pipe);
}

/// Get a pipe by name.
pub fn get_pipe(name: &str) -> Option<Arc<dyn Pipe>> {
    PipeRegistry::get(name)
}

/// Ensure builtin pipes are registered.
fn ensure_initialized() {
    INITIALIZED.with(|init| {
        if !*init.borrow() {
            register_builtin_pipes();
            *init.borrow_mut() = true;
        }
    });
}

/// Register all built-in pipes.
pub fn register_builtin_pipes() {
    // Text pipes
    register_pipe(Arc::new(UppercasePipe));
    register_pipe(Arc::new(LowercasePipe));
    register_pipe(Arc::new(TitlecasePipe));
    register_pipe(Arc::new(TrimPipe));
    register_pipe(Arc::new(SlicePipe));

    // Number pipes
    register_pipe(Arc::new(NumberPipe));
    register_pipe(Arc::new(CurrencyPipe));
    register_pipe(Arc::new(PercentPipe));

    // Date pipes
    register_pipe(Arc::new(DatePipe));

    // Utility pipes
    register_pipe(Arc::new(JsonPipe));
    register_pipe(Arc::new(DefaultPipe));
    register_pipe(Arc::new(ReplacePipe));
    register_pipe(Arc::new(PadStartPipe));
    register_pipe(Arc::new(PadEndPipe));
    register_pipe(Arc::new(TruncatePipe));
    register_pipe(Arc::new(ReversePipe));
    register_pipe(Arc::new(RepeatPipe));
    register_pipe(Arc::new(JoinPipe));
    register_pipe(Arc::new(SplitPipe));
    register_pipe(Arc::new(KeyvaluePipe));

    // Async pipes
    register_pipe(Arc::new(AsyncPipe));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_get_builtin() {
        let uppercase = get_pipe("uppercase");
        assert!(uppercase.is_some());
    }

    #[test]
    fn test_registry_all_builtins() {
        let names = PipeRegistry::names();
        assert!(names.contains(&"uppercase".to_string()));
        assert!(names.contains(&"lowercase".to_string()));
        assert!(names.contains(&"date".to_string()));
        assert!(names.contains(&"currency".to_string()));
    }

    #[test]
    fn test_pipe_execution() {
        let pipe = get_pipe("uppercase").unwrap();
        let result = pipe.transform("hello", &super::super::transform::PipeArgs::new());
        assert_eq!(result, "HELLO");
    }
}
