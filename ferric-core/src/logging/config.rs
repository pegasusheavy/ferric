//! Logging configuration.

use super::Level;
use std::collections::HashMap;

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Minimum log level.
    pub level: Level,
    /// Target-specific log levels.
    pub target_levels: HashMap<String, Level>,
    /// Include timestamps in output.
    pub show_timestamp: bool,
    /// Include target name in output.
    pub show_target: bool,
    /// Use colors in output.
    pub use_colors: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::Info,
            target_levels: HashMap::new(),
            show_timestamp: true,
            show_target: true,
            use_colors: true,
        }
    }
}

impl LogConfig {
    /// Create a new config with the given level.
    pub fn with_level(level: Level) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    /// Set the target filter.
    pub fn target_level(mut self, target: impl Into<String>, level: Level) -> Self {
        self.target_levels.insert(target.into(), level);
        self
    }

    /// Enable or disable timestamps.
    pub fn timestamps(mut self, enable: bool) -> Self {
        self.show_timestamp = enable;
        self
    }

    /// Enable or disable target name.
    pub fn target(mut self, enable: bool) -> Self {
        self.show_target = enable;
        self
    }

    /// Enable or disable colors.
    pub fn colors(mut self, enable: bool) -> Self {
        self.use_colors = enable;
        self
    }
}

/// Configure the global logging system.
pub fn configure(config: LogConfig) {
    super::GLOBAL_CONFIG.with(|global| {
        *global.borrow_mut() = config;
    });
}
