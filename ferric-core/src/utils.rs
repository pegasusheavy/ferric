//! Utility functions and helpers for the Ferric framework.

/// Set up a panic hook for better error messages in WebAssembly.
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
