//! Chunked App - Main Chunk
//!
//! This is the core chunk that loads immediately.
//! Contains app shell, routing, and home page.

use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console::log_1(&"🦀 Ferric Chunked App - Main Chunk Loaded".into());

    // Initialize app shell
    init_app()?;

    // Setup router
    setup_router()?;

    console::log_1(&"✅ Core initialized. Lazy chunks ready to load.".into());

    Ok(())
}

fn init_app() -> Result<(), JsValue> {
    console::log_1(&"  → Initializing app shell...".into());
    // App initialization code
    Ok(())
}

fn setup_router() -> Result<(), JsValue> {
    console::log_1(&"  → Setting up router...".into());
    // Router setup
    Ok(())
}

/// Public API for loading admin chunk
#[wasm_bindgen]
pub async fn load_admin_chunk() -> Result<(), JsValue> {
    console::log_1(&"📦 Loading admin chunk...".into());

    // Use the global chunk loader
    let result = js_sys::eval(
        "window.FerricChunks.loadChunk('admin')"
    )?;

    let promise = result.dyn_into::<js_sys::Promise>()?;
    wasm_bindgen_futures::JsFuture::from(promise).await?;

    console::log_1(&"✅ Admin chunk loaded".into());
    Ok(())
}

/// Public API for loading dashboard chunk
#[wasm_bindgen]
pub async fn load_dashboard_chunk() -> Result<(), JsValue> {
    console::log_1(&"📦 Loading dashboard chunk...".into());

    let result = js_sys::eval(
        "window.FerricChunks.loadChunk('dashboard')"
    )?;

    let promise = result.dyn_into::<js_sys::Promise>()?;
    wasm_bindgen_futures::JsFuture::from(promise).await?;

    console::log_1(&"✅ Dashboard chunk loaded".into());
    Ok(())
}

