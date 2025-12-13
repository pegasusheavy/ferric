//! Page components for the SSR demo.

use ferric_ssr::prelude::*;

/// Home page component.
pub struct HomePage {
    title: String,
    subtitle: String,
}

impl HomePage {
    /// Create a new home page.
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
        }
    }
}

impl Renderable for HomePage {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        let mut html = HtmlRenderer::new();

        // Hero section
        html.open_tag("div").attr("class", "hero").close_open();
        html.element("h1", &[], &self.title);
        html.element("p", &[("class", "subtitle")], &self.subtitle);
        html.close_tag("div");

        // Features section
        html.open_tag("div").attr("class", "features").close_open();

        // Feature 1
        html.open_tag("div").attr("class", "feature").close_open();
        html.element("h3", &[], "⚡ Fast Server Rendering");
        html.element(
            "p",
            &[],
            "Render your components on the server for instant page loads and better SEO.",
        );
        html.close_tag("div");

        // Feature 2
        html.open_tag("div").attr("class", "feature").close_open();
        html.element("h3", &[], "🔄 Seamless Hydration");
        html.element(
            "p",
            &[],
            "State is serialized and transferred to the client for smooth interactivity.",
        );
        html.close_tag("div");

        // Feature 3
        html.open_tag("div").attr("class", "feature").close_open();
        html.element("h3", &[], "🌐 Platform Detection");
        html.element(
            "p",
            &[],
            "Write isomorphic code that knows whether it's running on server or browser.",
        );
        html.close_tag("div");

        // Feature 4
        html.open_tag("div").attr("class", "feature").close_open();
        html.element("h3", &[], "🚀 Built on Hyper");
        html.element(
            "p",
            &[],
            "Production-ready HTTP server with async/await and excellent performance.",
        );
        html.close_tag("div");

        html.close_tag("div");

        // Code example
        html.open_tag("div").attr("class", "code-section").close_open();
        html.element("h2", &[], "Quick Example");
        html.open_tag("pre").close_open();
        html.open_tag("code").close_open();
        html.text(CODE_EXAMPLE);
        html.close_tag("code");
        html.close_tag("pre");
        html.close_tag("div");

        ctx.exit();
        Ok(html.finish())
    }
}

const CODE_EXAMPLE: &str = r#"use ferric_ssr::prelude::*;

struct MyComponent { message: String }

impl Renderable for MyComponent {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        let mut html = HtmlRenderer::new();
        html.element("h1", &[], &self.message);
        Ok(html.finish())
    }
}

#[tokio::main]
async fn main() {
    let server = SsrServer::new(SsrConfig::default())
        .get("/", |_| async {
            let component = MyComponent {
                message: "Hello, SSR!".into()
            };
            let html = render_to_string(&component)?;
            Ok(SsrResponse::html(&html))
        });

    server.run().await.unwrap();
}"#;

/// About page component.
pub struct AboutPage;

impl AboutPage {
    /// Create a new about page.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AboutPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderable for AboutPage {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        let mut html = HtmlRenderer::new();

        html.element("h1", &[], "About Ferric SSR");

        html.element("p", &[],
            "Ferric SSR is the server-side rendering module for the Ferric web framework. \
            It allows you to render your Rust components on the server and send pre-rendered \
            HTML to the client for faster page loads and better SEO."
        );

        html.element("h2", &[], "Key Features");

        html.open_tag("ul").attr("class", "feature-list").close_open();

        html.open_tag("li").close_open();
        html.raw("<strong>Component Rendering</strong> - Render any component implementing ");
        html.open_tag("code").close_open();
        html.text("Renderable");
        html.close_tag("code");
        html.text(" to HTML strings");
        html.close_tag("li");

        html.open_tag("li").close_open();
        html.raw("<strong>Hyper Integration</strong> - Built-in HTTP server using Hyper for production-grade performance");
        html.close_tag("li");

        html.open_tag("li").close_open();
        html.raw("<strong>State Serialization</strong> - Serialize component state for client-side hydration");
        html.close_tag("li");

        html.open_tag("li").close_open();
        html.raw("<strong>Platform Detection</strong> - Use ");
        html.open_tag("code").close_open();
        html.text("is_server()");
        html.close_tag("code");
        html.text(" and ");
        html.open_tag("code").close_open();
        html.text("is_browser()");
        html.close_tag("code");
        html.text(" for isomorphic code");
        html.close_tag("li");

        html.open_tag("li").close_open();
        html.raw("<strong>Streaming Support</strong> - Stream rendered content for faster time-to-first-byte");
        html.close_tag("li");

        html.close_tag("ul");

        html.element("h2", &[], "Architecture");

        html.element("p", &[],
            "The SSR module follows a simple but powerful architecture:"
        );

        html.open_tag("ol").close_open();
        html.element("li", &[], "Request comes in to the Hyper server");
        html.element("li", &[], "Route handler creates component instances");
        html.element("li", &[], "Components are rendered to HTML via the Renderable trait");
        html.element("li", &[], "State is serialized for hydration");
        html.element("li", &[], "Response is sent to the client");
        html.close_tag("ol");

        ctx.exit();
        Ok(html.finish())
    }
}

/// 404 Not Found page component.
pub struct NotFoundPage;

impl NotFoundPage {
    /// Create a new not found page.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NotFoundPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderable for NotFoundPage {
    fn render(&self, ctx: &mut RenderContext) -> SsrResult<String> {
        ctx.enter()?;

        let mut html = HtmlRenderer::new();

        html.open_tag("div")
            .attr("class", "not-found")
            .attr("style", "text-align: center; padding: 4rem 2rem;")
            .close_open();

        html.element("h1", &[("style", "font-size: 6rem; margin-bottom: 0;")], "404");
        html.element("h2", &[], "Page Not Found");
        html.element(
            "p",
            &[],
            "The page you're looking for doesn't exist or has been moved.",
        );
        html.element(
            "a",
            &[
                ("href", "/"),
                ("style", "color: #667eea; text-decoration: none; font-weight: bold;"),
            ],
            "← Go back home",
        );

        html.close_tag("div");

        ctx.exit();
        Ok(html.finish())
    }
}

