//! Page components
//!
//! Each page follows the Angular component pattern with separate files:
//! - `mod.rs` - Rust logic and component definition
//! - `*.component.html` - HTML template
//! - `*.component.scss` - Component styles

pub mod api;
pub mod docs;
pub mod examples;
pub mod guide;
pub mod home;
pub mod not_found;

pub use api::render as api;
pub use docs::{render as docs, render_page as docs_page};
pub use examples::render as examples;
pub use guide::render as guide;
pub use home::render as home;
pub use not_found::render as not_found;

//! Each page follows the Angular component pattern with separate files:
//! - `mod.rs` - Rust logic and component definition
//! - `*.component.html` - HTML template
//! - `*.component.scss` - Component styles

pub mod api;
pub mod docs;
pub mod examples;
pub mod guide;
pub mod home;
pub mod not_found;

pub use api::render as api;
pub use docs::{render as docs, render_page as docs_page};
pub use examples::render as examples;
pub use guide::render as guide;
pub use home::render as home;
pub use not_found::render as not_found;
