//! Promise abstraction for JavaScript interop and async operations.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use wasm_bindgen::prelude::*;

/// State of a Promise.
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState<T, E = JsValue> {
    /// Promise is pending
    Pending,
    /// Promise resolved successfully
    Resolved(T),
    /// Promise rejected with error
    Rejected(E),
}

impl<T, E> PromiseState<T, E> {
    pub fn is_pending(&self) -> bool {
        matches!(self, PromiseState::Pending)
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, PromiseState::Resolved(_))
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, PromiseState::Rejected(_))
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            PromiseState::Resolved(v) => Some(v),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&E> {
        match self {
            PromiseState::Rejected(e) => Some(e),
            _ => None,
        }
    }
}

/// A Promise wrapper for Rust/WASM interop.
pub struct Promise<T, E = JsValue> {
    inner: Rc<RefCell<PromiseInner<T, E>>>,
}

struct PromiseInner<T, E> {
    state: PromiseState<T, E>,
    waker: Option<Waker>,
    callbacks: Vec<Box<dyn Fn(&PromiseState<T, E>)>>,
}

impl<T: Clone, E: Clone> Promise<T, E> {
    /// Create a new pending promise.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(PromiseInner {
                state: PromiseState::Pending,
                waker: None,
                callbacks: Vec::new(),
            })),
        }
    }

    /// Create a resolved promise.
    pub fn resolved(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(PromiseInner {
                state: PromiseState::Resolved(value),
                waker: None,
                callbacks: Vec::new(),
            })),
        }
    }

    /// Create a rejected promise.
    pub fn rejected(error: E) -> Self {
        Self {
            inner: Rc::new(RefCell::new(PromiseInner {
                state: PromiseState::Rejected(error),
                waker: None,
                callbacks: Vec::new(),
            })),
        }
    }

    /// Create a promise from a JavaScript Promise.
    #[cfg(target_arch = "wasm32")]
    pub fn from_js(js_promise: js_sys::Promise) -> Self
    where
        T: From<JsValue> + 'static,
        E: From<JsValue> + 'static,
    {
        let promise = Self::new();
        let promise_clone = promise.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match JsFuture::from(js_promise).await {
                Ok(value) => promise_clone.resolve(T::from(value)),
                Err(err) => promise_clone.reject(E::from(err)),
            }
        });

        promise
    }

    /// Convert to a JavaScript Promise.
    #[cfg(target_arch = "wasm32")]
    pub fn to_js(&self) -> js_sys::Promise
    where
        T: Into<JsValue> + Clone + 'static,
        E: Into<JsValue> + Clone + 'static,
    {
        let promise = self.clone();

        js_sys::Promise::new(&mut |resolve, reject| {
            wasm_bindgen_futures::spawn_local(async move {
                match promise.await {
                    Ok(value) => {
                        let _ = resolve.call1(&JsValue::NULL, &value.into());
                    }
                    Err(error) => {
                        let _ = reject.call1(&JsValue::NULL, &error.into());
                    }
                }
            });
        })
    }

    /// Get the current state.
    pub fn state(&self) -> PromiseState<T, E> {
        self.inner.borrow().state.clone()
    }

    /// Resolve the promise.
    pub fn resolve(&self, value: T) {
        let mut inner = self.inner.borrow_mut();
        if inner.state.is_pending() {
            inner.state = PromiseState::Resolved(value);

            // Notify waker
            if let Some(waker) = inner.waker.take() {
                waker.wake();
            }

            // Call callbacks
            let state = inner.state.clone();
            for callback in &inner.callbacks {
                callback(&state);
            }
        }
    }

    /// Reject the promise.
    pub fn reject(&self, error: E) {
        let mut inner = self.inner.borrow_mut();
        if inner.state.is_pending() {
            inner.state = PromiseState::Rejected(error);

            // Notify waker
            if let Some(waker) = inner.waker.take() {
                waker.wake();
            }

            // Call callbacks
            let state = inner.state.clone();
            for callback in &inner.callbacks {
                callback(&state);
            }
        }
    }

    /// Register a callback for state changes.
    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn(&PromiseState<T, E>) + 'static,
    {
        let mut inner = self.inner.borrow_mut();

        // Call immediately if already settled
        if !inner.state.is_pending() {
            callback(&inner.state);
        }

        inner.callbacks.push(Box::new(callback));
    }

    /// Chain a transformation.
    pub fn then<U, F>(&self, f: F) -> Promise<U, E>
    where
        F: Fn(T) -> U + 'static,
        U: Clone + 'static,
        T: 'static,
        E: 'static,
    {
        let new_promise = Promise::new();
        let new_promise_clone = new_promise.clone();

        self.on_change(move |state| {
            match state {
                PromiseState::Resolved(value) => {
                    new_promise_clone.resolve(f(value.clone()));
                }
                PromiseState::Rejected(error) => {
                    new_promise_clone.reject(error.clone());
                }
                PromiseState::Pending => {}
            }
        });

        new_promise
    }

    /// Catch errors.
    pub fn catch<F>(&self, f: F) -> Promise<T, E>
    where
        F: Fn(E) -> T + 'static,
        T: 'static,
        E: 'static,
    {
        let new_promise = Promise::new();
        let new_promise_clone = new_promise.clone();

        self.on_change(move |state| {
            match state {
                PromiseState::Resolved(value) => {
                    new_promise_clone.resolve(value.clone());
                }
                PromiseState::Rejected(error) => {
                    new_promise_clone.resolve(f(error.clone()));
                }
                PromiseState::Pending => {}
            }
        });

        new_promise
    }
}

impl<T: Clone, E: Clone> Clone for Promise<T, E> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone, E: Clone> Future for Promise<T, E> {
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.borrow_mut();

        match &inner.state {
            PromiseState::Pending => {
                inner.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            PromiseState::Resolved(value) => Poll::Ready(Ok(value.clone())),
            PromiseState::Rejected(error) => Poll::Ready(Err(error.clone())),
        }
    }
}

/// Deferred promise that can be resolved/rejected externally.
pub struct Deferred<T, E = JsValue> {
    promise: Promise<T, E>,
}

impl<T: Clone, E: Clone> Deferred<T, E> {
    /// Create a new deferred promise.
    pub fn new() -> Self {
        Self {
            promise: Promise::new(),
        }
    }

    /// Get the promise.
    pub fn promise(&self) -> Promise<T, E> {
        self.promise.clone()
    }

    /// Resolve the promise.
    pub fn resolve(&self, value: T) {
        self.promise.resolve(value);
    }

    /// Reject the promise.
    pub fn reject(&self, error: E) {
        self.promise.reject(error);
    }
}

impl<T: Clone, E: Clone> Default for Deferred<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for common Promise operations.
pub mod helpers {
    use super::*;

    /// Create a promise that resolves after a delay (milliseconds).
    #[cfg(target_arch = "wasm32")]
    pub fn delay(ms: i32) -> Promise<(), JsValue> {
        let deferred = Deferred::new();
        let promise = deferred.promise();

        let callback = Closure::once(move || {
            deferred.resolve(());
        });

        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                ms,
            )
            .unwrap();

        callback.forget();
        promise
    }

    /// Wrap a Future in a Promise.
    pub fn from_future<F, T, E>(future: F) -> Promise<T, E>
    where
        F: Future<Output = Result<T, E>> + 'static,
        T: Clone + 'static,
        E: Clone + 'static,
    {
        let promise = Promise::new();
        let promise_clone = promise.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match future.await {
                Ok(value) => promise_clone.resolve(value),
                Err(error) => promise_clone.reject(error),
            }
        });

        promise
    }

    /// Combine multiple promises (all must resolve).
    pub fn all<T, E>(promises: Vec<Promise<T, E>>) -> Promise<Vec<T>, E>
    where
        T: Clone + 'static,
        E: Clone + 'static,
    {
        let result_promise = Promise::new();
        let result_promise_clone = result_promise.clone();
        let count = promises.len();

        if count == 0 {
            result_promise.resolve(vec![]);
            return result_promise;
        }

        let results = Rc::new(RefCell::new(Vec::with_capacity(count)));
        let completed = Rc::new(RefCell::new(0));

        for (i, promise) in promises.into_iter().enumerate() {
            let results_clone = Rc::clone(&results);
            let completed_clone = Rc::clone(&completed);
            let result_promise_clone2 = result_promise_clone.clone();

            promise.on_change(move |state| {
                match state {
                    PromiseState::Resolved(value) => {
                        results_clone.borrow_mut().push((i, value.clone()));
                        *completed_clone.borrow_mut() += 1;

                        if *completed_clone.borrow() == count {
                            let mut sorted_results = results_clone.borrow().clone();
                            sorted_results.sort_by_key(|(index, _)| *index);
                            let values: Vec<T> = sorted_results.into_iter().map(|(_, v)| v).collect();
                            result_promise_clone2.resolve(values);
                        }
                    }
                    PromiseState::Rejected(error) => {
                        result_promise_clone2.reject(error.clone());
                    }
                    _ => {}
                }
            });
        }

        result_promise
    }

    /// Race multiple promises (first to settle wins).
    pub fn race<T, E>(promises: Vec<Promise<T, E>>) -> Promise<T, E>
    where
        T: Clone + 'static,
        E: Clone + 'static,
    {
        let result_promise = Promise::new();
        let settled = Rc::new(RefCell::new(false));

        for promise in promises {
            let result_promise_clone = result_promise.clone();
            let settled_clone = Rc::clone(&settled);

            promise.on_change(move |state| {
                if !*settled_clone.borrow() {
                    *settled_clone.borrow_mut() = true;
                    match state {
                        PromiseState::Resolved(value) => {
                            result_promise_clone.resolve(value.clone());
                        }
                        PromiseState::Rejected(error) => {
                            result_promise_clone.reject(error.clone());
                        }
                        _ => {}
                    }
                }
            });
        }

        result_promise
    }
}

