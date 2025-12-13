//! Event handling utilities.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;

/// Add an event listener to an element.
pub fn add_event_listener<F>(
    element: &web_sys::EventTarget,
    event_type: &str,
    callback: F,
) -> Result<EventListener, JsValue>
where
    F: FnMut(web_sys::Event) + 'static,
{
    let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
    element.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())?;

    Ok(EventListener {
        target: element.clone(),
        event_type: event_type.to_string(),
        closure: Rc::new(closure),
    })
}

/// An event listener that can be removed.
pub struct EventListener {
    target: web_sys::EventTarget,
    event_type: String,
    closure: Rc<Closure<dyn FnMut(web_sys::Event)>>,
}

impl EventListener {
    /// Remove this event listener.
    pub fn remove(&self) -> Result<(), JsValue> {
        self.target.remove_event_listener_with_callback(
            &self.event_type,
            self.closure.as_ref().as_ref().unchecked_ref(),
        )
    }
}

impl Drop for EventListener {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

/// Event emitter for component outputs.
pub struct EventEmitter<T> {
    listeners: Rc<RefCell<Vec<Box<dyn Fn(&T)>>>>,
}

impl<T> EventEmitter<T> {
    /// Create a new event emitter.
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Emit an event to all listeners.
    pub fn emit(&self, value: &T) {
        for listener in self.listeners.borrow().iter() {
            listener(value);
        }
    }

    /// Subscribe to events.
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&T) + 'static,
    {
        self.listeners.borrow_mut().push(Box::new(callback));
    }
}

impl<T> Default for EventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for EventEmitter<T> {
    fn clone(&self) -> Self {
        Self {
            listeners: Rc::clone(&self.listeners),
        }
    }
}

/// Prevent the default action of an event.
pub fn prevent_default(event: &web_sys::Event) {
    event.prevent_default();
}

/// Stop event propagation.
pub fn stop_propagation(event: &web_sys::Event) {
    event.stop_propagation();
}

/// Stop immediate event propagation.
pub fn stop_immediate_propagation(event: &web_sys::Event) {
    event.stop_immediate_propagation();
}

/// Get the target element of an event.
pub fn event_target<T: JsCast>(event: &web_sys::Event) -> Option<T> {
    event.target()?.dyn_into::<T>().ok()
}

