//! Logging macros.

/// Log at trace level.
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Trace,
            $target,
            &format!($($arg)+),
        ));
    };
    ($($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Trace,
            module_path!(),
            &format!($($arg)+),
        ));
    };
}

/// Log at debug level.
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Debug,
            $target,
            &format!($($arg)+),
        ));
    };
    ($($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Debug,
            module_path!(),
            &format!($($arg)+),
        ));
    };
}

/// Log at info level.
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Info,
            $target,
            &format!($($arg)+),
        ));
    };
    ($($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Info,
            module_path!(),
            &format!($($arg)+),
        ));
    };
}

/// Log at warn level.
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Warn,
            $target,
            &format!($($arg)+),
        ));
    };
    ($($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Warn,
            module_path!(),
            &format!($($arg)+),
        ));
    };
}

/// Log at error level.
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Error,
            $target,
            &format!($($arg)+),
        ));
    };
    ($($arg:tt)+) => {
        $crate::logging::log(&$crate::logging::Record::new(
            $crate::logging::Level::Error,
            module_path!(),
            &format!($($arg)+),
        ));
    };
}

/// Check if logging is enabled at the given level.
#[macro_export]
macro_rules! log_enabled {
    ($level:expr) => {
        $crate::logging::enabled($level, module_path!())
    };
    ($level:expr, $target:expr) => {
        $crate::logging::enabled($level, $target)
    };
}

