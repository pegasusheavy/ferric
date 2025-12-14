//! # Ferric Markdown
//!
//! Client-side markdown to HTML compilation for the Ferric framework.
//!
//! ## Features
//!
//! - **Fast Parsing**: Uses `pulldown-cmark` for efficient markdown parsing
//! - **HTML Sanitization**: Built-in XSS protection with `ammonia`
//! - **Syntax Highlighting**: Optional code syntax highlighting
//! - **Injectable Service**: Fully integrated with Ferric's DI system
//! - **Template Pipes**: Use markdown directly in templates
//! - **Customizable**: Configure rendering options via DI tokens
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_markdown::{MarkdownService, MarkdownModule};
//! use ferric::di::Injector;
//!
//! // Setup
//! let injector = Injector::root();
//! MarkdownModule::provide(&injector);
//!
//! // Use the service
//! let md_service = injector.resolve::<MarkdownService>().unwrap();
//! let html = md_service.render("# Hello, **World**!");
//! ```
//!
//! ## Template Usage
//!
//! ```html
//! <!-- Using the markdown pipe -->
//! <div [innerHTML]="content | markdown"></div>
//!
//! <!-- With syntax highlighting -->
//! <div [innerHTML]="codeExample | markdown:syntax"></div>
//! ```

mod config;
mod di;
mod parser;
mod pipe;
mod renderer;
mod sanitizer;

pub use config::{MarkdownConfig, RenderOptions, SyntaxTheme};
pub use di::{MarkdownModule, tokens as markdown_tokens};
pub use parser::MarkdownParser;
pub use pipe::MarkdownPipe;
pub use renderer::MarkdownRenderer;
pub use sanitizer::HtmlSanitizer;

use ferric_core::di::{Injectable, Injector};

/// Main markdown service for converting markdown to HTML.
///
/// This service is injectable and uses configuration from DI tokens.
///
/// ## Example
///
/// ```ignore
/// use ferric_markdown::MarkdownService;
/// use ferric::di::Injector;
///
/// let injector = Injector::root();
/// injector.register_singleton::<MarkdownService>();
///
/// let service = injector.resolve::<MarkdownService>().unwrap();
/// let html = service.render("# Hello");
/// assert!(html.contains("<h1>"));
/// ```
pub struct MarkdownService {
    config: MarkdownConfig,
    renderer: MarkdownRenderer,
    sanitizer: HtmlSanitizer,
}

impl MarkdownService {
    /// Create a new markdown service with the given configuration.
    pub fn new(config: MarkdownConfig) -> Self {
        Self {
            renderer: MarkdownRenderer::new(&config),
            sanitizer: HtmlSanitizer::new(&config),
            config,
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(MarkdownConfig::default())
    }

    /// Render markdown to HTML.
    ///
    /// The output is sanitized by default unless `unsafe-html` feature is enabled
    /// and configured.
    pub fn render(&self, markdown: &str) -> String {
        let html = self.renderer.render(markdown);

        if self.config.sanitize_html {
            self.sanitizer.sanitize(&html)
        } else {
            html
        }
    }

    /// Render markdown to HTML with custom options.
    pub fn render_with_options(&self, markdown: &str, options: &RenderOptions) -> String {
        let renderer = MarkdownRenderer::with_options(options);
        let html = renderer.render(markdown);

        if options.sanitize_html {
            let sanitizer = HtmlSanitizer::with_options(options);
            sanitizer.sanitize(&html)
        } else {
            html
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &MarkdownConfig {
        &self.config
    }
}

impl Injectable for MarkdownService {
    fn create(injector: &Injector) -> Self {
        let config = injector
            .resolve::<MarkdownConfig>()
            .map(|c| (*c).clone())
            .unwrap_or_default();

        Self::new(config)
    }
}

/// Prelude for convenient imports.
pub mod prelude {
    pub use super::{
        MarkdownService,
        MarkdownConfig,
        MarkdownModule,
        MarkdownPipe,
        RenderOptions,
        SyntaxTheme,
        markdown_tokens,
    };
}

