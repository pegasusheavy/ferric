//! Async queue for task management.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::rc::Rc;

/// An async queue for managing tasks.
pub struct AsyncQueue<T> {
    queue: Rc<RefCell<VecDeque<T>>>,
    processing: Rc<RefCell<bool>>,
}

impl<T> AsyncQueue<T> {
    /// Create a new async queue.
    pub fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
            processing: Rc::new(RefCell::new(false)),
        }
    }

    /// Add an item to the queue.
    pub fn enqueue(&self, item: T) {
        self.queue.borrow_mut().push_back(item);
    }

    /// Remove and return the next item.
    pub fn dequeue(&self) -> Option<T> {
        self.queue.borrow_mut().pop_front()
    }

    /// Get the queue length.
    pub fn len(&self) -> usize {
        self.queue.borrow().len()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// Clear the queue.
    pub fn clear(&self) {
        self.queue.borrow_mut().clear();
    }

    /// Process the queue with an async function.
    pub async fn process<F, Fut>(&self, processor: F)
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = ()>,
    {
        if *self.processing.borrow() {
            return;
        }

        *self.processing.borrow_mut() = true;

        while let Some(item) = self.dequeue() {
            processor(item).await;
        }

        *self.processing.borrow_mut() = false;
    }
}

impl<T> Default for AsyncQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for AsyncQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Rc::clone(&self.queue),
            processing: Rc::clone(&self.processing),
        }
    }
}

