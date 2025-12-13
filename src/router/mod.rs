//! Client-side routing for Ferric applications.
//!
//! Provides navigation, route guards, and URL parameter extraction.

mod config;
mod guard;
mod outlet;
mod service;

pub use config::*;
pub use guard::*;
pub use outlet::*;
pub use service::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Initialize the router and set up navigation event listeners.
pub fn init_router() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global window exists");

    // Listen for popstate events (back/forward navigation)
    let closure = Closure::wrap(Box::new(move |_event: web_sys::PopStateEvent| {
        // Handle navigation
        if let Some(window) = web_sys::window() {
            if let Ok(pathname) = window.location().pathname() {
                // TODO: Trigger route change
                web_sys::console::log_1(&format!("Navigation to: {}", pathname).into());
            }
        }
    }) as Box<dyn FnMut(_)>);

    window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

