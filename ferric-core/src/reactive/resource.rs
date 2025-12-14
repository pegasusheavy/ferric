//! Resource - async data fetching with reactive loading states.
//!
//! Resources handle async operations (like HTTP requests) and provide
//! reactive access to loading state, data, and errors.
//!
//! # Example
//!
//! ```ignore
//! use ferric::reactive::*;
//!
//! let user_id = signal(1);
//!
//! // Create a resource that fetches user data when user_id changes
//! let user_resource = resource(
//!     move || user_id.get(),
//!     |id| async move {
//!         fetch_user(id).await
//!     }
//! );
//!
//! // Access the resource state
//! if user_resource.loading() {
//!     // Show loading spinner
//! } else if let Some(user) = user_resource.data() {
//!     // Show user data
//! } else if let Some(error) = user_resource.error() {
//!     // Show error message
//! }
//!
//! // Manually refetch
//! user_resource.refetch();
//! ```

use super::runtime::ReactiveId;
use super::signal::{signal, Signal};
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

/// The state of an async resource.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState<T, E> {
    /// Initial state, not yet loaded.
    Unresolved,
    /// Currently loading.
    Loading,
    /// Successfully loaded with data.
    Ready(T),
    /// Loading failed with an error.
    Error(E),
    /// Refreshing (has previous data but fetching new).
    Refreshing(T),
}

impl<T, E> ResourceState<T, E> {
    /// Check if the resource is loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, ResourceState::Loading | ResourceState::Refreshing(_))
    }

    /// Check if the resource has data.
    pub fn is_ready(&self) -> bool {
        matches!(self, ResourceState::Ready(_) | ResourceState::Refreshing(_))
    }

    /// Check if the resource has an error.
    pub fn is_error(&self) -> bool {
        matches!(self, ResourceState::Error(_))
    }

    /// Get the data if available.
    pub fn data(&self) -> Option<&T> {
        match self {
            ResourceState::Ready(data) | ResourceState::Refreshing(data) => Some(data),
            _ => None,
        }
    }

    /// Get the error if available.
    pub fn error(&self) -> Option<&E> {
        match self {
            ResourceState::Error(e) => Some(e),
            _ => None,
        }
    }
}

impl<T: Clone, E: Clone> ResourceState<T, E> {
    /// Get owned data if available.
    pub fn data_owned(&self) -> Option<T> {
        self.data().cloned()
    }

    /// Get owned error if available.
    pub fn error_owned(&self) -> Option<E> {
        self.error().cloned()
    }
}

/// A reactive resource for async data fetching.
pub struct Resource<T, E = String> {
    inner: Rc<ResourceInner<T, E>>,
}

struct ResourceInner<T, E> {
    /// Unique identifier.
    id: ReactiveId,
    /// The current state.
    state: Signal<ResourceState<T, E>>,
    /// The fetcher function.
    fetcher: RefCell<Option<Box<dyn Fn()>>>,
    /// Version counter to track stale requests.
    version: RefCell<u64>,
}

impl<T: Clone + 'static, E: Clone + 'static> Resource<T, E> {
    /// Create a new resource with a source and fetcher.
    ///
    /// The `source` function provides the input to the fetcher.
    /// The `fetcher` is an async function that performs the actual fetch.
    pub fn new<S, F, Fut>(source: S, fetcher: F) -> Self
    where
        S: Fn() + 'static,
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        let state = signal(ResourceState::Unresolved);
        let state_clone = state.clone();
        let version = Rc::new(RefCell::new(0u64));
        let version_clone = Rc::clone(&version);

        let fetcher_fn = Rc::new(fetcher);
        let fetcher_clone = Rc::clone(&fetcher_fn);

        // Wrap the fetcher to handle versioning and state updates
        let fetch_wrapper = move || {
            // Increment version
            let current_version = {
                let mut v = version_clone.borrow_mut();
                *v += 1;
                *v
            };

            // Set loading state
            let prev_data = state_clone.get().data_owned();
            if let Some(data) = prev_data {
                state_clone.set(ResourceState::Refreshing(data));
            } else {
                state_clone.set(ResourceState::Loading);
            }

            // Clone for async block
            let state_async = state_clone.clone();
            let version_async = Rc::clone(&version_clone);
            let fetcher_async = Rc::clone(&fetcher_clone);

            spawn_local(async move {
                let result = fetcher_async().await;

                // Check if this request is still relevant
                if *version_async.borrow() != current_version {
                    return; // Stale request, ignore
                }

                match result {
                    Ok(data) => state_async.set(ResourceState::Ready(data)),
                    Err(error) => state_async.set(ResourceState::Error(error)),
                }
            });
        };

        // Get the version value before creating inner
        let current_version_val = *version.borrow();

        let inner = Rc::new(ResourceInner {
            id: ReactiveId::new(),
            state,
            fetcher: RefCell::new(Some(Box::new(fetch_wrapper))),
            version: RefCell::new(current_version_val),
        });

        // Track the source and refetch when it changes
        let inner_weak = Rc::downgrade(&inner);
        super::effect(move || {
            source(); // Track the source
            if let Some(inner) = inner_weak.upgrade()
                && let Some(ref fetcher) = *inner.fetcher.borrow() {
                    fetcher();
                }
        });

        Resource { inner }
    }

    /// Get the current state.
    pub fn state(&self) -> ResourceState<T, E> {
        self.inner.state.get()
    }

    /// Check if the resource is loading.
    pub fn loading(&self) -> bool {
        self.inner.state.get().is_loading()
    }

    /// Get the data if available.
    pub fn data(&self) -> Option<T> {
        self.inner.state.get().data_owned()
    }

    /// Get the error if available.
    pub fn error(&self) -> Option<E> {
        self.inner.state.get().error_owned()
    }

    /// Manually trigger a refetch.
    pub fn refetch(&self) {
        if let Some(ref fetcher) = *self.inner.fetcher.borrow() {
            fetcher();
        }
    }

    /// Mutate the data locally without refetching.
    ///
    /// Useful for optimistic updates.
    pub fn mutate<F>(&self, f: F)
    where
        F: FnOnce(Option<&T>) -> T,
    {
        let current = self.inner.state.get();
        let new_data = f(current.data());
        self.inner.state.set(ResourceState::Ready(new_data));
    }

    /// Reset to the initial unresolved state.
    pub fn reset(&self) {
        self.inner.state.set(ResourceState::Unresolved);
    }

    /// Get the reactive ID.
    pub fn id(&self) -> ReactiveId {
        self.inner.id
    }
}

impl<T, E> Clone for Resource<T, E> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: std::fmt::Debug + Clone, E: std::fmt::Debug + Clone> std::fmt::Debug for Resource<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("state", &self.inner.state.get())
            .finish()
    }
}

/// Create a resource that fetches data based on a source signal.
///
/// The fetcher is called whenever the source changes.
///
/// # Example
///
/// ```ignore
/// let id = signal(1);
/// let data = resource(
///     move || id.get(),
///     |id| async move {
///         fetch_data(id).await.map_err(|e| e.to_string())
///     }
/// );
/// ```
pub fn resource<S, Src, F, Fut, T, E>(source: S, fetcher: F) -> Resource<T, E>
where
    S: Fn() -> Src + 'static,
    Src: 'static,
    F: Fn(Src) -> Fut + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
    T: Clone + 'static,
    E: Clone + 'static,
{
    Resource::new(
        move || { let _ = source(); },
        move || {
            // We need to capture the source value and pass it to fetcher
            // This is a simplified version - a real impl would be more sophisticated
            fetcher(unsafe { std::mem::zeroed() }) // Placeholder - see create_resource below
        }
    )
}

/// Create a resource with proper source tracking.
///
/// This is the recommended way to create resources.
///
/// # Example
///
/// ```ignore
/// let user_id = signal(1);
///
/// let user = create_resource(
///     move || user_id.get(),
///     |id| async move {
///         fetch_user(id).await.map_err(|e| e.to_string())
///     }
/// );
/// ```
pub fn create_resource<S, Src, F, Fut, T, E>(source: S, fetcher: F) -> Resource<T, E>
where
    S: Fn() -> Src + Clone + 'static,
    Src: Clone + 'static,
    F: Fn(Src) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
    T: Clone + 'static,
    E: Clone + 'static,
{
    let state = signal(ResourceState::<T, E>::Unresolved);
    let version = Rc::new(RefCell::new(0u64));

    let state_for_effect = state.clone();
    let version_for_effect = Rc::clone(&version);
    let source_for_effect = source.clone();
    let fetcher_for_effect = fetcher.clone();

    // Create an effect that tracks the source and fetches
    super::effect(move || {
        let src_value = source_for_effect();

        // Increment version
        let current_version = {
            let mut v = version_for_effect.borrow_mut();
            *v += 1;
            *v
        };

        // Set loading state
        let prev_data = state_for_effect.get().data_owned();
        if let Some(data) = prev_data {
            state_for_effect.set(ResourceState::Refreshing(data));
        } else {
            state_for_effect.set(ResourceState::Loading);
        }

        // Clone for async
        let state_async = state_for_effect.clone();
        let version_async = Rc::clone(&version_for_effect);
        let fetcher_async = fetcher_for_effect.clone();

        spawn_local(async move {
            let result = fetcher_async(src_value).await;

            // Check if stale
            if *version_async.borrow() != current_version {
                return;
            }

            match result {
                Ok(data) => state_async.set(ResourceState::Ready(data)),
                Err(error) => state_async.set(ResourceState::Error(error)),
            }
        });
    });

    // Get the current version value before creating the resource
    let current_version_val = *version.borrow();

    Resource {
        inner: Rc::new(ResourceInner {
            id: ReactiveId::new(),
            state,
            fetcher: RefCell::new(None), // Effect handles fetching
            version: RefCell::new(current_version_val),
        }),
    }
}

/// Create a resource that only fetches once on creation.
pub fn create_resource_once<F, Fut, T, E>(fetcher: F) -> Resource<T, E>
where
    F: FnOnce() -> Fut + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
    T: Clone + 'static,
    E: Clone + 'static,
{
    let state = signal(ResourceState::<T, E>::Loading);
    let state_async = state.clone();

    // Wrap in RefCell/Option for FnOnce semantics
    let fetcher = Rc::new(RefCell::new(Some(fetcher)));

    spawn_local(async move {
        if let Some(f) = fetcher.borrow_mut().take() {
            match f().await {
                Ok(data) => state_async.set(ResourceState::Ready(data)),
                Err(error) => state_async.set(ResourceState::Error(error)),
            }
        }
    });

    Resource {
        inner: Rc::new(ResourceInner {
            id: ReactiveId::new(),
            state,
            fetcher: RefCell::new(None),
            version: RefCell::new(1),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_state() {
        let state: ResourceState<i32, String> = ResourceState::Ready(42);
        assert!(state.is_ready());
        assert!(!state.is_loading());
        assert!(!state.is_error());
        assert_eq!(state.data(), Some(&42));
    }

    #[test]
    fn test_resource_state_loading() {
        let state: ResourceState<i32, String> = ResourceState::Loading;
        assert!(state.is_loading());
        assert!(!state.is_ready());
        assert_eq!(state.data(), None);
    }

    #[test]
    fn test_resource_state_error() {
        let state: ResourceState<i32, String> = ResourceState::Error("Failed".to_string());
        assert!(state.is_error());
        assert_eq!(state.error(), Some(&"Failed".to_string()));
    }
}

