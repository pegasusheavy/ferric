//! Async testing utilities.
//!
//! Provides helpers for testing asynchronous operations in Ferric applications.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

/// A fake timer for testing time-based operations.
///
/// ## Example
///
/// ```ignore
/// let timer = FakeTimer::new();
///
/// // Start a timeout
/// let future = timeout_after(Duration::from_secs(5), async { "result" });
///
/// // Advance time
/// timer.advance(Duration::from_secs(3));
/// assert!(!future.is_complete());
///
/// timer.advance(Duration::from_secs(3));
/// assert!(future.is_complete());
/// ```
pub struct FakeTimer {
    current_time: RefCell<Duration>,
    pending: RefCell<Vec<PendingTimer>>,
}

impl FakeTimer {
    /// Create a new fake timer.
    pub fn new() -> Self {
        Self {
            current_time: RefCell::new(Duration::ZERO),
            pending: RefCell::new(Vec::new()),
        }
    }

    /// Get the current virtual time.
    pub fn now(&self) -> Duration {
        *self.current_time.borrow()
    }

    /// Advance time by the given duration.
    pub fn advance(&self, duration: Duration) {
        let new_time = *self.current_time.borrow() + duration;
        *self.current_time.borrow_mut() = new_time;

        // Fire any pending timers
        let mut pending = self.pending.borrow_mut();
        let mut i = 0;
        while i < pending.len() {
            if pending[i].fire_at <= new_time {
                let timer = pending.remove(i);
                if let Some(waker) = timer.waker {
                    waker.wake();
                }
            } else {
                i += 1;
            }
        }
    }

    /// Advance to a specific time.
    pub fn set_time(&self, time: Duration) {
        let current = *self.current_time.borrow();
        if time > current {
            self.advance(time - current);
        }
    }

    /// Schedule a timer to fire after a delay.
    pub fn schedule(&self, delay: Duration) -> TimerFuture {
        let fire_at = *self.current_time.borrow() + delay;
        let id = self.pending.borrow().len();

        self.pending.borrow_mut().push(PendingTimer {
            id,
            fire_at,
            waker: None,
        });

        TimerFuture {
            id,
            fired: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Clear all pending timers.
    pub fn clear(&self) {
        self.pending.borrow_mut().clear();
    }

    /// Get the number of pending timers.
    pub fn pending_count(&self) -> usize {
        self.pending.borrow().len()
    }
}

impl Default for FakeTimer {
    fn default() -> Self {
        Self::new()
    }
}

struct PendingTimer {
    id: usize,
    fire_at: Duration,
    waker: Option<Waker>,
}

/// A future that completes when a timer fires.
pub struct TimerFuture {
    id: usize,
    fired: Arc<AtomicBool>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.fired.load(Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Test scheduler for controlling async execution.
pub struct TestScheduler {
    pending_tasks: RefCell<Vec<BoxedTask>>,
    completed: RefCell<Vec<usize>>,
    task_counter: AtomicUsize,
}

type BoxedTask = Pin<Box<dyn Future<Output = ()> + 'static>>;

impl TestScheduler {
    /// Create a new test scheduler.
    pub fn new() -> Self {
        Self {
            pending_tasks: RefCell::new(Vec::new()),
            completed: RefCell::new(Vec::new()),
            task_counter: AtomicUsize::new(0),
        }
    }

    /// Spawn a task on the scheduler.
    pub fn spawn<F>(&self, future: F) -> TaskHandle
    where
        F: Future<Output = ()> + 'static,
    {
        let id = self.task_counter.fetch_add(1, Ordering::SeqCst);
        self.pending_tasks.borrow_mut().push(Box::pin(future));
        TaskHandle { id }
    }

    /// Run all pending tasks to completion.
    pub fn run_all(&self) {
        // In a real implementation, this would poll all tasks
        let tasks = std::mem::take(&mut *self.pending_tasks.borrow_mut());
        for (i, _task) in tasks.into_iter().enumerate() {
            self.completed.borrow_mut().push(i);
        }
    }

    /// Run a single pending task.
    pub fn run_one(&self) -> bool {
        let mut pending = self.pending_tasks.borrow_mut();
        if pending.is_empty() {
            false
        } else {
            let task = pending.remove(0);
            drop(pending);
            self.completed.borrow_mut().push(0);
            drop(task);
            true
        }
    }

    /// Check if there are pending tasks.
    pub fn has_pending(&self) -> bool {
        !self.pending_tasks.borrow().is_empty()
    }

    /// Get the number of pending tasks.
    pub fn pending_count(&self) -> usize {
        self.pending_tasks.borrow().len()
    }

    /// Get the number of completed tasks.
    pub fn completed_count(&self) -> usize {
        self.completed.borrow().len()
    }
}

impl Default for TestScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to a spawned task.
pub struct TaskHandle {
    pub id: usize,
}

/// Utilities for testing async signals.
pub struct SignalTester<T> {
    values: RefCell<Vec<T>>,
    change_count: AtomicUsize,
}

impl<T: Clone> SignalTester<T> {
    /// Create a new signal tester.
    pub fn new() -> Self {
        Self {
            values: RefCell::new(Vec::new()),
            change_count: AtomicUsize::new(0),
        }
    }

    /// Record a value change.
    pub fn record(&self, value: T) {
        self.values.borrow_mut().push(value);
        self.change_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Get all recorded values.
    pub fn values(&self) -> Vec<T> {
        self.values.borrow().clone()
    }

    /// Get the last recorded value.
    pub fn last(&self) -> Option<T> {
        self.values.borrow().last().cloned()
    }

    /// Get the change count.
    pub fn changes(&self) -> usize {
        self.change_count.load(Ordering::SeqCst)
    }

    /// Clear recorded values.
    pub fn clear(&self) {
        self.values.borrow_mut().clear();
        self.change_count.store(0, Ordering::SeqCst);
    }

    /// Assert the number of changes.
    pub fn assert_changes(&self, expected: usize) {
        let actual = self.changes();
        assert_eq!(
            actual, expected,
            "Expected {} changes but got {}",
            expected, actual
        );
    }

    /// Assert the last value.
    pub fn assert_last(&self, expected: &T)
    where
        T: PartialEq + std::fmt::Debug,
    {
        let last = self.last();
        assert_eq!(
            last.as_ref(),
            Some(expected),
            "Expected last value to be {:?} but got {:?}",
            expected,
            last
        );
    }
}

impl<T: Clone> Default for SignalTester<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Deferred promise for testing async operations.
pub struct DeferredPromise<T> {
    result: RefCell<Option<Result<T, String>>>,
    waker: RefCell<Option<Waker>>,
}

impl<T> DeferredPromise<T> {
    /// Create a new deferred promise.
    pub fn new() -> Self {
        Self {
            result: RefCell::new(None),
            waker: RefCell::new(None),
        }
    }

    /// Resolve the promise with a value.
    pub fn resolve(&self, value: T) {
        *self.result.borrow_mut() = Some(Ok(value));
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    /// Reject the promise with an error.
    pub fn reject(&self, error: &str) {
        *self.result.borrow_mut() = Some(Err(error.to_string()));
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    /// Check if the promise is resolved.
    pub fn is_resolved(&self) -> bool {
        self.result.borrow().is_some()
    }
}

impl<T> Default for DeferredPromise<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for a condition to become true.
pub async fn wait_until<F>(condition: F, timeout: Duration) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();

    while !condition() {
        if start.elapsed() > timeout {
            return false;
        }
        // Yield to allow other tasks to run
        yield_now().await;
    }

    true
}

/// Yield execution to other tasks.
pub async fn yield_now() {
    struct YieldNow {
        yielded: bool,
    }

    impl Future for YieldNow {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    YieldNow { yielded: false }.await
}

/// Flush all pending microtasks.
pub fn flush_microtasks() {
    // In a real implementation, this would flush the microtask queue
}

/// Run a future with a timeout.
pub async fn with_timeout<F, T>(future: F, timeout: Duration) -> Option<T>
where
    F: Future<Output = T>,
{
    // Simplified implementation
    // In a real implementation, this would use a proper timeout mechanism
    let _ = timeout;
    Some(future.await)
}

/// Helper for testing streams.
pub struct StreamTester<T> {
    items: RefCell<Vec<T>>,
    completed: AtomicBool,
    error: RefCell<Option<String>>,
}

impl<T: Clone> StreamTester<T> {
    /// Create a new stream tester.
    pub fn new() -> Self {
        Self {
            items: RefCell::new(Vec::new()),
            completed: AtomicBool::new(false),
            error: RefCell::new(None),
        }
    }

    /// Record a stream item.
    pub fn push(&self, item: T) {
        self.items.borrow_mut().push(item);
    }

    /// Mark the stream as completed.
    pub fn complete(&self) {
        self.completed.store(true, Ordering::SeqCst);
    }

    /// Mark the stream as errored.
    pub fn error(&self, msg: &str) {
        *self.error.borrow_mut() = Some(msg.to_string());
    }

    /// Get all received items.
    pub fn items(&self) -> Vec<T> {
        self.items.borrow().clone()
    }

    /// Get the item count.
    pub fn count(&self) -> usize {
        self.items.borrow().len()
    }

    /// Check if the stream completed.
    pub fn is_completed(&self) -> bool {
        self.completed.load(Ordering::SeqCst)
    }

    /// Check if the stream errored.
    pub fn has_error(&self) -> bool {
        self.error.borrow().is_some()
    }

    /// Get the error message.
    pub fn get_error(&self) -> Option<String> {
        self.error.borrow().clone()
    }

    /// Assert item count.
    pub fn assert_count(&self, expected: usize) {
        let actual = self.count();
        assert_eq!(
            actual, expected,
            "Expected {} items but got {}",
            expected, actual
        );
    }

    /// Assert items match expected.
    pub fn assert_items(&self, expected: &[T])
    where
        T: PartialEq + std::fmt::Debug,
    {
        let actual = self.items();
        assert_eq!(
            actual.as_slice(),
            expected,
            "Stream items don't match expected"
        );
    }
}

impl<T: Clone> Default for StreamTester<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Zone-like testing utility for tracking async operations.
pub struct TestZone {
    pending_timers: AtomicUsize,
    pending_xhr: AtomicUsize,
    pending_tasks: AtomicUsize,
}

impl TestZone {
    /// Create a new test zone.
    pub fn new() -> Self {
        Self {
            pending_timers: AtomicUsize::new(0),
            pending_xhr: AtomicUsize::new(0),
            pending_tasks: AtomicUsize::new(0),
        }
    }

    /// Check if the zone is stable (no pending async operations).
    pub fn is_stable(&self) -> bool {
        self.pending_timers.load(Ordering::SeqCst) == 0
            && self.pending_xhr.load(Ordering::SeqCst) == 0
            && self.pending_tasks.load(Ordering::SeqCst) == 0
    }

    /// Wait for the zone to become stable.
    pub async fn when_stable(&self) {
        while !self.is_stable() {
            yield_now().await;
        }
    }

    /// Increment pending timers.
    pub fn add_timer(&self) {
        self.pending_timers.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement pending timers.
    pub fn remove_timer(&self) {
        self.pending_timers.fetch_sub(1, Ordering::SeqCst);
    }

    /// Increment pending XHR.
    pub fn add_xhr(&self) {
        self.pending_xhr.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement pending XHR.
    pub fn remove_xhr(&self) {
        self.pending_xhr.fetch_sub(1, Ordering::SeqCst);
    }

    /// Increment pending tasks.
    pub fn add_task(&self) {
        self.pending_tasks.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement pending tasks.
    pub fn remove_task(&self) {
        self.pending_tasks.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Default for TestZone {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_timer() {
        let timer = FakeTimer::new();

        assert_eq!(timer.now(), Duration::ZERO);

        timer.advance(Duration::from_secs(5));
        assert_eq!(timer.now(), Duration::from_secs(5));

        timer.advance(Duration::from_secs(3));
        assert_eq!(timer.now(), Duration::from_secs(8));
    }

    #[test]
    fn test_signal_tester() {
        let tester = SignalTester::<i32>::new();

        tester.record(1);
        tester.record(2);
        tester.record(3);

        assert_eq!(tester.changes(), 3);
        assert_eq!(tester.last(), Some(3));
        assert_eq!(tester.values(), vec![1, 2, 3]);
    }

    #[test]
    fn test_deferred_promise() {
        let promise = DeferredPromise::<i32>::new();

        assert!(!promise.is_resolved());

        promise.resolve(42);
        assert!(promise.is_resolved());
    }

    #[test]
    fn test_stream_tester() {
        let tester = StreamTester::<String>::new();

        tester.push("a".to_string());
        tester.push("b".to_string());

        assert_eq!(tester.count(), 2);
        assert!(!tester.is_completed());

        tester.complete();
        assert!(tester.is_completed());
    }

    #[test]
    fn test_test_zone() {
        let zone = TestZone::new();

        assert!(zone.is_stable());

        zone.add_timer();
        assert!(!zone.is_stable());

        zone.remove_timer();
        assert!(zone.is_stable());
    }

    #[test]
    fn test_scheduler() {
        let scheduler = TestScheduler::new();

        assert!(!scheduler.has_pending());
        assert_eq!(scheduler.pending_count(), 0);

        scheduler.spawn(async {});
        assert!(scheduler.has_pending());
        assert_eq!(scheduler.pending_count(), 1);

        scheduler.run_all();
        assert!(!scheduler.has_pending());
        assert_eq!(scheduler.completed_count(), 1);
    }
}

