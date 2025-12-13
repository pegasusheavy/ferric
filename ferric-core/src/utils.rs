//! Utility functions and helpers for the Ferric framework.

/// Set up a panic hook for better error messages in WebAssembly.
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Note: The logging functions below are deprecated.
// Use the new logging module instead: `use ferric::prelude::*` and then
// `info!("message")`, `warn!("message")`, `error!("message")` etc.

/// Log a message to the browser console.
///
/// # Deprecated
/// Use `info!("message")` from the logging module instead.
#[deprecated(since = "0.2.0", note = "Use the logging module: `info!(\"message\")`")]
pub fn log(message: &str) {
    web_sys::console::log_1(&message.into());
}

/// Log an error to the browser console.
///
/// # Deprecated
/// Use `error!("message")` from the logging module instead.
#[deprecated(since = "0.2.0", note = "Use the logging module: `error!(\"message\")`")]
pub fn error_log(message: &str) {
    web_sys::console::error_1(&message.into());
}

/// Log a warning to the browser console.
///
/// # Deprecated
/// Use `warn!("message")` from the logging module instead.
#[deprecated(since = "0.2.0", note = "Use the logging module: `warn!(\"message\")`")]
pub fn warn_log(message: &str) {
    web_sys::console::warn_1(&message.into());
}
