//! Task spawning utilities for async operations.

use std::future::Future;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen_futures::spawn_local;

/// Handle to a spawned task that can be cancelled.
pub struct TaskHandle {
    cancelled: Rc<RefCell<bool>>,
}

impl TaskHandle {
    /// Cancel the task.
    pub fn cancel(&self) {
        *self.cancelled.borrow_mut() = true;
    }

    /// Check if the task is cancelled.
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow()
    }
}

/// Spawn a future on the local executor.
pub fn spawn_local_task<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(future);
}

/// Spawn a future and get a handle to cancel it.
pub fn spawn_local_with_handle<F>(future: F) -> TaskHandle
where
    F: Future<Output = ()> + 'static,
{
    let cancelled = Rc::new(RefCell::new(false));
    let cancelled_clone = Rc::clone(&cancelled);

    spawn_local(async move {
        if !*cancelled_clone.borrow() {
            future.await;
        }
    });

    TaskHandle { cancelled }
}

/// Spawn a future that returns a result.
pub fn spawn_local_result<F, T>(future: F, callback: impl FnOnce(T) + 'static)
where
    F: Future<Output = T> + 'static,
    T: 'static,
{
    spawn_local(async move {
        let result = future.await;
        callback(result);
    });
}

/// Execute a future and get the result via a callback.
pub fn execute<F, T>(future: F) -> impl Future<Output = T>
where
    F: Future<Output = T>,
{
    future
}

