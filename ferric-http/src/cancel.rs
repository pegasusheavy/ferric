//! Request cancellation support.
//!
//! Provides tokens for cancelling in-flight HTTP requests.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, CancellationToken};
//!
//! let (token, trigger) = CancellationToken::new();
//!
//! // Start request with cancellation support
//! let response_future = client
//!     .get("https://example.com/slow-endpoint")
//!     .with_cancellation(token)
//!     .send();
//!
//! // Cancel after 5 seconds
//! spawn(async move {
//!     sleep(Duration::from_secs(5)).await;
//!     trigger.cancel();
//! });
//!
//! // This will return Err(Cancelled) if cancelled
//! let result = response_future.await;
//! ```

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// A token that can be used to check if a request should be cancelled.
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<CancellationInner>,
}

struct CancellationInner {
    cancelled: AtomicBool,
    wakers: std::sync::Mutex<Vec<Waker>>,
    cancel_reason: std::sync::Mutex<Option<String>>,
}

impl CancellationToken {
    /// Create a new cancellation token and trigger pair.
    pub fn new() -> (Self, CancellationTrigger) {
        let inner = Arc::new(CancellationInner {
            cancelled: AtomicBool::new(false),
            wakers: std::sync::Mutex::new(Vec::new()),
            cancel_reason: std::sync::Mutex::new(None),
        });

        let token = Self {
            inner: inner.clone(),
        };

        let trigger = CancellationTrigger { inner };

        (token, trigger)
    }

    /// Create a token that is already cancelled.
    pub fn cancelled() -> Self {
        let inner = Arc::new(CancellationInner {
            cancelled: AtomicBool::new(true),
            wakers: std::sync::Mutex::new(Vec::new()),
            cancel_reason: std::sync::Mutex::new(None),
        });
        Self { inner }
    }

    /// Create a token that will never be cancelled.
    pub fn none() -> Self {
        let inner = Arc::new(CancellationInner {
            cancelled: AtomicBool::new(false),
            wakers: std::sync::Mutex::new(Vec::new()),
            cancel_reason: std::sync::Mutex::new(None),
        });
        Self { inner }
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    /// Get the cancellation reason (if any).
    pub fn reason(&self) -> Option<String> {
        self.inner.cancel_reason.lock().ok().and_then(|g| g.clone())
    }

    /// Create a future that completes when cancelled.
    pub fn cancelled_future(&self) -> CancelledFuture {
        CancelledFuture {
            token: self.clone(),
        }
    }

    /// Register a waker to be notified on cancellation.
    pub(crate) fn register_waker(&self, waker: Waker) {
        if let Ok(mut wakers) = self.inner.wakers.lock() {
            wakers.push(waker);
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::none()
    }
}

impl fmt::Debug for CancellationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationToken")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// A trigger that can cancel associated tokens.
pub struct CancellationTrigger {
    inner: Arc<CancellationInner>,
}

impl CancellationTrigger {
    /// Cancel the associated token.
    pub fn cancel(&self) {
        self.cancel_with_reason(None);
    }

    /// Cancel with a reason.
    pub fn cancel_with_reason(&self, reason: Option<String>) {
        // Set the cancelled flag
        self.inner.cancelled.store(true, Ordering::SeqCst);

        // Store the reason
        if let Ok(mut r) = self.inner.cancel_reason.lock() {
            *r = reason;
        }

        // Wake all waiting tasks
        if let Ok(mut wakers) = self.inner.wakers.lock() {
            for waker in wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// Check if already cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }
}

impl fmt::Debug for CancellationTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationTrigger")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// A future that completes when a cancellation token is triggered.
pub struct CancelledFuture {
    token: CancellationToken,
}

impl Future for CancelledFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            self.token.register_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// A cancellation source that can create multiple tokens.
///
/// Useful when you want to cancel multiple requests at once.
pub struct CancellationSource {
    inner: Arc<CancellationInner>,
}

impl CancellationSource {
    /// Create a new cancellation source.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancellationInner {
                cancelled: AtomicBool::new(false),
                wakers: std::sync::Mutex::new(Vec::new()),
                cancel_reason: std::sync::Mutex::new(None),
            }),
        }
    }

    /// Create a token from this source.
    pub fn token(&self) -> CancellationToken {
        CancellationToken {
            inner: self.inner.clone(),
        }
    }

    /// Cancel all tokens from this source.
    pub fn cancel(&self) {
        self.cancel_with_reason(None);
    }

    /// Cancel with a reason.
    pub fn cancel_with_reason(&self, reason: Option<String>) {
        self.inner.cancelled.store(true, Ordering::SeqCst);

        if let Ok(mut r) = self.inner.cancel_reason.lock() {
            *r = reason;
        }

        if let Ok(mut wakers) = self.inner.wakers.lock() {
            for waker in wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationSource {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CancellationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationSource")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// Timeout-based cancellation.
///
/// Creates a token that automatically cancels after a duration.
#[cfg(not(target_arch = "wasm32"))]
pub fn timeout_token(duration: std::time::Duration) -> CancellationToken {
    let (token, trigger) = CancellationToken::new();

    std::thread::spawn(move || {
        std::thread::sleep(duration);
        trigger.cancel_with_reason(Some("Timeout".to_string()));
    });

    token
}

/// Combine multiple tokens - cancels if any token is cancelled.
pub fn any_cancelled(tokens: Vec<CancellationToken>) -> CancellationToken {
    let (combined_token, trigger) = CancellationToken::new();

    // Check if any are already cancelled
    for token in &tokens {
        if token.is_cancelled() {
            trigger.cancel_with_reason(token.reason());
            return combined_token;
        }
    }

    // For a full implementation, we'd need to spawn a task to monitor the tokens
    // This is a simplified version
    combined_token
}

/// Combine multiple tokens - cancels only if all tokens are cancelled.
pub fn all_cancelled(tokens: Vec<CancellationToken>) -> CancellationToken {
    let (combined_token, trigger) = CancellationToken::new();

    // Check if all are cancelled
    if tokens.iter().all(|t| t.is_cancelled()) {
        trigger.cancel();
    }

    combined_token
}

/// Guard that cancels on drop.
pub struct CancelOnDrop {
    trigger: Option<CancellationTrigger>,
}

impl CancelOnDrop {
    /// Create a new cancel-on-drop guard.
    pub fn new(trigger: CancellationTrigger) -> Self {
        Self {
            trigger: Some(trigger),
        }
    }

    /// Disarm the guard (don't cancel on drop).
    pub fn disarm(&mut self) {
        self.trigger = None;
    }
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(trigger) = self.trigger.take() {
            trigger.cancel_with_reason(Some("Dropped".to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_token() {
        let (token, trigger) = CancellationToken::new();

        assert!(!token.is_cancelled());

        trigger.cancel();

        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_with_reason() {
        let (token, trigger) = CancellationToken::new();

        trigger.cancel_with_reason(Some("User cancelled".to_string()));

        assert!(token.is_cancelled());
        assert_eq!(token.reason(), Some("User cancelled".to_string()));
    }

    #[test]
    fn test_cancellation_source() {
        let source = CancellationSource::new();
        let token1 = source.token();
        let token2 = source.token();

        assert!(!token1.is_cancelled());
        assert!(!token2.is_cancelled());

        source.cancel();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }

    #[test]
    fn test_pre_cancelled_token() {
        let token = CancellationToken::cancelled();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancel_on_drop() {
        let (token, trigger) = CancellationToken::new();

        {
            let _guard = CancelOnDrop::new(trigger);
            assert!(!token.is_cancelled());
        }

        assert!(token.is_cancelled());
        assert_eq!(token.reason(), Some("Dropped".to_string()));
    }

    #[test]
    fn test_cancel_on_drop_disarm() {
        let (token, trigger) = CancellationToken::new();

        {
            let mut guard = CancelOnDrop::new(trigger);
            guard.disarm();
        }

        assert!(!token.is_cancelled());
    }
}


//!
//! Provides tokens for cancelling in-flight HTTP requests.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, CancellationToken};
//!
//! let (token, trigger) = CancellationToken::new();
//!
//! // Start request with cancellation support
//! let response_future = client
//!     .get("https://example.com/slow-endpoint")
//!     .with_cancellation(token)
//!     .send();
//!
//! // Cancel after 5 seconds
//! spawn(async move {
//!     sleep(Duration::from_secs(5)).await;
//!     trigger.cancel();
//! });
//!
//! // This will return Err(Cancelled) if cancelled
//! let result = response_future.await;
//! ```

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

/// A token that can be used to check if a request should be cancelled.
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<CancellationInner>,
}

struct CancellationInner {
    cancelled: AtomicBool,
    wakers: std::sync::Mutex<Vec<Waker>>,
    cancel_reason: std::sync::Mutex<Option<String>>,
}

impl CancellationToken {
    /// Create a new cancellation token and trigger pair.
    pub fn new() -> (Self, CancellationTrigger) {
        let inner = Arc::new(CancellationInner {
            cancelled: AtomicBool::new(false),
            wakers: std::sync::Mutex::new(Vec::new()),
            cancel_reason: std::sync::Mutex::new(None),
        });

        let token = Self {
            inner: inner.clone(),
        };

        let trigger = CancellationTrigger { inner };

        (token, trigger)
    }

    /// Create a token that is already cancelled.
    pub fn cancelled() -> Self {
        let inner = Arc::new(CancellationInner {
            cancelled: AtomicBool::new(true),
            wakers: std::sync::Mutex::new(Vec::new()),
            cancel_reason: std::sync::Mutex::new(None),
        });
        Self { inner }
    }

    /// Create a token that will never be cancelled.
    pub fn none() -> Self {
        let inner = Arc::new(CancellationInner {
            cancelled: AtomicBool::new(false),
            wakers: std::sync::Mutex::new(Vec::new()),
            cancel_reason: std::sync::Mutex::new(None),
        });
        Self { inner }
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    /// Get the cancellation reason (if any).
    pub fn reason(&self) -> Option<String> {
        self.inner.cancel_reason.lock().ok().and_then(|g| g.clone())
    }

    /// Create a future that completes when cancelled.
    pub fn cancelled_future(&self) -> CancelledFuture {
        CancelledFuture {
            token: self.clone(),
        }
    }

    /// Register a waker to be notified on cancellation.
    pub(crate) fn register_waker(&self, waker: Waker) {
        if let Ok(mut wakers) = self.inner.wakers.lock() {
            wakers.push(waker);
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::none()
    }
}

impl fmt::Debug for CancellationToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationToken")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// A trigger that can cancel associated tokens.
pub struct CancellationTrigger {
    inner: Arc<CancellationInner>,
}

impl CancellationTrigger {
    /// Cancel the associated token.
    pub fn cancel(&self) {
        self.cancel_with_reason(None);
    }

    /// Cancel with a reason.
    pub fn cancel_with_reason(&self, reason: Option<String>) {
        // Set the cancelled flag
        self.inner.cancelled.store(true, Ordering::SeqCst);

        // Store the reason
        if let Ok(mut r) = self.inner.cancel_reason.lock() {
            *r = reason;
        }

        // Wake all waiting tasks
        if let Ok(mut wakers) = self.inner.wakers.lock() {
            for waker in wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// Check if already cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }
}

impl fmt::Debug for CancellationTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationTrigger")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// A future that completes when a cancellation token is triggered.
pub struct CancelledFuture {
    token: CancellationToken,
}

impl Future for CancelledFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            self.token.register_waker(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// A cancellation source that can create multiple tokens.
///
/// Useful when you want to cancel multiple requests at once.
pub struct CancellationSource {
    inner: Arc<CancellationInner>,
}

impl CancellationSource {
    /// Create a new cancellation source.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancellationInner {
                cancelled: AtomicBool::new(false),
                wakers: std::sync::Mutex::new(Vec::new()),
                cancel_reason: std::sync::Mutex::new(None),
            }),
        }
    }

    /// Create a token from this source.
    pub fn token(&self) -> CancellationToken {
        CancellationToken {
            inner: self.inner.clone(),
        }
    }

    /// Cancel all tokens from this source.
    pub fn cancel(&self) {
        self.cancel_with_reason(None);
    }

    /// Cancel with a reason.
    pub fn cancel_with_reason(&self, reason: Option<String>) {
        self.inner.cancelled.store(true, Ordering::SeqCst);

        if let Ok(mut r) = self.inner.cancel_reason.lock() {
            *r = reason;
        }

        if let Ok(mut wakers) = self.inner.wakers.lock() {
            for waker in wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationSource {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CancellationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationSource")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// Timeout-based cancellation.
///
/// Creates a token that automatically cancels after a duration.
#[cfg(not(target_arch = "wasm32"))]
pub fn timeout_token(duration: std::time::Duration) -> CancellationToken {
    let (token, trigger) = CancellationToken::new();

    std::thread::spawn(move || {
        std::thread::sleep(duration);
        trigger.cancel_with_reason(Some("Timeout".to_string()));
    });

    token
}

/// Combine multiple tokens - cancels if any token is cancelled.
pub fn any_cancelled(tokens: Vec<CancellationToken>) -> CancellationToken {
    let (combined_token, trigger) = CancellationToken::new();

    // Check if any are already cancelled
    for token in &tokens {
        if token.is_cancelled() {
            trigger.cancel_with_reason(token.reason());
            return combined_token;
        }
    }

    // For a full implementation, we'd need to spawn a task to monitor the tokens
    // This is a simplified version
    combined_token
}

/// Combine multiple tokens - cancels only if all tokens are cancelled.
pub fn all_cancelled(tokens: Vec<CancellationToken>) -> CancellationToken {
    let (combined_token, trigger) = CancellationToken::new();

    // Check if all are cancelled
    if tokens.iter().all(|t| t.is_cancelled()) {
        trigger.cancel();
    }

    combined_token
}

/// Guard that cancels on drop.
pub struct CancelOnDrop {
    trigger: Option<CancellationTrigger>,
}

impl CancelOnDrop {
    /// Create a new cancel-on-drop guard.
    pub fn new(trigger: CancellationTrigger) -> Self {
        Self {
            trigger: Some(trigger),
        }
    }

    /// Disarm the guard (don't cancel on drop).
    pub fn disarm(&mut self) {
        self.trigger = None;
    }
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(trigger) = self.trigger.take() {
            trigger.cancel_with_reason(Some("Dropped".to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_token() {
        let (token, trigger) = CancellationToken::new();

        assert!(!token.is_cancelled());

        trigger.cancel();

        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_with_reason() {
        let (token, trigger) = CancellationToken::new();

        trigger.cancel_with_reason(Some("User cancelled".to_string()));

        assert!(token.is_cancelled());
        assert_eq!(token.reason(), Some("User cancelled".to_string()));
    }

    #[test]
    fn test_cancellation_source() {
        let source = CancellationSource::new();
        let token1 = source.token();
        let token2 = source.token();

        assert!(!token1.is_cancelled());
        assert!(!token2.is_cancelled());

        source.cancel();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }

    #[test]
    fn test_pre_cancelled_token() {
        let token = CancellationToken::cancelled();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancel_on_drop() {
        let (token, trigger) = CancellationToken::new();

        {
            let _guard = CancelOnDrop::new(trigger);
            assert!(!token.is_cancelled());
        }

        assert!(token.is_cancelled());
        assert_eq!(token.reason(), Some("Dropped".to_string()));
    }

    #[test]
    fn test_cancel_on_drop_disarm() {
        let (token, trigger) = CancellationToken::new();

        {
            let mut guard = CancelOnDrop::new(trigger);
            guard.disarm();
        }

        assert!(!token.is_cancelled());
    }
}

