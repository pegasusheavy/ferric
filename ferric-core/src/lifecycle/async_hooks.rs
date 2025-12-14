//! Async lifecycle hooks for components.

use std::future::Future;
use std::pin::Pin;

/// Async version of OnInit lifecycle hook.
#[async_trait::async_trait(?Send)]
pub trait AsyncOnInit {
    /// Called after component is initialized.
    /// Use for async initialization like fetching data.
    async fn async_on_init(&self) {}
}

/// Async version of AfterViewInit lifecycle hook.
#[async_trait::async_trait(?Send)]
pub trait AsyncAfterViewInit {
    /// Called after the view is initialized.
    /// Use for async operations that depend on the view.
    async fn async_after_view_init(&self) {}
}

/// Async version of OnDestroy lifecycle hook.
#[async_trait::async_trait(?Send)]
pub trait AsyncOnDestroy {
    /// Called before component is destroyed.
    /// Use for async cleanup like closing connections.
    async fn async_on_destroy(&self) {}
}

/// Async data loading lifecycle hook.
#[async_trait::async_trait(?Send)]
pub trait AsyncDataLoader {
    type Data;

    /// Load data asynchronously.
    async fn load_data(&self) -> Result<Self::Data, String>;
}

/// Async validation lifecycle hook.
#[async_trait::async_trait(?Send)]
pub trait AsyncValidator {
    /// Validate component state asynchronously.
    async fn async_validate(&self) -> Result<(), Vec<String>>;
}

/// Helper to execute async lifecycle hooks.
pub struct AsyncLifecycleRunner;

impl AsyncLifecycleRunner {
    /// Run async initialization for a component.
    pub async fn run_init<T: AsyncOnInit>(component: &T) {
        component.async_on_init().await;
    }

    /// Run async after view init for a component.
    pub async fn run_after_view_init<T: AsyncAfterViewInit>(component: &T) {
        component.async_after_view_init().await;
    }

    /// Run async destroy for a component.
    pub async fn run_destroy<T: AsyncOnDestroy>(component: &T) {
        component.async_on_destroy().await;
    }

    /// Load data for a component.
    pub async fn load_data<T: AsyncDataLoader>(component: &T) -> Result<T::Data, String> {
        component.load_data().await
    }

    /// Validate a component.
    pub async fn validate<T: AsyncValidator>(component: &T) -> Result<(), Vec<String>> {
        component.async_validate().await
    }
}

/// Trait for components with async initialization.
pub trait AsyncComponent: AsyncOnInit {
    /// Initialize the component asynchronously.
    fn initialize(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(self.async_on_init())
    }
}

impl<T: AsyncOnInit> AsyncComponent for T {}

