//! Debug mode and development helpers.
//!
//! Provides enhanced logging, validation, and debugging utilities
//! for development builds.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// Global debug mode flag.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Debug mode controller.
pub struct DevMode;

impl DevMode {
    /// Enable debug mode.
    pub fn enable() {
        DEBUG_ENABLED.store(true, Ordering::SeqCst);
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&"[Ferric DevTools] Debug mode enabled".into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("[Ferric DevTools] Debug mode enabled");
        }
    }

    /// Disable debug mode.
    pub fn disable() {
        DEBUG_ENABLED.store(false, Ordering::SeqCst);
    }

    /// Check if debug mode is enabled.
    pub fn is_enabled() -> bool {
        DEBUG_ENABLED.load(Ordering::SeqCst)
    }

    /// Toggle debug mode.
    pub fn toggle() -> bool {
        let current = DEBUG_ENABLED.load(Ordering::SeqCst);
        DEBUG_ENABLED.store(!current, Ordering::SeqCst);
        !current
    }
}

/// Debug configuration options.
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Log all component lifecycle events.
    pub log_lifecycle: bool,
    /// Log all change detection cycles.
    pub log_change_detection: bool,
    /// Log all signal updates.
    pub log_signals: bool,
    /// Log all route changes.
    pub log_routing: bool,
    /// Log all DI resolutions.
    pub log_di: bool,
    /// Enable strict mode (extra validation).
    pub strict_mode: bool,
    /// Show component boundaries in the DOM.
    pub show_component_boundaries: bool,
    /// Highlight changed elements.
    pub highlight_changes: bool,
    /// Break on errors.
    pub break_on_error: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_lifecycle: true,
            log_change_detection: false,
            log_signals: false,
            log_routing: true,
            log_di: false,
            strict_mode: true,
            show_component_boundaries: false,
            highlight_changes: false,
            break_on_error: false,
        }
    }
}

impl DebugConfig {
    /// Create a minimal debug config (less noise).
    pub fn minimal() -> Self {
        Self {
            log_lifecycle: false,
            log_change_detection: false,
            log_signals: false,
            log_routing: false,
            log_di: false,
            strict_mode: false,
            show_component_boundaries: false,
            highlight_changes: false,
            break_on_error: false,
        }
    }

    /// Create a verbose debug config.
    pub fn verbose() -> Self {
        Self {
            log_lifecycle: true,
            log_change_detection: true,
            log_signals: true,
            log_routing: true,
            log_di: true,
            strict_mode: true,
            show_component_boundaries: true,
            highlight_changes: true,
            break_on_error: false,
        }
    }
}

thread_local! {
    static DEBUG_CONFIG: RefCell<DebugConfig> = RefCell::new(DebugConfig::default());
}

/// Set the debug configuration.
pub fn set_debug_config(config: DebugConfig) {
    DEBUG_CONFIG.with(|c| *c.borrow_mut() = config);
}

/// Get the current debug configuration.
pub fn debug_config() -> DebugConfig {
    DEBUG_CONFIG.with(|c| c.borrow().clone())
}

/// Log a debug message if debug mode is enabled.
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::devtools::DevMode::is_enabled() {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&format!("[Ferric] {}", format!($($arg)*)).into());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                println!("[Ferric] {}", format!($($arg)*));
            }
        }
    };
}

/// Log a warning if debug mode is enabled.
#[macro_export]
macro_rules! debug_warn {
    ($($arg:tt)*) => {
        if $crate::devtools::DevMode::is_enabled() {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::warn_1(&format!("[Ferric Warning] {}", format!($($arg)*)).into());
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                eprintln!("[Ferric Warning] {}", format!($($arg)*));
            }
        }
    };
}

/// Log an error (always, regardless of debug mode).
#[macro_export]
macro_rules! debug_error {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::error_1(&format!("[Ferric Error] {}", format!($($arg)*)).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            eprintln!("[Ferric Error] {}", format!($($arg)*));
        }
    };
}

/// Debug assertion that only runs in debug mode.
#[macro_export]
macro_rules! debug_assert_ferric {
    ($condition:expr) => {
        if $crate::devtools::DevMode::is_enabled() && $crate::devtools::debug_config().strict_mode {
            debug_assert!($condition);
        }
    };
    ($condition:expr, $($arg:tt)*) => {
        if $crate::devtools::DevMode::is_enabled() && $crate::devtools::debug_config().strict_mode {
            debug_assert!($condition, $($arg)*);
        }
    };
}

/// Validation helper for development.
pub struct DebugValidator;

impl DebugValidator {
    /// Validate that a value is not None.
    pub fn require_some<T>(value: Option<T>, name: &str) -> Option<T> {
        if DevMode::is_enabled() && value.is_none() {
            debug_warn!("Required value '{}' is None", name);
        }
        value
    }

    /// Validate that a string is not empty.
    pub fn require_non_empty(value: &str, name: &str) -> bool {
        if DevMode::is_enabled() && value.is_empty() {
            debug_warn!("Required string '{}' is empty", name);
            return false;
        }
        true
    }

    /// Validate that a collection is not empty.
    pub fn require_non_empty_vec<T>(value: &[T], name: &str) -> bool {
        if DevMode::is_enabled() && value.is_empty() {
            debug_warn!("Required collection '{}' is empty", name);
            return false;
        }
        true
    }

    /// Validate a condition.
    pub fn check(condition: bool, message: &str) {
        if DevMode::is_enabled() && !condition {
            debug_warn!("Validation failed: {}", message);
        }
    }
}

/// Debug counter for tracking occurrences.
pub struct DebugCounter {
    name: String,
    count: AtomicU64,
}

impl DebugCounter {
    /// Create a new counter.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            count: AtomicU64::new(0),
        }
    }

    /// Increment the counter.
    pub fn increment(&self) -> u64 {
        let new_count = self.count.fetch_add(1, Ordering::SeqCst) + 1;
        if DevMode::is_enabled() {
            debug_log!("{}: {}", self.name, new_count);
        }
        new_count
    }

    /// Get the current count.
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
    }

    /// Reset the counter.
    pub fn reset(&self) {
        self.count.store(0, Ordering::SeqCst);
    }
}

/// Debug timer for measuring durations.
pub struct DebugTimer {
    name: String,
    start: std::time::Instant,
    checkpoints: RefCell<Vec<(String, Duration)>>,
}

impl DebugTimer {
    /// Start a new timer.
    pub fn start(name: &str) -> Self {
        if DevMode::is_enabled() {
            debug_log!("Timer '{}' started", name);
        }
        Self {
            name: name.to_string(),
            start: std::time::Instant::now(),
            checkpoints: RefCell::new(Vec::new()),
        }
    }

    /// Record a checkpoint.
    pub fn checkpoint(&self, label: &str) {
        let elapsed = self.start.elapsed();
        self.checkpoints.borrow_mut().push((label.to_string(), elapsed));
        if DevMode::is_enabled() {
            debug_log!("Timer '{}' checkpoint '{}': {:?}", self.name, label, elapsed);
        }
    }

    /// Get elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and log the duration.
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        if DevMode::is_enabled() {
            debug_log!("Timer '{}' stopped: {:?}", self.name, elapsed);
        }
        elapsed
    }
}

/// Macro for timing a block of code.
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:expr) => {{
        let _timer = $crate::devtools::DebugTimer::start($name);
        let result = $block;
        result
    }};
}

/// Debug breakpoint (only in debug mode).
pub fn breakpoint() {
    if DevMode::is_enabled() && debug_config().break_on_error {
        #[cfg(target_arch = "wasm32")]
        {
            // Use debugger statement via JS
            web_sys::console::log_1(&"[Ferric] Breakpoint hit".into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // On native, we can't really break, but we can print
            eprintln!("[Ferric] Breakpoint hit - attach debugger");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_mode() {
        DevMode::disable();
        assert!(!DevMode::is_enabled());

        DevMode::enable();
        assert!(DevMode::is_enabled());

        DevMode::disable();
    }

    #[test]
    fn test_debug_config() {
        let config = DebugConfig::default();
        assert!(config.log_lifecycle);
        assert!(config.strict_mode);

        let minimal = DebugConfig::minimal();
        assert!(!minimal.log_lifecycle);
        assert!(!minimal.strict_mode);
    }

    #[test]
    fn test_debug_counter() {
        let counter = DebugCounter::new("test");
        assert_eq!(counter.count(), 0);

        counter.increment();
        assert_eq!(counter.count(), 1);

        counter.increment();
        counter.increment();
        assert_eq!(counter.count(), 3);

        counter.reset();
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn test_debug_timer() {
        let timer = DebugTimer::start("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.stop();
        assert!(elapsed.as_millis() >= 10);
    }
}


