//! Log output implementations.

use super::{LogConfig, Record};

/// Trait for log outputs.
pub trait LogOutput {
    /// Write a log record.
    fn write(&self, record: &Record, config: &LogConfig);
}

/// Console output for logging.
pub struct ConsoleOutput;

impl LogOutput for ConsoleOutput {
    fn write(&self, record: &Record, config: &LogConfig) {
        log_to_console(record, config);
    }
}

/// Log a record to the browser console.
pub fn log_to_console(record: &Record, config: &LogConfig) {
    let mut output = String::new();

    if config.show_target && !record.target.is_empty() {
        output.push_str(&format!("[{}] ", record.target));
    }

    output.push_str(&format!("{}: {}", record.level, record.message));

    // Add key-value pairs
    if !record.kvs.is_empty() {
        output.push_str(" {");
        for (i, kv) in record.kvs.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("{}={:?}", kv.key, kv.value));
        }
        output.push('}');
    }

    // Use web_sys console based on level
    #[cfg(target_arch = "wasm32")]
    {
        use super::Level;
        match record.level {
            Level::Error => web_sys::console::error_1(&output.into()),
            Level::Warn => web_sys::console::warn_1(&output.into()),
            Level::Info => web_sys::console::info_1(&output.into()),
            Level::Debug | Level::Trace => web_sys::console::log_1(&output.into()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", output);
    }
}
