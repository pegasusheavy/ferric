//! Debouncing utilities for async operations.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;

/// Debounced async function wrapper.
pub struct Debounced<F> {
    func: F,
    delay: Duration,
    timeout_id: Rc<RefCell<Option<i32>>>,
}

impl<F> Debounced<F> {
    /// Create a new debounced function.
    pub fn new(func: F, delay: Duration) -> Self {
        Self {
            func,
            delay,
            timeout_id: Rc::new(RefCell::new(None)),
        }
    }

    /// Call the debounced function.
    pub fn call(&self)
    where
        F: Fn() + 'static,
    {
        // Clear existing timeout
        if let Some(_id) = self.timeout_id.borrow_mut().take() {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    window.clear_timeout_with_handle(id);
                }
            }
        }

        // Set new timeout
        let func_clone = &self.func as *const F;
        let _timeout_id_clone = Rc::clone(&self.timeout_id);
        let _ms = self.delay.as_millis() as i32;

        let callback = Closure::once(move || unsafe {
            (*func_clone)();
        });

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    ms,
                ) {
                    *timeout_id_clone.borrow_mut() = Some(id);
                }
            }
        }

        callback.forget();
    }

    /// Cancel any pending call.
    pub fn cancel(&self) {
        if let Some(_id) = self.timeout_id.borrow_mut().take() {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    window.clear_timeout_with_handle(id);
                }
            }
        }
    }
}

/// Create a debounced function.
pub fn debounce<F>(func: F, delay: Duration) -> Debounced<F>
where
    F: Fn() + 'static,
{
    Debounced::new(func, delay)
}

/// Debounce an async function call.
pub fn debounce_async<F, Fut, T>(func: F, _delay: Duration) -> impl Fn() -> Fut
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    move || func()
}

