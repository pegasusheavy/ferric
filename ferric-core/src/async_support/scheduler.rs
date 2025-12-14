//! Async task scheduler.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::cell::RefCell;

type BoxFuture = Pin<Box<dyn Future<Output = ()>>>;

/// Priority levels for scheduled tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

struct ScheduledTask {
    future: BoxFuture,
    priority: Priority,
}

/// Async task scheduler with priority support.
pub struct AsyncScheduler {
    tasks: Rc<RefCell<VecDeque<ScheduledTask>>>,
    running: Rc<RefCell<bool>>,
}

impl AsyncScheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        Self {
            tasks: Rc::new(RefCell::new(VecDeque::new())),
            running: Rc::new(RefCell::new(false)),
        }
    }

    /// Schedule a task with default priority.
    pub fn schedule<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.schedule_with_priority(future, Priority::Normal);
    }

    /// Schedule a task with specific priority.
    pub fn schedule_with_priority<F>(&self, future: F, priority: Priority)
    where
        F: Future<Output = ()> + 'static,
    {
        let task = ScheduledTask {
            future: Box::pin(future),
            priority,
        };

        let mut tasks = self.tasks.borrow_mut();

        // Insert based on priority (higher priority first)
        let insert_pos = tasks
            .iter()
            .position(|t| t.priority < priority)
            .unwrap_or(tasks.len());

        tasks.insert(insert_pos, task);
    }

    /// Run all scheduled tasks.
    pub async fn run(&self) {
        if *self.running.borrow() {
            return;
        }

        *self.running.borrow_mut() = true;

        while let Some(task) = self.tasks.borrow_mut().pop_front() {
            task.future.await;
        }

        *self.running.borrow_mut() = false;
    }

    /// Get the number of pending tasks.
    pub fn pending_count(&self) -> usize {
        self.tasks.borrow().len()
    }

    /// Clear all pending tasks.
    pub fn clear(&self) {
        self.tasks.borrow_mut().clear();
    }

    /// Check if the scheduler is currently running.
    pub fn is_running(&self) -> bool {
        *self.running.borrow()
    }
}

impl Default for AsyncScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AsyncScheduler {
    fn clone(&self) -> Self {
        Self {
            tasks: Rc::clone(&self.tasks),
            running: Rc::clone(&self.running),
        }
    }
}

/// Global scheduler instance (single-threaded for WASM).
thread_local! {
    static GLOBAL_SCHEDULER: RefCell<Option<AsyncScheduler>> = const { RefCell::new(None) };
}

/// Get the global scheduler.
pub fn global_scheduler() -> AsyncScheduler {
    GLOBAL_SCHEDULER.with(|scheduler| {
        scheduler
            .borrow_mut()
            .get_or_insert_with(AsyncScheduler::new)
            .clone()
    })
}

/// Schedule a task on the global scheduler.
pub fn schedule<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    global_scheduler().schedule(future);
}

/// Schedule a high-priority task on the global scheduler.
pub fn schedule_high_priority<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    global_scheduler().schedule_with_priority(future, Priority::High);
}

