//! Comprehensive async support for Ferric framework.
//!
//! This module provides Promise-based async primitives,WASM interop,
//! and utilities for building async-first applications.

pub mod promise;
pub mod future_ext;
pub mod spawn;
pub mod timeout;
pub mod debounce;
pub mod throttle;
pub mod queue;
pub mod scheduler;

pub use promise::*;
pub use future_ext::*;
pub use spawn::*;
pub use timeout::*;
pub use debounce::*;
pub use throttle::*;
pub use queue::*;
pub use scheduler::*;

/// Prelude for async support.
pub mod prelude {
    pub use super::promise::{Promise, PromiseState, Deferred};
    pub use super::future_ext::FutureExt;
    pub use super::spawn::{spawn_local_task, spawn_local_with_handle, spawn_local_result};
    pub use super::timeout::{timeout, sleep, Timeout};
    pub use super::debounce::{Debounced, debounce};
    pub use super::throttle::{Throttled, throttle};
    pub use super::queue::AsyncQueue;
    pub use super::scheduler::{AsyncScheduler, Priority, schedule, schedule_high_priority};
}

