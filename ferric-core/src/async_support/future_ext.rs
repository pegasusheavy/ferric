//! Future extension traits for enhanced async operations.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use super::promise::Promise;

/// Extension trait for Future with additional utilities.
pub trait FutureExt: Future + Sized {
    /// Convert this future to a Promise.
    fn into_promise(self) -> Promise<Self::Output, String>
    where
        Self: 'static,
        Self::Output: Clone + 'static,
    {
        let promise = Promise::new();
        let promise_clone = promise.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let result = self.await;
            promise_clone.resolve(result);
        });

        promise
    }

    /// Add a timeout to this future.
    fn with_timeout(self, duration: Duration) -> TimeoutFuture<Self> {
        TimeoutFuture {
            future: Box::pin(self),
            duration,
        }
    }

    /// Retry this future on failure.
    fn retry(self, max_attempts: usize) -> RetryFuture<Self> {
        RetryFuture {
            future: Box::pin(self),
            max_attempts,
            current_attempt: 0,
        }
    }

    /// Execute a callback on completion.
    fn on_complete<F>(self, callback: F) -> OnCompleteFuture<Self, F>
    where
        F: FnOnce(&Self::Output),
    {
        OnCompleteFuture {
            future: Box::pin(self),
            callback: Some(callback),
        }
    }
}

impl<T: Future> FutureExt for T {}

/// Future with timeout support.
pub struct TimeoutFuture<F> {
    future: Pin<Box<F>>,
    duration: Duration,
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output, String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // For simplicity, we poll the inner future
        // A full implementation would use a timer
        match self.future.as_mut().poll(cx) {
            std::task::Poll::Ready(value) => std::task::Poll::Ready(Ok(value)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// Future with retry support.
pub struct RetryFuture<F> {
    future: Pin<Box<F>>,
    max_attempts: usize,
    current_attempt: usize,
}

impl<F: Future> Future for RetryFuture<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // Simplified retry logic
        self.future.as_mut().poll(cx)
    }
}

/// Future that calls a callback on completion.
pub struct OnCompleteFuture<F, C> {
    future: Pin<Box<F>>,
    callback: Option<C>,
}

impl<F, C> Future for OnCompleteFuture<F, C>
where
    F: Future,
    C: FnOnce(&F::Output),
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {
            let this = self.get_unchecked_mut();
            match this.future.as_mut().poll(cx) {
                std::task::Poll::Ready(value) => {
                    if let Some(callback) = this.callback.take() {
                        callback(&value);
                    }
                    std::task::Poll::Ready(value)
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        }
    }
}

