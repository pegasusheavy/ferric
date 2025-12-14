//! Timeout utilities for async operations.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use wasm_bindgen::prelude::*;

/// Future that resolves after a delay.
pub struct Timeout {
    ms: i32,
    timeout_id: Option<i32>,
    resolved: bool,
}

impl Timeout {
    /// Create a new timeout.
    pub fn new(duration: Duration) -> Self {
        Self {
            ms: duration.as_millis() as i32,
            timeout_id: None,
            resolved: false,
        }
    }
}

impl Future for Timeout {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.resolved {
            return Poll::Ready(());
        }

        if self.timeout_id.is_none() {
            // Set up timeout
            let waker = cx.waker().clone();
            let resolved_ptr = &mut self.resolved as *mut bool;

            let callback = Closure::once(move || {
                unsafe {
                    *resolved_ptr = true;
                }
                waker.wake();
            });

            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        callback.as_ref().unchecked_ref(),
                        self.ms,
                    ) {
                        self.timeout_id = Some(id);
                    }
                }
            }

            callback.forget();
        }

        Poll::Pending
    }
}

/// Sleep for the specified duration.
pub fn sleep(duration: Duration) -> Timeout {
    Timeout::new(duration)
}

/// Add a timeout to a future.
pub async fn timeout<F, T>(_duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    // Simplified implementation
    // A full implementation would race the future against a timeout
    Ok(future.await)
}

/// Error returned when a timeout occurs.
#[derive(Debug, Clone)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation timed out")
    }
}

impl std::error::Error for TimeoutError {}

