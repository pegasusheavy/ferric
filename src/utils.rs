//! Utility functions and helpers for the Ferric framework.

/// Set up a panic hook for better error messages in WebAssembly.
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Log a message to the browser console.
pub fn log(message: &str) {
    web_sys::console::log_1(&message.into());
}

/// Log an error to the browser console.
pub fn error(message: &str) {
    web_sys::console::error_1(&message.into());
}

/// Log a warning to the browser console.
pub fn warn(message: &str) {
    web_sys::console::warn_1(&message.into());
}

