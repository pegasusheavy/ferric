//! DOM manipulation utilities for Ferric.
//!
//! Provides safe wrappers around web-sys for common DOM operations.

mod element;
mod events;
mod query;

pub use element::*;
pub use events::*;
pub use query::*;

use wasm_bindgen::prelude::*;

/// Get the global window object.
pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global window exists")
}

/// Get the document object.
pub fn document() -> web_sys::Document {
    window().document().expect("should have a document")
}

/// Get the document body.
pub fn body() -> web_sys::HtmlElement {
    document().body().expect("should have a body")
}

/// Request an animation frame.
pub fn request_animation_frame(callback: &Closure<dyn FnMut()>) -> i32 {
    window()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .expect("should register animation frame callback")
}

/// Set a timeout.
pub fn set_timeout(callback: &Closure<dyn FnMut()>, millis: i32) -> i32 {
    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            millis,
        )
        .expect("should set timeout")
}

/// Set an interval.
pub fn set_interval(callback: &Closure<dyn FnMut()>, millis: i32) -> i32 {
    window()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            millis,
        )
        .expect("should set interval")
}

/// Clear a timeout.
pub fn clear_timeout(id: i32) {
    window().clear_timeout_with_handle(id);
}

/// Clear an interval.
pub fn clear_interval(id: i32) {
    window().clear_interval_with_handle(id);
}

