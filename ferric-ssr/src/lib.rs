//! # Ferric SSR
//!
//! Server-side rendering module for the Ferric framework, built on Hyper.
//!
//! This crate provides the infrastructure to render Ferric components on the server
//! and serve them via HTTP using Hyper as the underlying HTTP implementation.
//!
//! ## Features
//!
//! - **Component rendering**: Render Ferric components to HTML strings
//! - **Hyper integration**: Built-in HTTP server using Hyper
//! - **Streaming support**: Stream rendered content for faster time-to-first-byte
//! - **State serialization**: Serialize component state for client hydration
//! - **Middleware support**: Extensible request/response processing pipeline
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_ssr::{SsrServer, SsrConfig, render_to_string};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = SsrConfig::new()
//!         .bind("127.0.0.1:3000")
//!         .with_static_dir("./static");
//!
//!     let server = SsrServer::new(config)
//!         .route("/", |req| async move {
//!             let html = render_to_string(AppComponent::new());
//!             Ok(html)
//!         });
//!
//!     server.run().await.unwrap();
//! }
//! ```

mod config;
mod error;
pub mod hydration;
pub mod platform;
mod render;
mod response;
mod server;
mod state;
mod stream;

pub use config::SsrConfig;
pub use error::{SsrError, SsrResult};
pub use hydration::{
    FullHydration, HydrationConfig, HydrationMarkers, HydrationStrategy,
    Island, IslandLoading, IslandPriority, PartialHydration,
};
pub use platform::{
    is_browser, is_server, set_platform, with_platform, Platform, PlatformValue,
    render_on_browser, render_on_server, server_or, browser_or,
    BrowserOnly, ServerOnly, PlatformAware,
};
pub use render::{
    render_to_string, render_to_string_with_state, render_with_hydration, wrap_in_shell,
    HtmlRenderer, RenderContext, Renderable, ShellConfig,
};
pub use response::{HtmlResponse, SsrResponse};
pub use server::{SsrServer, SsrService};
pub use state::{SerializedState, StateSerializer};
pub use stream::{render_to_stream, HtmlStream};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::config::SsrConfig;
    pub use crate::error::{SsrError, SsrResult};
    pub use crate::hydration::{
        FullHydration, HydrationConfig, HydrationStrategy,
        Island, IslandLoading, IslandPriority, PartialHydration,
    };
    pub use crate::platform::{
        is_browser, is_server, set_platform, with_platform, Platform, PlatformValue,
        render_on_browser, render_on_server, server_or, browser_or,
        BrowserOnly, ServerOnly,
    };
    pub use crate::render::{render_to_string, render_with_hydration, HtmlRenderer, RenderContext, Renderable};
    pub use crate::response::{HtmlResponse, SsrResponse};
    pub use crate::server::SsrServer;
    pub use crate::state::StateSerializer;
    pub use crate::stream::render_to_stream;
}
