//! Ferric Documentation Website
//!
//! A showcase of the Ferric framework built with Ferric itself.

use wasm_bindgen::prelude::*;

mod app;
mod components;
mod pages;

pub use app::App;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Mount the application
    App::mount("#app")?;

    Ok(())
}
