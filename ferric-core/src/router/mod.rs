//! Client-side routing for Ferric applications.
//!
//! Provides navigation, route guards, URL parameter extraction, and more.
//!
//! ## Features
//!
//! - **Route Configuration** - Define routes with paths, components, and guards
//! - **Navigation** - Programmatic and declarative navigation
//! - **Route Parameters** - Extract path and query parameters
//! - **Route Guards** - Protect routes with activation/deactivation guards
//! - **Data Resolvers** - Pre-fetch data before route activation
//! - **Child Routes** - Nested routing support
//! - **Named Outlets** - Multiple router outlets
//! - **Navigation Events** - Subscribe to navigation lifecycle events
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_core::router::{Router, Route, Routes};
//!
//! // Configure routes
//! let routes: Routes = vec![
//!     Route::new("/").component("app-home"),
//!     Route::new("/users").component("app-users")
//!         .children(vec![
//!             Route::new(":id").component("app-user-detail"),
//!         ]),
//!     Route::new("/admin")
//!         .can_activate("AuthGuard")
//!         .component("app-admin"),
//!     Route::new("**").redirect_to("/"),  // Wildcard
//! ];
//!
//! // Create and initialize router
//! let router = Router::new(routes);
//! router.init()?;
//!
//! // Navigate programmatically
//! router.navigate("/users/123")?;
//!
//! // Subscribe to events
//! router.events().subscribe(|event| {
//!     match event {
//!         Event::NavigationEnd(e) => println!("Navigated to: {}", e.url),
//!         _ => {}
//!     }
//! });
//! ```

mod config;
mod events;
mod guard;
mod lazy;
mod outlet;
mod params;
mod resolver;
mod service;

pub use config::*;
pub use events::*;
pub use guard::*;
pub use lazy::*;
pub use outlet::*;
pub use params::*;
pub use resolver::*;
pub use service::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Initialize the router and set up navigation event listeners.
pub fn init_router() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global window exists");

    // Listen for popstate events (back/forward navigation)
    let closure = Closure::wrap(Box::new(move |_event: web_sys::PopStateEvent| {
        // Handle navigation
        if let Some(window) = web_sys::window() {
            if let Ok(pathname) = window.location().pathname() {
                // TODO: Trigger route change
                web_sys::console::log_1(&format!("Navigation to: {}", pathname).into());
            }
        }
    }) as Box<dyn FnMut(_)>);

    window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}

/// Create a routes array using a builder pattern.
#[macro_export]
macro_rules! routes {
    (
        $($route:expr),* $(,)?
    ) => {
        vec![$($route),*]
    };
}
