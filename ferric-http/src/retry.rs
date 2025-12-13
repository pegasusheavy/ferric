//! Retry logic for HTTP requests.
//!
//! Provides configurable retry policies for handling transient failures.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, RetryPolicy, RetryConfig};
//!
//! // Retry with exponential backoff
//! let policy = RetryPolicy::exponential()
//!     .max_retries(3)
//!     .initial_delay_ms(100)
//!     .max_delay_ms(5000);
//!
//! let response = client
//!     .get("https://api.example.com/data")
//!     .with_retry(policy)
//!     .send()
//!     .await?;
//! ```

use crate::{Error, Request, Response, Result};
use std::fmt;
use std::time::Duration;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay between retries (milliseconds).
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds).
    pub max_delay_ms: u64,
    /// Backoff strategy.
    pub strategy: RetryStrategy,
    /// Status codes to retry on.
    pub retry_status_codes: Vec<u16>,
    /// Whether to retry on network errors.
    pub retry_on_network_error: bool,
    /// Whether to retry on timeout.
    pub retry_on_timeout: bool,
    /// Jitter factor (0.0 to 1.0) to add randomness to delays.
    pub jitter: f64,
}

impl RetryConfig {
    /// Create a new retry config with defaults.
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            strategy: RetryStrategy::Exponential { factor: 2.0 },
            retry_status_codes: vec![408, 429, 500, 502, 503, 504],
            retry_on_network_error: true,
            retry_on_timeout: true,
            jitter: 0.1,
        }
    }

    /// Set maximum retries.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Set initial delay in milliseconds.
    pub fn initial_delay_ms(mut self, ms: u64) -> Self {
        self.initial_delay_ms = ms;
        self
    }

    /// Set maximum delay in milliseconds.
    pub fn max_delay_ms(mut self, ms: u64) -> Self {
        self.max_delay_ms = ms;
        self
    }

    /// Set the backoff strategy.
    pub fn strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set status codes to retry on.
    pub fn retry_on_status(mut self, codes: Vec<u16>) -> Self {
        self.retry_status_codes = codes;
        self
    }

    /// Add a status code to retry on.
    pub fn add_retry_status(mut self, code: u16) -> Self {
        if !self.retry_status_codes.contains(&code) {
            self.retry_status_codes.push(code);
        }
        self
    }

    /// Set whether to retry on network errors.
    pub fn retry_network_errors(mut self, retry: bool) -> Self {
        self.retry_on_network_error = retry;
        self
    }

    /// Set whether to retry on timeout.
    pub fn retry_timeout(mut self, retry: bool) -> Self {
        self.retry_on_timeout = retry;
        self
    }

    /// Set jitter factor (0.0 to 1.0).
    pub fn jitter(mut self, factor: f64) -> Self {
        self.jitter = factor.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for a given attempt.
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = match self.strategy {
            RetryStrategy::Fixed => self.initial_delay_ms,
            RetryStrategy::Linear { increment_ms } => {
                self.initial_delay_ms + (attempt as u64 * increment_ms)
            }
            RetryStrategy::Exponential { factor } => {
                (self.initial_delay_ms as f64 * factor.powi(attempt as i32)) as u64
            }
        };

        let capped_delay = base_delay.min(self.max_delay_ms);

        // Add jitter
        let jittered_delay = if self.jitter > 0.0 {
            let jitter_range = (capped_delay as f64 * self.jitter) as u64;
            let jitter = rand_jitter(jitter_range);
            capped_delay.saturating_add(jitter)
        } else {
            capped_delay
        };

        Duration::from_millis(jittered_delay)
    }

    /// Check if a response status should be retried.
    pub fn should_retry_status(&self, status: u16) -> bool {
        self.retry_status_codes.contains(&status)
    }

    /// Check if an error should be retried.
    pub fn should_retry_error(&self, error: &Error) -> bool {
        match error {
            Error::Network(_) => self.retry_on_network_error,
            Error::Timeout => self.retry_on_timeout,
            Error::Status { code, .. } => self.should_retry_status(*code),
            _ => false,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Backoff strategy for retries.
#[derive(Debug, Clone, Copy)]
pub enum RetryStrategy {
    /// Fixed delay between retries.
    Fixed,
    /// Linearly increasing delay.
    Linear {
        /// Increment per attempt (milliseconds).
        increment_ms: u64,
    },
    /// Exponentially increasing delay.
    Exponential {
        /// Multiplier for each attempt.
        factor: f64,
    },
}

impl RetryStrategy {
    /// Create a fixed strategy.
    pub fn fixed() -> Self {
        Self::Fixed
    }

    /// Create a linear strategy.
    pub fn linear(increment_ms: u64) -> Self {
        Self::Linear { increment_ms }
    }

    /// Create an exponential strategy.
    pub fn exponential(factor: f64) -> Self {
        Self::Exponential { factor }
    }
}

/// Pre-configured retry policies.
#[derive(Debug, Clone)]
pub struct RetryPolicy;

impl RetryPolicy {
    /// No retries.
    pub fn none() -> RetryConfig {
        RetryConfig::new().max_retries(0)
    }

    /// Simple retry with fixed delay.
    pub fn fixed(retries: u32, delay_ms: u64) -> RetryConfig {
        RetryConfig::new()
            .max_retries(retries)
            .initial_delay_ms(delay_ms)
            .strategy(RetryStrategy::Fixed)
    }

    /// Exponential backoff (default settings).
    pub fn exponential() -> RetryConfig {
        RetryConfig::new()
    }

    /// Aggressive retry for critical requests.
    pub fn aggressive() -> RetryConfig {
        RetryConfig::new()
            .max_retries(5)
            .initial_delay_ms(50)
            .max_delay_ms(10_000)
            .jitter(0.2)
    }

    /// Conservative retry for non-critical requests.
    pub fn conservative() -> RetryConfig {
        RetryConfig::new()
            .max_retries(2)
            .initial_delay_ms(500)
            .max_delay_ms(5_000)
            .jitter(0.1)
    }

    /// Retry for idempotent operations only (GET, HEAD, OPTIONS).
    pub fn idempotent_only() -> RetryConfig {
        RetryConfig::new().max_retries(3)
    }
}

/// Retry state tracking.
#[derive(Debug, Clone)]
pub struct RetryState {
    /// Current attempt number (0-based).
    pub attempt: u32,
    /// Configuration.
    pub config: RetryConfig,
    /// Last error encountered.
    pub last_error: Option<String>,
    /// Total time spent retrying.
    pub total_delay_ms: u64,
}

impl RetryState {
    /// Create a new retry state.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            attempt: 0,
            config,
            last_error: None,
            total_delay_ms: 0,
        }
    }

    /// Check if more retries are available.
    pub fn has_retries_remaining(&self) -> bool {
        self.attempt < self.config.max_retries
    }

    /// Get the delay for the next retry.
    pub fn next_delay(&self) -> Duration {
        self.config.calculate_delay(self.attempt)
    }

    /// Record a retry attempt.
    pub fn record_attempt(&mut self, error: Option<String>) {
        self.attempt += 1;
        self.last_error = error;
    }

    /// Record delay time.
    pub fn record_delay(&mut self, delay: Duration) {
        self.total_delay_ms += delay.as_millis() as u64;
    }

    /// Check if should retry based on error.
    pub fn should_retry(&self, error: &Error) -> bool {
        self.has_retries_remaining() && self.config.should_retry_error(error)
    }

    /// Check if should retry based on response status.
    pub fn should_retry_response(&self, response: &Response) -> bool {
        self.has_retries_remaining() && self.config.should_retry_status(response.status())
    }

    /// Reset the retry state.
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.last_error = None;
        self.total_delay_ms = 0;
    }
}

/// Information about a retry attempt.
#[derive(Debug, Clone)]
pub struct RetryInfo {
    /// Which attempt this is (1-based).
    pub attempt: u32,
    /// Maximum attempts.
    pub max_attempts: u32,
    /// Delay before this attempt.
    pub delay: Duration,
    /// Reason for retry.
    pub reason: RetryReason,
}

/// Reason for a retry.
#[derive(Debug, Clone)]
pub enum RetryReason {
    /// HTTP status code triggered retry.
    StatusCode(u16),
    /// Network error triggered retry.
    NetworkError(String),
    /// Timeout triggered retry.
    Timeout,
}

impl fmt::Display for RetryReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetryReason::StatusCode(code) => write!(f, "HTTP {}", code),
            RetryReason::NetworkError(msg) => write!(f, "Network error: {}", msg),
            RetryReason::Timeout => write!(f, "Timeout"),
        }
    }
}

/// Callback for retry events.
pub type RetryCallback = Box<dyn Fn(&RetryInfo) + Send + Sync>;

/// Retry executor with callbacks.
pub struct RetryExecutor {
    config: RetryConfig,
    on_retry: Option<RetryCallback>,
}

impl RetryExecutor {
    /// Create a new retry executor.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            on_retry: None,
        }
    }

    /// Set a callback for retry events.
    pub fn on_retry<F>(mut self, callback: F) -> Self
    where
        F: Fn(&RetryInfo) + Send + Sync + 'static,
    {
        self.on_retry = Some(Box::new(callback));
        self
    }

    /// Notify about a retry.
    pub fn notify_retry(&self, info: &RetryInfo) {
        if let Some(ref cb) = self.on_retry {
            cb(info);
        }
    }

    /// Get the retry config.
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

impl fmt::Debug for RetryExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetryExecutor")
            .field("config", &self.config)
            .field("has_callback", &self.on_retry.is_some())
            .finish()
    }
}

// Simple pseudo-random jitter (no external dependencies)
fn rand_jitter(max: u64) -> u64 {
    if max == 0 {
        return 0;
    }

    // Use a simple PRNG based on time
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);
        // Simple xorshift
        let mut x = seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % max
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Use JS Math.random() on WASM
        let random: f64 = js_sys::Math::random();
        (random * max as f64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_delay() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Fixed)
            .initial_delay_ms(100)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_linear_delay() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Linear { increment_ms: 100 })
            .initial_delay_ms(100)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(300));
    }

    #[test]
    fn test_exponential_delay() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Exponential { factor: 2.0 })
            .initial_delay_ms(100)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Exponential { factor: 10.0 })
            .initial_delay_ms(100)
            .max_delay_ms(500)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(500)); // Capped
        assert_eq!(config.calculate_delay(5), Duration::from_millis(500)); // Still capped
    }

    #[test]
    fn test_retry_status_codes() {
        let config = RetryConfig::new();

        assert!(config.should_retry_status(500));
        assert!(config.should_retry_status(503));
        assert!(config.should_retry_status(429));
        assert!(!config.should_retry_status(404));
        assert!(!config.should_retry_status(200));
    }

    #[test]
    fn test_retry_state() {
        let config = RetryConfig::new().max_retries(3);
        let mut state = RetryState::new(config);

        assert!(state.has_retries_remaining());
        assert_eq!(state.attempt, 0);

        state.record_attempt(Some("Error 1".to_string()));
        assert!(state.has_retries_remaining());
        assert_eq!(state.attempt, 1);

        state.record_attempt(Some("Error 2".to_string()));
        state.record_attempt(Some("Error 3".to_string()));
        assert!(!state.has_retries_remaining());
    }

    #[test]
    fn test_retry_policies() {
        let none = RetryPolicy::none();
        assert_eq!(none.max_retries, 0);

        let fixed = RetryPolicy::fixed(5, 200);
        assert_eq!(fixed.max_retries, 5);
        assert_eq!(fixed.initial_delay_ms, 200);

        let aggressive = RetryPolicy::aggressive();
        assert_eq!(aggressive.max_retries, 5);

        let conservative = RetryPolicy::conservative();
        assert_eq!(conservative.max_retries, 2);
    }
}


//!
//! Provides configurable retry policies for handling transient failures.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, RetryPolicy, RetryConfig};
//!
//! // Retry with exponential backoff
//! let policy = RetryPolicy::exponential()
//!     .max_retries(3)
//!     .initial_delay_ms(100)
//!     .max_delay_ms(5000);
//!
//! let response = client
//!     .get("https://api.example.com/data")
//!     .with_retry(policy)
//!     .send()
//!     .await?;
//! ```

use crate::{Error, Request, Response, Result};
use std::fmt;
use std::time::Duration;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay between retries (milliseconds).
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds).
    pub max_delay_ms: u64,
    /// Backoff strategy.
    pub strategy: RetryStrategy,
    /// Status codes to retry on.
    pub retry_status_codes: Vec<u16>,
    /// Whether to retry on network errors.
    pub retry_on_network_error: bool,
    /// Whether to retry on timeout.
    pub retry_on_timeout: bool,
    /// Jitter factor (0.0 to 1.0) to add randomness to delays.
    pub jitter: f64,
}

impl RetryConfig {
    /// Create a new retry config with defaults.
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            strategy: RetryStrategy::Exponential { factor: 2.0 },
            retry_status_codes: vec![408, 429, 500, 502, 503, 504],
            retry_on_network_error: true,
            retry_on_timeout: true,
            jitter: 0.1,
        }
    }

    /// Set maximum retries.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Set initial delay in milliseconds.
    pub fn initial_delay_ms(mut self, ms: u64) -> Self {
        self.initial_delay_ms = ms;
        self
    }

    /// Set maximum delay in milliseconds.
    pub fn max_delay_ms(mut self, ms: u64) -> Self {
        self.max_delay_ms = ms;
        self
    }

    /// Set the backoff strategy.
    pub fn strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set status codes to retry on.
    pub fn retry_on_status(mut self, codes: Vec<u16>) -> Self {
        self.retry_status_codes = codes;
        self
    }

    /// Add a status code to retry on.
    pub fn add_retry_status(mut self, code: u16) -> Self {
        if !self.retry_status_codes.contains(&code) {
            self.retry_status_codes.push(code);
        }
        self
    }

    /// Set whether to retry on network errors.
    pub fn retry_network_errors(mut self, retry: bool) -> Self {
        self.retry_on_network_error = retry;
        self
    }

    /// Set whether to retry on timeout.
    pub fn retry_timeout(mut self, retry: bool) -> Self {
        self.retry_on_timeout = retry;
        self
    }

    /// Set jitter factor (0.0 to 1.0).
    pub fn jitter(mut self, factor: f64) -> Self {
        self.jitter = factor.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for a given attempt.
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = match self.strategy {
            RetryStrategy::Fixed => self.initial_delay_ms,
            RetryStrategy::Linear { increment_ms } => {
                self.initial_delay_ms + (attempt as u64 * increment_ms)
            }
            RetryStrategy::Exponential { factor } => {
                (self.initial_delay_ms as f64 * factor.powi(attempt as i32)) as u64
            }
        };

        let capped_delay = base_delay.min(self.max_delay_ms);

        // Add jitter
        let jittered_delay = if self.jitter > 0.0 {
            let jitter_range = (capped_delay as f64 * self.jitter) as u64;
            let jitter = rand_jitter(jitter_range);
            capped_delay.saturating_add(jitter)
        } else {
            capped_delay
        };

        Duration::from_millis(jittered_delay)
    }

    /// Check if a response status should be retried.
    pub fn should_retry_status(&self, status: u16) -> bool {
        self.retry_status_codes.contains(&status)
    }

    /// Check if an error should be retried.
    pub fn should_retry_error(&self, error: &Error) -> bool {
        match error {
            Error::Network(_) => self.retry_on_network_error,
            Error::Timeout => self.retry_on_timeout,
            Error::Status { code, .. } => self.should_retry_status(*code),
            _ => false,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Backoff strategy for retries.
#[derive(Debug, Clone, Copy)]
pub enum RetryStrategy {
    /// Fixed delay between retries.
    Fixed,
    /// Linearly increasing delay.
    Linear {
        /// Increment per attempt (milliseconds).
        increment_ms: u64,
    },
    /// Exponentially increasing delay.
    Exponential {
        /// Multiplier for each attempt.
        factor: f64,
    },
}

impl RetryStrategy {
    /// Create a fixed strategy.
    pub fn fixed() -> Self {
        Self::Fixed
    }

    /// Create a linear strategy.
    pub fn linear(increment_ms: u64) -> Self {
        Self::Linear { increment_ms }
    }

    /// Create an exponential strategy.
    pub fn exponential(factor: f64) -> Self {
        Self::Exponential { factor }
    }
}

/// Pre-configured retry policies.
#[derive(Debug, Clone)]
pub struct RetryPolicy;

impl RetryPolicy {
    /// No retries.
    pub fn none() -> RetryConfig {
        RetryConfig::new().max_retries(0)
    }

    /// Simple retry with fixed delay.
    pub fn fixed(retries: u32, delay_ms: u64) -> RetryConfig {
        RetryConfig::new()
            .max_retries(retries)
            .initial_delay_ms(delay_ms)
            .strategy(RetryStrategy::Fixed)
    }

    /// Exponential backoff (default settings).
    pub fn exponential() -> RetryConfig {
        RetryConfig::new()
    }

    /// Aggressive retry for critical requests.
    pub fn aggressive() -> RetryConfig {
        RetryConfig::new()
            .max_retries(5)
            .initial_delay_ms(50)
            .max_delay_ms(10_000)
            .jitter(0.2)
    }

    /// Conservative retry for non-critical requests.
    pub fn conservative() -> RetryConfig {
        RetryConfig::new()
            .max_retries(2)
            .initial_delay_ms(500)
            .max_delay_ms(5_000)
            .jitter(0.1)
    }

    /// Retry for idempotent operations only (GET, HEAD, OPTIONS).
    pub fn idempotent_only() -> RetryConfig {
        RetryConfig::new().max_retries(3)
    }
}

/// Retry state tracking.
#[derive(Debug, Clone)]
pub struct RetryState {
    /// Current attempt number (0-based).
    pub attempt: u32,
    /// Configuration.
    pub config: RetryConfig,
    /// Last error encountered.
    pub last_error: Option<String>,
    /// Total time spent retrying.
    pub total_delay_ms: u64,
}

impl RetryState {
    /// Create a new retry state.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            attempt: 0,
            config,
            last_error: None,
            total_delay_ms: 0,
        }
    }

    /// Check if more retries are available.
    pub fn has_retries_remaining(&self) -> bool {
        self.attempt < self.config.max_retries
    }

    /// Get the delay for the next retry.
    pub fn next_delay(&self) -> Duration {
        self.config.calculate_delay(self.attempt)
    }

    /// Record a retry attempt.
    pub fn record_attempt(&mut self, error: Option<String>) {
        self.attempt += 1;
        self.last_error = error;
    }

    /// Record delay time.
    pub fn record_delay(&mut self, delay: Duration) {
        self.total_delay_ms += delay.as_millis() as u64;
    }

    /// Check if should retry based on error.
    pub fn should_retry(&self, error: &Error) -> bool {
        self.has_retries_remaining() && self.config.should_retry_error(error)
    }

    /// Check if should retry based on response status.
    pub fn should_retry_response(&self, response: &Response) -> bool {
        self.has_retries_remaining() && self.config.should_retry_status(response.status())
    }

    /// Reset the retry state.
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.last_error = None;
        self.total_delay_ms = 0;
    }
}

/// Information about a retry attempt.
#[derive(Debug, Clone)]
pub struct RetryInfo {
    /// Which attempt this is (1-based).
    pub attempt: u32,
    /// Maximum attempts.
    pub max_attempts: u32,
    /// Delay before this attempt.
    pub delay: Duration,
    /// Reason for retry.
    pub reason: RetryReason,
}

/// Reason for a retry.
#[derive(Debug, Clone)]
pub enum RetryReason {
    /// HTTP status code triggered retry.
    StatusCode(u16),
    /// Network error triggered retry.
    NetworkError(String),
    /// Timeout triggered retry.
    Timeout,
}

impl fmt::Display for RetryReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetryReason::StatusCode(code) => write!(f, "HTTP {}", code),
            RetryReason::NetworkError(msg) => write!(f, "Network error: {}", msg),
            RetryReason::Timeout => write!(f, "Timeout"),
        }
    }
}

/// Callback for retry events.
pub type RetryCallback = Box<dyn Fn(&RetryInfo) + Send + Sync>;

/// Retry executor with callbacks.
pub struct RetryExecutor {
    config: RetryConfig,
    on_retry: Option<RetryCallback>,
}

impl RetryExecutor {
    /// Create a new retry executor.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            on_retry: None,
        }
    }

    /// Set a callback for retry events.
    pub fn on_retry<F>(mut self, callback: F) -> Self
    where
        F: Fn(&RetryInfo) + Send + Sync + 'static,
    {
        self.on_retry = Some(Box::new(callback));
        self
    }

    /// Notify about a retry.
    pub fn notify_retry(&self, info: &RetryInfo) {
        if let Some(ref cb) = self.on_retry {
            cb(info);
        }
    }

    /// Get the retry config.
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

impl fmt::Debug for RetryExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetryExecutor")
            .field("config", &self.config)
            .field("has_callback", &self.on_retry.is_some())
            .finish()
    }
}

// Simple pseudo-random jitter (no external dependencies)
fn rand_jitter(max: u64) -> u64 {
    if max == 0 {
        return 0;
    }

    // Use a simple PRNG based on time
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);
        // Simple xorshift
        let mut x = seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % max
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Use JS Math.random() on WASM
        let random: f64 = js_sys::Math::random();
        (random * max as f64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_delay() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Fixed)
            .initial_delay_ms(100)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_linear_delay() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Linear { increment_ms: 100 })
            .initial_delay_ms(100)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(300));
    }

    #[test]
    fn test_exponential_delay() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Exponential { factor: 2.0 })
            .initial_delay_ms(100)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig::new()
            .strategy(RetryStrategy::Exponential { factor: 10.0 })
            .initial_delay_ms(100)
            .max_delay_ms(500)
            .jitter(0.0);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(500)); // Capped
        assert_eq!(config.calculate_delay(5), Duration::from_millis(500)); // Still capped
    }

    #[test]
    fn test_retry_status_codes() {
        let config = RetryConfig::new();

        assert!(config.should_retry_status(500));
        assert!(config.should_retry_status(503));
        assert!(config.should_retry_status(429));
        assert!(!config.should_retry_status(404));
        assert!(!config.should_retry_status(200));
    }

    #[test]
    fn test_retry_state() {
        let config = RetryConfig::new().max_retries(3);
        let mut state = RetryState::new(config);

        assert!(state.has_retries_remaining());
        assert_eq!(state.attempt, 0);

        state.record_attempt(Some("Error 1".to_string()));
        assert!(state.has_retries_remaining());
        assert_eq!(state.attempt, 1);

        state.record_attempt(Some("Error 2".to_string()));
        state.record_attempt(Some("Error 3".to_string()));
        assert!(!state.has_retries_remaining());
    }

    #[test]
    fn test_retry_policies() {
        let none = RetryPolicy::none();
        assert_eq!(none.max_retries, 0);

        let fixed = RetryPolicy::fixed(5, 200);
        assert_eq!(fixed.max_retries, 5);
        assert_eq!(fixed.initial_delay_ms, 200);

        let aggressive = RetryPolicy::aggressive();
        assert_eq!(aggressive.max_retries, 5);

        let conservative = RetryPolicy::conservative();
        assert_eq!(conservative.max_retries, 2);
    }
}

