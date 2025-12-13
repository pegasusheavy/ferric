//! # Ferric Reactivity Demo
//!
//! A comprehensive demonstration of Ferric's reactive system featuring:
//!
//! - **Signals**: Core reactive primitives for mutable state
//! - **Computed**: Derived values with automatic dependency tracking
//! - **Effects**: Side effects that run when dependencies change
//! - **Batching**: Multiple updates coalesced into single notification
//! - **Watch**: Observe specific values with old/new comparisons
//! - **Resources**: Async data fetching with loading/error states
//! - **Untracked**: Read signals without creating dependencies
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Reactive Data Flow                            │
//! │                                                                  │
//! │   User Input ──► Signal.set() ──► Computed recalc               │
//! │                        │               │                         │
//! │                        ▼               ▼                         │
//! │                   Effects run    DOM Updates                     │
//! │                        │                                         │
//! │                        ▼                                         │
//! │               localStorage sync                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use wasm_bindgen::prelude::*;

mod counter;
mod shopping_cart;
mod async_search;
mod timer;
mod form_validation;

pub use counter::CounterDemo;
pub use shopping_cart::ShoppingCartDemo;
pub use async_search::AsyncSearchDemo;
pub use timer::TimerDemo;
pub use form_validation::FormValidationDemo;

/// Initialize the demo application.
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"🦀 Ferric Reactivity Demo starting...".into());

    // Run all demos
    let document = web_sys::window()
        .ok_or("No window")?
        .document()
        .ok_or("No document")?;

    // Each demo mounts to its own container
    if let Some(el) = document.get_element_by_id("counter-demo") {
        CounterDemo::mount(&el)?;
    }

    if let Some(el) = document.get_element_by_id("cart-demo") {
        ShoppingCartDemo::mount(&el)?;
    }

    if let Some(el) = document.get_element_by_id("search-demo") {
        AsyncSearchDemo::mount(&el)?;
    }

    if let Some(el) = document.get_element_by_id("timer-demo") {
        TimerDemo::mount(&el)?;
    }

    if let Some(el) = document.get_element_by_id("form-demo") {
        FormValidationDemo::mount(&el)?;
    }

    web_sys::console::log_1(&"✅ All demos mounted!".into());

    Ok(())
}

