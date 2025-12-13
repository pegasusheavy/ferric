//! Development tools for debugging and profiling Ferric applications.
//!
//! This module provides utilities for:
//! - Debug mode with enhanced logging and validation
//! - Component inspection in browser devtools
//! - Performance profiling of change detection
//!
//! ## Features
//!
//! All devtools are designed to have minimal overhead in production builds.
//! Use the `debug_assertions` feature flag to enable/disable devtools.
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_core::devtools::*;
//!
//! // Enable debug mode
//! DevMode::enable();
//!
//! // Start profiling
//! let profiler = Profiler::new();
//! profiler.start_recording();
//!
//! // ... run your app ...
//!
//! let report = profiler.stop_recording();
//! report.print_summary();
//! ```

mod debug;
mod inspector;
mod profiler;
mod console;
mod state;

pub use debug::*;
pub use inspector::*;
pub use profiler::*;
pub use console::*;
pub use state::*;

/// Check if devtools are enabled (debug build).
#[inline]
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions) || DevMode::is_enabled()
}

/// Initialize devtools for the application.
pub fn init_devtools() {
    if is_dev_mode() {
        DevMode::enable();
        #[cfg(target_arch = "wasm32")]
        {
            ComponentInspector::install();
        }
    }
}


