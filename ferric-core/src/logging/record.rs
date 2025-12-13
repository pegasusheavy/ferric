//! Log record types.

use super::Level;

/// A value in a key-value pair.
#[derive(Debug, Clone)]
pub enum Value {
    /// A string value.
    String(String),
    /// An integer value.
    Int(i64),
    /// A float value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
}

/// A key-value pair for structured logging.
#[derive(Debug, Clone)]
pub struct KeyValue<'a> {
    /// The key.
    pub key: &'a str,
    /// The value.
    pub value: Value,
}

/// A log record.
#[derive(Debug)]
pub struct Record<'a> {
    /// The log level.
    pub level: Level,
    /// The target (module/component name).
    pub target: &'a str,
    /// The log message.
    pub message: &'a str,
    /// Key-value pairs for structured logging.
    pub kvs: Vec<KeyValue<'a>>,
}

impl<'a> Record<'a> {
    /// Create a new log record.
    pub fn new(level: Level, target: &'a str, message: &'a str) -> Self {
        Self {
            level,
            target,
            message,
            kvs: Vec::new(),
        }
    }
}

/// Builder for log records.
pub struct RecordBuilder<'a> {
    level: Level,
    target: &'a str,
    message: &'a str,
    kvs: Vec<KeyValue<'a>>,
}

impl<'a> RecordBuilder<'a> {
    /// Create a new record builder.
    pub fn new() -> Self {
        Self {
            level: Level::Info,
            target: "",
            message: "",
            kvs: Vec::new(),
        }
    }

    /// Set the log level.
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the target.
    pub fn target(mut self, target: &'a str) -> Self {
        self.target = target;
        self
    }

    /// Set the message.
    pub fn message(mut self, message: &'a str) -> Self {
        self.message = message;
        self
    }

    /// Build the record.
    pub fn build(self) -> Record<'a> {
        Record {
            level: self.level,
            target: self.target,
            message: self.message,
            kvs: self.kvs,
        }
    }
}

impl<'a> Default for RecordBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
