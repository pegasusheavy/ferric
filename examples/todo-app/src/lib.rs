//! Todo List Application
//!
//! A reactive todo list demonstrating Ferric's signals, computed values, and effects.
//!
//! ## Features
//!
//! - **Reactive State**: All state managed with `Signal<T>`
//! - **Computed Values**: Derived state with automatic dependency tracking
//! - **Effects**: Side effects for DOM updates and localStorage persistence
//! - **Batching**: Multiple updates batched for efficiency
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        TodoStore                             │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │   Signals   │  │  Computed   │  │      Effects        │  │
//! │  │  - todos    │  │  - filtered │  │  - persistence      │  │
//! │  │  - filter   │──│  - stats    │──│  - dom updates      │  │
//! │  │  - editing  │  │  - flags    │  │                     │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use wasm_bindgen::prelude::*;

mod app;
mod components;
mod models;
mod services;
mod store;
mod template;

pub use app::TodoApp;
pub use store::TodoStore;

/// Initialize the application
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Log startup
    web_sys::console::log_1(&"🦀 Reactive Todo App starting...".into());

    // Bootstrap the application
    let app = TodoApp::new();
    app.mount("#app")?;

    web_sys::console::log_1(&"✅ Todo App mounted with reactive signals!".into());

    Ok(())
}
