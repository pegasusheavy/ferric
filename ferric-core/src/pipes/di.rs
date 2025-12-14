//! Dependency Injection for Pipes.

use crate::di::{Injectable, Injector, MultiToken};
use super::Pipe;
use std::rc::Rc;

/// Token for registering pipes.
pub const PIPES: MultiToken<Rc<dyn Pipe>> =
    MultiToken::with_id("PIPES", 4001);

/// DI-aware pipe registry service.
pub struct InjectablePipeRegistry {
    pipes: std::cell::RefCell<std::collections::HashMap<String, Rc<dyn Pipe>>>,
}

impl Default for InjectablePipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InjectablePipeRegistry {
    pub fn new() -> Self {
        Self {
            pipes: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    pub fn register(&self, name: impl Into<String>, pipe: Rc<dyn Pipe>) {
        self.pipes.borrow_mut().insert(name.into(), pipe);
    }

    pub fn get(&self, name: &str) -> Option<Rc<dyn Pipe>> {
        self.pipes.borrow().get(name).cloned()
    }
}

impl Injectable for InjectablePipeRegistry {
    fn create(_injector: &Injector) -> Self {
        Self::new()
    }
}

/// Helper to register pipes with DI.
pub struct PipesModule;

impl PipesModule {
    pub fn provide(injector: &Injector) {
        injector.register_singleton::<InjectablePipeRegistry>();
    }
}

