//! Logger instances for scoped logging.

use super::{log, Level, Record};
use super::record::Value;
use std::rc::Rc;

/// A logger instance that can be cloned and passed around.
///
/// Loggers have a target (typically a module or component name) and can
/// carry contextual key-value pairs that are added to all log records.
#[derive(Clone)]
pub struct Logger {
    inner: Rc<LoggerInner>,
}

struct LoggerInner {
    /// The target name for this logger.
    target: String,

    /// Parent logger for hierarchical logging.
    parent: Option<Rc<LoggerInner>>,

    /// Contextual key-value pairs attached to all logs from this logger.
    context: Vec<(String, String)>,
}

impl Logger {
    /// Create the root logger (no target).
    pub(crate) fn root() -> Self {
        Self {
            inner: Rc::new(LoggerInner {
                target: String::new(),
                parent: None,
                context: Vec::new(),
            }),
        }
    }

    /// Create a new logger with the given target.
    ///
    /// The target is typically a module path or component name.
    ///
    /// # Example
    /// ```ignore
    /// let logger = Logger::new("my_component");
    /// logger.info("Initialized");
    /// ```
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(LoggerInner {
                target: target.into(),
                parent: None,
                context: Vec::new(),
            }),
        }
    }

    /// Create a child logger with a sub-target.
    ///
    /// The child's target is `parent::child`.
    ///
    /// # Example
    /// ```ignore
    /// let logger = Logger::new("app");
    /// let child = logger.child("network");
    /// // child's target is "app::network"
    /// ```
    pub fn child(&self, sub_target: impl AsRef<str>) -> Self {
        let target = if self.inner.target.is_empty() {
            sub_target.as_ref().to_string()
        } else {
            format!("{}::{}", self.inner.target, sub_target.as_ref())
        };

        Self {
            inner: Rc::new(LoggerInner {
                target,
                parent: Some(Rc::clone(&self.inner)),
                context: Vec::new(),
            }),
        }
    }

    /// Create a logger with additional context.
    ///
    /// Context key-value pairs are included in all log records.
    ///
    /// # Example
    /// ```ignore
    /// let logger = Logger::new("request")
    ///     .with_context("request_id", "abc123")
    ///     .with_context("user_id", "42");
    /// logger.info("Processing"); // includes request_id and user_id
    /// ```
    pub fn with_context(self, key: impl Into<String>, value: impl ToString) -> Self {
        let mut context = self.inner.context.clone();
        context.push((key.into(), value.to_string()));

        Self {
            inner: Rc::new(LoggerInner {
                target: self.inner.target.clone(),
                parent: self.inner.parent.clone(),
                context,
            }),
        }
    }

    /// Get the logger's target.
    pub fn target(&self) -> &str {
        &self.inner.target
    }

    /// Log a message at the given level.
    pub fn log(&self, level: Level, message: &str) {
        // Collect context and build record
        let context = self.collect_context();
        let mut record = Record::new(level, &self.inner.target, message);

        // Add context key-value pairs
        // We store the owned strings temporarily and reference them
        let context_kvs: Vec<_> = context.iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();

        for (key, value) in &context_kvs {
            record.kvs.push(super::record::KeyValue {
                key,
                value: Value::String(value.clone()),
            });
        }

        log(&record);
    }

    /// Log a formatted message at the given level.
    pub fn log_fmt(&self, level: Level, args: std::fmt::Arguments<'_>) {
        let message = args.to_string();
        self.log(level, &message);
    }

    /// Log at TRACE level.
    #[inline]
    pub fn trace(&self, message: &str) {
        self.log(Level::Trace, message);
    }

    /// Log at DEBUG level.
    #[inline]
    pub fn debug(&self, message: &str) {
        self.log(Level::Debug, message);
    }

    /// Log at INFO level.
    #[inline]
    pub fn info(&self, message: &str) {
        self.log(Level::Info, message);
    }

    /// Log at WARN level.
    #[inline]
    pub fn warn(&self, message: &str) {
        self.log(Level::Warn, message);
    }

    /// Log at ERROR level.
    #[inline]
    pub fn error(&self, message: &str) {
        self.log(Level::Error, message);
    }

    /// Log at TRACE level with formatting.
    #[inline]
    pub fn trace_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.log_fmt(Level::Trace, args);
    }

    /// Log at DEBUG level with formatting.
    #[inline]
    pub fn debug_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.log_fmt(Level::Debug, args);
    }

    /// Log at INFO level with formatting.
    #[inline]
    pub fn info_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.log_fmt(Level::Info, args);
    }

    /// Log at WARN level with formatting.
    #[inline]
    pub fn warn_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.log_fmt(Level::Warn, args);
    }

    /// Log at ERROR level with formatting.
    #[inline]
    pub fn error_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.log_fmt(Level::Error, args);
    }

    /// Collect context key-value pairs from this logger and its parents.
    ///
    /// Returns a vector of (key, value) pairs in parent-first order.
    fn collect_context(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();

        // Walk up parent chain (collecting in reverse)
        let mut contexts_stack: Vec<&[(String, String)]> = Vec::new();

        if !self.inner.context.is_empty() {
            contexts_stack.push(&self.inner.context);
        }

        let mut current = self.inner.parent.as_ref();
        while let Some(parent) = current {
            if !parent.context.is_empty() {
                contexts_stack.push(&parent.context);
            }
            current = parent.parent.as_ref();
        }

        // Add in reverse order (parent first)
        for ctx in contexts_stack.into_iter().rev() {
            for (key, value) in ctx {
                result.push((key.clone(), value.clone()));
            }
        }

        result
    }
}

impl std::fmt::Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("target", &self.inner.target)
            .field("context_len", &self.inner.context.len())
            .finish()
    }
}

/// Convenience trait for logging on any type that has a logger.
pub trait Loggable {
    /// Get the logger for this object.
    fn logger(&self) -> &Logger;

    /// Log at TRACE level.
    fn log_trace(&self, message: &str) {
        self.logger().trace(message);
    }

    /// Log at DEBUG level.
    fn log_debug(&self, message: &str) {
        self.logger().debug(message);
    }

    /// Log at INFO level.
    fn log_info(&self, message: &str) {
        self.logger().info(message);
    }

    /// Log at WARN level.
    fn log_warn(&self, message: &str) {
        self.logger().warn(message);
    }

    /// Log at ERROR level.
    fn log_error(&self, message: &str) {
        self.logger().error(message);
    }
}
