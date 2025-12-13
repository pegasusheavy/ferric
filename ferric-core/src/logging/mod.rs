//! Memory-safe logging system for Ferric.
//!
//! This module provides a comprehensive logging API that is:
//! - **Memory safe**: No unsafe code, uses Rust's ownership model
//! - **Allocation-efficient**: Lazy formatting via macros
//! - **Configurable**: Filter by level and target
//! - **Structured**: Support for key-value pairs
//! - **Scoped**: Create child loggers with context
//!
//! # Quick Start
//!
//! ```ignore
//! use ferric::logging::*;
//!
//! // Use the global logger
//! info!("Application started");
//! warn!("Low memory: {} MB remaining", 42);
//! error!(target: "network", "Connection failed: {}", err);
//!
//! // With structured data
//! info!(user_id = 123, action = "login"; "User logged in");
//!
//! // Create a scoped logger
//! let logger = Logger::new("my_component");
//! logger.info("Component initialized");
//! ```
//!
//! # Configuration
//!
//! ```ignore
//! use ferric::logging::*;
//!
//! // Set global log level
//! set_log_level(Level::Debug);
//!
//! // Set level for specific targets
//! set_target_level("network", Level::Trace);
//! set_target_level("render", Level::Warn);
//!
//! // Configure output format
//! configure(LogConfig {
//!     show_timestamp: true,
//!     show_target: true,
//!     use_colors: true,
//!     ..Default::default()
//! });
//! ```

mod config;
mod level;
mod logger;
mod macros;
mod output;
mod record;

pub use config::{configure, LogConfig};
pub use level::Level;
pub use logger::Logger;
pub use output::{ConsoleOutput, LogOutput};
pub use record::{Record, RecordBuilder};

use std::cell::RefCell;

// Thread-local global state (safe in WASM's single-threaded environment)
thread_local! {
    static GLOBAL_CONFIG: RefCell<LogConfig> = RefCell::new(LogConfig::default());
    static GLOBAL_LOGGER: RefCell<Logger> = RefCell::new(Logger::root());
}

/// Set the global log level.
///
/// Messages below this level will be filtered out.
pub fn set_log_level(level: Level) {
    GLOBAL_CONFIG.with(|config| {
        config.borrow_mut().level = level;
    });
}

/// Get the current global log level.
pub fn log_level() -> Level {
    GLOBAL_CONFIG.with(|config| config.borrow().level)
}

/// Set the log level for a specific target.
///
/// This allows fine-grained control over logging verbosity for different
/// parts of your application.
pub fn set_target_level(target: &str, level: Level) {
    GLOBAL_CONFIG.with(|config| {
        config.borrow_mut().target_levels.insert(target.to_string(), level);
    });
}

/// Check if a message at the given level and target would be logged.
///
/// Use this for expensive formatting operations:
/// ```ignore
/// if log_enabled!(Level::Debug, "expensive") {
///     let data = expensive_computation();
///     debug!(target: "expensive", "{:?}", data);
/// }
/// ```
pub fn enabled(level: Level, target: &str) -> bool {
    GLOBAL_CONFIG.with(|config| {
        let config = config.borrow();

        // Check target-specific level first
        if let Some(target_level) = config.target_levels.get(target) {
            return level >= *target_level;
        }

        // Fall back to global level
        level >= config.level
    })
}

/// Log a record to the global logger.
///
/// This is typically called by the logging macros, not directly.
pub fn log(record: &Record) {
    if !enabled(record.level, record.target) {
        return;
    }

    GLOBAL_CONFIG.with(|config| {
        let config = config.borrow();
        output::log_to_console(record, &config);
    });
}

/// Get the global logger.
pub fn global_logger() -> Logger {
    GLOBAL_LOGGER.with(|logger| logger.borrow().clone())
}

// Note: Logging macros (trace!, debug!, info!, warn!, error!, log_enabled!)
// are defined in macros.rs and exported at the crate root level.
// Import them via `use ferric::prelude::*` or `use ferric::{trace, debug, info, warn, error}`.

// Re-export record types that macros need
pub use record::{KeyValue, Value};
