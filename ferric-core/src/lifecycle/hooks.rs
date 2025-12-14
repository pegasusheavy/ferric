//! Lifecycle hook traits and utilities.
//!
//! This module provides both sync and async variants of lifecycle hooks,
//! as well as utilities for hook execution.

use std::future::Future;
use std::pin::Pin;

/// Result type for lifecycle hooks that can fail.
pub type HookResult<T = ()> = Result<T, HookError>;

/// Error type for lifecycle hook failures.
#[derive(Debug, Clone)]
pub struct HookError {
    /// The hook that failed.
    pub hook: &'static str,
    /// Error message.
    pub message: String,
}

impl HookError {
    /// Create a new hook error.
    pub fn new(hook: &'static str, message: impl Into<String>) -> Self {
        Self {
            hook,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} hook failed: {}", self.hook, self.message)
    }
}

impl std::error::Error for HookError {}

/// Async lifecycle hooks for components that need to perform async operations.
pub trait AsyncLifecycle {
    /// Async version of on_init.
    fn on_init_async(&mut self) -> Pin<Box<dyn Future<Output = HookResult>>> {
        Box::pin(async { Ok(()) })
    }

    /// Async version of after_view_init.
    fn after_view_init_async(&mut self) -> Pin<Box<dyn Future<Output = HookResult>>> {
        Box::pin(async { Ok(()) })
    }

    /// Async version of after_content_init.
    fn after_content_init_async(&mut self) -> Pin<Box<dyn Future<Output = HookResult>>> {
        Box::pin(async { Ok(()) })
    }

    /// Async version of on_destroy.
    fn on_destroy_async(&mut self) -> Pin<Box<dyn Future<Output = HookResult>>> {
        Box::pin(async { Ok(()) })
    }
}

/// Trait for components that can handle errors in lifecycle hooks.
pub trait ErrorBoundary {
    /// Called when a lifecycle hook fails.
    fn on_error(&mut self, error: HookError);

    /// Called when a child component's lifecycle hook fails.
    fn on_child_error(&mut self, error: HookError) {
        // Default: propagate to own error handler
        self.on_error(error);
    }
}

/// Trait for components that need to handle navigation events.
pub trait NavigationLifecycle {
    /// Called when navigating away from this component.
    /// Return `false` to prevent navigation (e.g., for unsaved changes).
    fn can_deactivate(&self) -> bool {
        true
    }

    /// Called when navigating to this component.
    /// Return `false` to prevent navigation.
    fn can_activate(&self) -> bool {
        true
    }

    /// Called after successful navigation to this component.
    fn on_activate(&mut self) {}

    /// Called after successful navigation away from this component.
    fn on_deactivate(&mut self) {}
}

/// Trait for components that need to handle visibility changes.
pub trait VisibilityLifecycle {
    /// Called when the component becomes visible.
    fn on_visible(&mut self) {}

    /// Called when the component becomes hidden.
    fn on_hidden(&mut self) {}

    /// Called when the component enters the viewport.
    fn on_enter_viewport(&mut self) {}

    /// Called when the component leaves the viewport.
    fn on_leave_viewport(&mut self) {}
}

/// Trait for components that need to handle focus events.
pub trait FocusLifecycle {
    /// Called when the component receives focus.
    fn on_focus(&mut self) {}

    /// Called when the component loses focus.
    fn on_blur(&mut self) {}
}

/// Trait for components that need to handle resize events.
pub trait ResizeLifecycle {
    /// Called when the component's size changes.
    fn on_resize(&mut self, width: f64, height: f64);
}

/// Trait for components that need to handle window/document events.
pub trait DocumentLifecycle {
    /// Called when the window is resized.
    fn on_window_resize(&mut self, _width: f64, _height: f64) {}

    /// Called when the window scrolls.
    fn on_window_scroll(&mut self, _scroll_x: f64, _scroll_y: f64) {}

    /// Called when the document visibility changes.
    fn on_visibility_change(&mut self, _visible: bool) {}

    /// Called before the page is unloaded.
    fn on_before_unload(&self) -> Option<String> {
        None
    }
}

/// Builder for configuring lifecycle hook execution.
pub struct LifecycleHookBuilder<T> {
    component: T,
    skip_init: bool,
    skip_view_init: bool,
    skip_content_init: bool,
}

impl<T: super::Lifecycle> LifecycleHookBuilder<T> {
    /// Create a new builder for a component.
    pub fn new(component: T) -> Self {
        Self {
            component,
            skip_init: false,
            skip_view_init: false,
            skip_content_init: false,
        }
    }

    /// Skip the on_init hook.
    pub fn skip_init(mut self) -> Self {
        self.skip_init = true;
        self
    }

    /// Skip the after_view_init hook.
    pub fn skip_view_init(mut self) -> Self {
        self.skip_view_init = true;
        self
    }

    /// Skip the after_content_init hook.
    pub fn skip_content_init(mut self) -> Self {
        self.skip_content_init = true;
        self
    }

    /// Run the configured hooks and return the component.
    pub fn run(mut self) -> T {
        if !self.skip_init {
            self.component.on_init();
        }
        if !self.skip_view_init {
            self.component.after_view_init();
        }
        if !self.skip_content_init {
            self.component.after_content_init();
        }
        self.component
    }

    /// Get a reference to the inner component.
    pub fn component(&self) -> &T {
        &self.component
    }

    /// Get a mutable reference to the inner component.
    pub fn component_mut(&mut self) -> &mut T {
        &mut self.component
    }
}

/// Macro to implement all lifecycle traits with default implementations.
#[macro_export]
macro_rules! impl_lifecycle_defaults {
    ($type:ty) => {
        impl $crate::lifecycle::Lifecycle for $type {}
    };
}

/// Macro to implement lifecycle with custom hooks.
#[macro_export]
macro_rules! impl_lifecycle {
    ($type:ty, {
        $(on_init: $init:expr,)?
        $(after_view_init: $view_init:expr,)?
        $(after_content_init: $content_init:expr,)?
        $(on_changes: $changes:expr,)?
        $(do_check: $check:expr,)?
        $(on_destroy: $destroy:expr,)?
    }) => {
        impl $crate::lifecycle::Lifecycle for $type {
            $(
                fn on_init(&mut self) {
                    ($init)(self)
                }
            )?
            $(
                fn after_view_init(&mut self) {
                    ($view_init)(self)
                }
            )?
            $(
                fn after_content_init(&mut self) {
                    ($content_init)(self)
                }
            )?
            $(
                fn on_changes(&mut self, changes: &$crate::lifecycle::Changes) {
                    ($changes)(self, changes)
                }
            )?
            $(
                fn do_check(&mut self) {
                    ($check)(self)
                }
            )?
            $(
                fn on_destroy(&mut self) {
                    ($destroy)(self)
                }
            )?
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::Lifecycle;

    struct TestComponent {
        initialized: bool,
        view_ready: bool,
    }

    impl Lifecycle for TestComponent {
        fn on_init(&mut self) {
            self.initialized = true;
        }

        fn after_view_init(&mut self) {
            self.view_ready = true;
        }
    }

    #[test]
    fn test_hook_builder() {
        let component = TestComponent {
            initialized: false,
            view_ready: false,
        };

        let component = LifecycleHookBuilder::new(component).run();

        assert!(component.initialized);
        assert!(component.view_ready);
    }

    #[test]
    fn test_hook_builder_skip() {
        let component = TestComponent {
            initialized: false,
            view_ready: false,
        };

        let component = LifecycleHookBuilder::new(component)
            .skip_init()
            .run();

        assert!(!component.initialized);
        assert!(component.view_ready);
    }
}

