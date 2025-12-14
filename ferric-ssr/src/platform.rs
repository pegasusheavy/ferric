//! Platform detection utilities for isomorphic/universal applications.
//!
//! This module provides utilities to detect whether code is running on the server
//! (during SSR) or in the browser (client-side), similar to Angular's
//! `isPlatformServer` and `isPlatformBrowser`.
//!
//! # Usage
//!
//! ```ignore
//! use ferric_ssr::platform::{is_server, is_browser, Platform};
//!
//! // Simple boolean checks
//! if is_server() {
//!     println!("Running on server");
//! }
//!
//! if is_browser() {
//!     // Access browser APIs safely
//!     web_sys::window().unwrap();
//! }
//!
//! // Or use the Platform enum
//! match Platform::current() {
//!     Platform::Server => println!("SSR mode"),
//!     Platform::Browser => println!("Client mode"),
//! }
//! ```
//!
//! # Conditional Rendering
//!
//! ```ignore
//! use ferric_ssr::platform::{is_browser, render_on_browser, render_on_server};
//!
//! // Only render certain content on specific platforms
//! let server_only = render_on_server(|| "<noscript>Enable JavaScript</noscript>");
//! let browser_only = render_on_browser(|| "<div id='hydrated'>Hydrated!</div>");
//! ```

use std::cell::RefCell;

/// The platform where code is executing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Platform {
    /// Server-side rendering environment.
    #[default]
    Server,
    /// Browser/client-side environment.
    Browser,
}

impl Platform {
    /// Get the current platform.
    pub fn current() -> Self {
        CURRENT_PLATFORM.with(|p| *p.borrow())
    }

    /// Check if we're on the server.
    pub fn is_server() -> bool {
        Self::current() == Platform::Server
    }

    /// Check if we're in the browser.
    pub fn is_browser() -> bool {
        Self::current() == Platform::Browser
    }

    /// Get a human-readable name for the platform.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Server => "server",
            Platform::Browser => "browser",
        }
    }
}


impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// Thread-local storage for current platform
thread_local! {
    static CURRENT_PLATFORM: RefCell<Platform> = RefCell::new(Platform::default());
}

/// Check if currently running on the server (during SSR).
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::is_server;
///
/// if is_server() {
///     // Server-only logic: database access, file system, etc.
///     println!("Rendering on server");
/// }
/// ```
#[inline]
pub fn is_server() -> bool {
    Platform::is_server()
}

/// Check if currently running in the browser.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::is_browser;
///
/// if is_browser() {
///     // Browser-only logic: DOM manipulation, browser APIs, etc.
///     let window = web_sys::window().unwrap();
/// }
/// ```
#[inline]
pub fn is_browser() -> bool {
    Platform::is_browser()
}

/// Set the current platform context.
///
/// This is typically called at application startup to establish
/// the execution environment.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::{set_platform, Platform};
///
/// // In your SSR server entry point:
/// set_platform(Platform::Server);
///
/// // In your browser entry point:
/// set_platform(Platform::Browser);
/// ```
pub fn set_platform(platform: Platform) {
    CURRENT_PLATFORM.with(|p| {
        *p.borrow_mut() = platform;
    });
}

/// Execute a function within a specific platform context.
///
/// This temporarily sets the platform for the duration of the closure,
/// then restores the previous platform.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::{with_platform, Platform, is_server};
///
/// with_platform(Platform::Server, || {
///     assert!(is_server());
///     // Render components as if on server
/// });
/// ```
pub fn with_platform<F, R>(platform: Platform, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = Platform::current();
    set_platform(platform);
    let result = f();
    set_platform(previous);
    result
}

/// Conditionally render content only on the server.
///
/// Returns the result of the closure on the server, or `None` in the browser.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::render_on_server;
///
/// let server_content = render_on_server(|| {
///     "<script>window.__SSR__ = true;</script>".to_string()
/// });
/// ```
pub fn render_on_server<F, T>(f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    if is_server() {
        Some(f())
    } else {
        None
    }
}

/// Conditionally render content only in the browser.
///
/// Returns the result of the closure in the browser, or `None` on the server.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::render_on_browser;
///
/// let browser_content = render_on_browser(|| {
///     // This only runs in browser, safe to use browser APIs
///     "<div>Interactive content</div>".to_string()
/// });
/// ```
pub fn render_on_browser<F, T>(f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    if is_browser() {
        Some(f())
    } else {
        None
    }
}

/// Get content for server, with a fallback for browser.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::server_or;
///
/// let content = server_or(
///     || "Server-rendered content",
///     || "Browser fallback"
/// );
/// ```
pub fn server_or<F1, F2, T>(server_fn: F1, browser_fn: F2) -> T
where
    F1: FnOnce() -> T,
    F2: FnOnce() -> T,
{
    if is_server() {
        server_fn()
    } else {
        browser_fn()
    }
}

/// Get content for browser, with a fallback for server.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::browser_or;
///
/// let content = browser_or(
///     || "Browser-specific content",
///     || "Server fallback"
/// );
/// ```
pub fn browser_or<F1, F2, T>(browser_fn: F1, server_fn: F2) -> T
where
    F1: FnOnce() -> T,
    F2: FnOnce() -> T,
{
    if is_browser() {
        browser_fn()
    } else {
        server_fn()
    }
}

/// A platform-aware value that can have different values on server vs browser.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::PlatformValue;
///
/// let api_url = PlatformValue::new(
///     "http://localhost:3000/api",  // Server (internal)
///     "/api"                         // Browser (relative)
/// );
///
/// let url = api_url.get();
/// ```
#[derive(Debug, Clone)]
pub struct PlatformValue<T> {
    server_value: T,
    browser_value: T,
}

impl<T: Clone> PlatformValue<T> {
    /// Create a new platform-aware value.
    pub fn new(server_value: T, browser_value: T) -> Self {
        Self {
            server_value,
            browser_value,
        }
    }

    /// Get the value for the current platform.
    pub fn get(&self) -> T {
        if is_server() {
            self.server_value.clone()
        } else {
            self.browser_value.clone()
        }
    }

    /// Get the server value regardless of current platform.
    pub fn server(&self) -> &T {
        &self.server_value
    }

    /// Get the browser value regardless of current platform.
    pub fn browser(&self) -> &T {
        &self.browser_value
    }
}

impl<T: Clone + Default> Default for PlatformValue<T> {
    fn default() -> Self {
        Self::new(T::default(), T::default())
    }
}

/// Marker trait for platform-specific behavior.
///
/// Implement this trait to create components or services that
/// behave differently based on the platform.
pub trait PlatformAware {
    /// Check if this implementation supports the current platform.
    fn supports_platform(&self) -> bool {
        true
    }

    /// Get the preferred platform for this implementation.
    fn preferred_platform(&self) -> Option<Platform> {
        None
    }
}

/// A guard that ensures code runs only on the server.
///
/// # Panics
///
/// Panics if called from the browser context.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::ServerOnly;
///
/// fn server_only_operation() {
///     let _guard = ServerOnly::new();
///     // This code will panic if called from browser
///     std::fs::read_to_string("config.json").unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct ServerOnly {
    _private: (),
}

impl ServerOnly {
    /// Create a new server-only guard.
    ///
    /// # Panics
    ///
    /// Panics if called from browser context.
    pub fn new() -> Self {
        assert!(
            is_server(),
            "ServerOnly: This code must only run on the server"
        );
        Self { _private: () }
    }

    /// Try to create a server-only guard.
    ///
    /// Returns `None` if called from browser context.
    pub fn try_new() -> Option<Self> {
        if is_server() {
            Some(Self { _private: () })
        } else {
            None
        }
    }
}

/// A guard that ensures code runs only in the browser.
///
/// # Panics
///
/// Panics if called from the server context.
///
/// # Example
///
/// ```ignore
/// use ferric_ssr::platform::BrowserOnly;
///
/// fn browser_only_operation() {
///     let _guard = BrowserOnly::new();
///     // This code will panic if called from server
///     web_sys::window().unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct BrowserOnly {
    _private: (),
}

impl BrowserOnly {
    /// Create a new browser-only guard.
    ///
    /// # Panics
    ///
    /// Panics if called from server context.
    pub fn new() -> Self {
        assert!(
            is_browser(),
            "BrowserOnly: This code must only run in the browser"
        );
        Self { _private: () }
    }

    /// Try to create a browser-only guard.
    ///
    /// Returns `None` if called from server context.
    pub fn try_new() -> Option<Self> {
        if is_browser() {
            Some(Self { _private: () })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_platform() {
        // On non-wasm, default should be Server
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(Platform::default(), Platform::Server);
    }

    #[test]
    fn test_set_platform() {
        let original = Platform::current();

        set_platform(Platform::Server);
        assert!(is_server());
        assert!(!is_browser());

        set_platform(Platform::Browser);
        assert!(is_browser());
        assert!(!is_server());

        // Restore
        set_platform(original);
    }

    #[test]
    fn test_with_platform() {
        let original = Platform::current();

        with_platform(Platform::Server, || {
            assert!(is_server());
        });

        with_platform(Platform::Browser, || {
            assert!(is_browser());
        });

        // Should be restored
        assert_eq!(Platform::current(), original);
    }

    #[test]
    fn test_render_on_server() {
        set_platform(Platform::Server);
        let result = render_on_server(|| 42);
        assert_eq!(result, Some(42));

        set_platform(Platform::Browser);
        let result = render_on_server(|| 42);
        assert_eq!(result, None);
    }

    #[test]
    fn test_render_on_browser() {
        set_platform(Platform::Browser);
        let result = render_on_browser(|| 42);
        assert_eq!(result, Some(42));

        set_platform(Platform::Server);
        let result = render_on_browser(|| 42);
        assert_eq!(result, None);
    }

    #[test]
    fn test_platform_value() {
        let value = PlatformValue::new("server", "browser");

        set_platform(Platform::Server);
        assert_eq!(value.get(), "server");

        set_platform(Platform::Browser);
        assert_eq!(value.get(), "browser");

        // Direct access
        assert_eq!(value.server(), &"server");
        assert_eq!(value.browser(), &"browser");
    }

    #[test]
    fn test_server_or() {
        set_platform(Platform::Server);
        assert_eq!(server_or(|| "server", || "browser"), "server");

        set_platform(Platform::Browser);
        assert_eq!(server_or(|| "server", || "browser"), "browser");
    }

    #[test]
    fn test_browser_or() {
        set_platform(Platform::Browser);
        assert_eq!(browser_or(|| "browser", || "server"), "browser");

        set_platform(Platform::Server);
        assert_eq!(browser_or(|| "browser", || "server"), "server");
    }

    #[test]
    fn test_server_only_guard() {
        set_platform(Platform::Server);
        let guard = ServerOnly::try_new();
        assert!(guard.is_some());

        set_platform(Platform::Browser);
        let guard = ServerOnly::try_new();
        assert!(guard.is_none());
    }

    #[test]
    fn test_browser_only_guard() {
        set_platform(Platform::Browser);
        let guard = BrowserOnly::try_new();
        assert!(guard.is_some());

        set_platform(Platform::Server);
        let guard = BrowserOnly::try_new();
        assert!(guard.is_none());
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Server.to_string(), "server");
        assert_eq!(Platform::Browser.to_string(), "browser");
    }
}

