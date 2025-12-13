//! Project scaffolding templates.

use crate::utils::to_snake_case;

pub fn cargo_toml(name: &str) -> String {
    let snake_name = to_snake_case(name);
    format!(
        r#"[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"
description = "A Ferric web application"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
ferric-core = {{ git = "https://github.com/pegasusheavy/ferric.git" }}
ferric-macros = {{ git = "https://github.com/pegasusheavy/ferric.git" }}
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = {{ version = "0.3", features = [
    "console",
    "Document",
    "Element",
    "HtmlElement",
    "Node",
    "Window",
    "Event",
    "EventTarget",
    "HtmlInputElement",
    "HtmlStyleElement",
    "HtmlHeadElement",
] }}
js-sys = "0.3"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "s"
lto = true
"#,
        snake_name = snake_name
    )
}

pub fn cargo_toml_with_ssr(name: &str) -> String {
    let snake_name = to_snake_case(name);
    format!(
        r#"[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"
description = "A Ferric web application with SSR support"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
ferric-core = {{ git = "https://github.com/pegasusheavy/ferric.git" }}
ferric-macros = {{ git = "https://github.com/pegasusheavy/ferric.git" }}
ferric-ssr = {{ git = "https://github.com/pegasusheavy/ferric.git" }}
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = {{ version = "0.3", features = [
    "console",
    "Document",
    "Element",
    "HtmlElement",
    "Node",
    "Window",
    "Event",
    "EventTarget",
    "HtmlInputElement",
    "HtmlStyleElement",
    "HtmlHeadElement",
] }}
js-sys = "0.3"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# SSR dependencies (server-only)
tokio = {{ version = "1.35", features = ["full"], optional = true }}
hyper = {{ version = "1.5", features = ["full"], optional = true }}

[features]
default = []
ssr = ["tokio", "hyper"]

[dev-dependencies]
wasm-bindgen-test = "0.3"

[[bin]]
name = "server"
path = "src/server/main.rs"
required-features = ["ssr"]

[profile.release]
opt-level = "s"
lto = true
"#,
        snake_name = snake_name
    )
}

pub fn lib_rs(name: &str) -> String {
    format!(
        r##"//! {name} - A Ferric web application

use wasm_bindgen::prelude::*;
use ferric_core::prelude::*;

mod app;

pub use app::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {{
    // Initialize logging
    ferric_core::logging::set_log_level(ferric_core::logging::Level::Debug);

    // Initialize the application
    info!("{name} starting...");

    // Mount the app component
    app::App::mount("#app-root")?;

    info!("{name} initialized");
    Ok(())
}}
"##,
        name = name
    )
}

pub fn lib_rs_with_ssr(name: &str) -> String {
    format!(
        r##"//! {name} - A Ferric web application with SSR support

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use ferric_core::prelude::*;

mod app;

pub use app::*;

// Re-export for SSR
pub use app::App;

/// Browser entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {{
    use ferric_ssr::{{set_platform, Platform}};

    // Set platform to browser
    set_platform(Platform::Browser);

    // Initialize logging
    ferric_core::logging::set_log_level(ferric_core::logging::Level::Debug);

    // Initialize the application
    info!("{name} starting in browser...");

    // Hydrate the app (reuse SSR-rendered HTML)
    app::App::hydrate("#app-root")?;

    info!("{name} hydrated");
    Ok(())
}}
"##,
        name = name
    )
}

pub fn app_mod(name: &str) -> String {
    format!(
        r#"//! Main application module for {name}

mod app;

pub use app::App;
"#,
        name = name
    )
}

pub fn app_component() -> String {
    r#"//! Root application component

use ferric_core::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, Element};

pub struct App {
    title: Signal<String>,
    #[cfg(target_arch = "wasm32")]
    root: Element,
}

impl App {
    /// Mount the app (browser-only)
    #[cfg(target_arch = "wasm32")]
    pub fn mount(selector: &str) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");

        let root = document
            .query_selector(selector)?
            .expect("root element not found");

        let app = App {
            root,
            title: Signal::new("Welcome to Ferric!".to_string()),
        };

        app.render(&document)?;
        Ok(())
    }

    /// Hydrate the app from SSR-rendered HTML (browser-only)
    #[cfg(target_arch = "wasm32")]
    pub fn hydrate(selector: &str) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");

        let root = document
            .query_selector(selector)?
            .expect("root element not found");

        let app = App {
            root,
            title: Signal::new("Welcome to Ferric!".to_string()),
        };

        // Re-attach event listeners without re-rendering
        app.attach_handlers(&document)?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn render(&self, document: &Document) -> Result<(), JsValue> {
        // Load external template
        let template_html = include_str!("../../templates/app.html");
        self.root.set_inner_html(template_html);

        // Inject styles
        inject_styles(document, include_str!("../styles/app.scss"));

        self.attach_handlers(document)?;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn attach_handlers(&self, _document: &Document) -> Result<(), JsValue> {
        // Set up reactive bindings
        // This is where you'd attach event listeners and reactive updates
        Ok(())
    }

    /// Render to HTML string (for SSR)
    pub fn render_to_string() -> String {
        include_str!("../../templates/app.html").to_string()
    }
}

#[cfg(target_arch = "wasm32")]
fn inject_styles(document: &Document, css: &str) {
    let style = document.create_element("style").unwrap();
    style.set_text_content(Some(css));
    if let Some(head) = document.head() {
        let _ = head.append_child(&style);
    }
}
"#
    .to_string()
}

pub fn app_html(name: &str) -> String {
    format!(
        r#"<div class="app-container">
  <header class="app-header">
    <h1 class="app-title">Welcome to {name}!</h1>
    <p class="app-subtitle">Built with Ferric 🦀</p>
  </header>

  <main class="app-content">
    <section class="hero">
      <h2>Get Started</h2>
      <p>Edit <code>src/app/app.rs</code> to start building your application.</p>
    </section>

    <section class="features">
      <div class="feature">
        <h3>🚀 Fast</h3>
        <p>WebAssembly performance with Rust safety</p>
      </div>
      <div class="feature">
        <h3>⚡ Reactive</h3>
        <p>Signals, computed values, and effects</p>
      </div>
      <div class="feature">
        <h3>🔧 Modular</h3>
        <p>Component-based architecture with DI</p>
      </div>
    </section>
  </main>

  <footer class="app-footer">
    <p>Powered by <a href="https://github.com/pegasusheavy/ferric">Ferric</a></p>
  </footer>
</div>
"#,
        name = name
    )
}

pub fn app_scss() -> String {
    r#".app-container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  font-family: system-ui, -apple-system, sans-serif;
  color: #1a1a2e;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.app-header {
  text-align: center;
  padding: 4rem 2rem 2rem;
  color: white;
}

.app-title {
  font-size: 3rem;
  font-weight: 700;
  margin: 0;
  text-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

.app-subtitle {
  font-size: 1.25rem;
  opacity: 0.9;
  margin-top: 0.5rem;
}

.app-content {
  flex: 1;
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
}

.hero {
  background: white;
  border-radius: 1rem;
  padding: 2rem;
  text-align: center;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
  margin-bottom: 2rem;

  h2 {
    font-size: 2rem;
    margin: 0 0 1rem;
    color: #1a1a2e;
  }

  code {
    background: #f0f0f5;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-family: 'Fira Code', monospace;
  }
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
}

.feature {
  background: white;
  border-radius: 1rem;
  padding: 1.5rem;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
  transition: transform 0.2s ease;

  &:hover {
    transform: translateY(-4px);
  }

  h3 {
    font-size: 1.25rem;
    margin: 0 0 0.5rem;
  }

  p {
    color: #666;
    margin: 0;
  }
}

.app-footer {
  text-align: center;
  padding: 2rem;
  color: white;

  a {
    color: white;
    font-weight: 600;
  }
}
"#
    .to_string()
}

/// Root stylesheet that imports all component styles (Angular-style)
pub fn root_styles_scss() -> String {
    r#"// =============================================================================
// Root Stylesheet
// =============================================================================
// This is the main entry point for all styles, similar to Angular's styles.scss
// All component and global styles are imported here.
//
// IMPORTANT: All @use statements must come first in the file!

// -----------------------------------------------------------------------------
// 1. Core Variables & Mixins
// -----------------------------------------------------------------------------
@use 'styles/variables' as *;

// -----------------------------------------------------------------------------
// 2. App Component
// -----------------------------------------------------------------------------
@use 'app/app.component.scss';

// =============================================================================
// Base Styles / CSS Reset
// =============================================================================

*,
*::before,
*::after {
    box-sizing: border-box;
}

html {
    font-size: 16px;
    line-height: 1.5;
    scroll-behavior: smooth;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

body {
    margin: 0;
    padding: 0;
    font-family: $font-sans;
    background: $color-bg;
    color: $color-text;
    min-height: 100vh;
}

a {
    color: $color-primary;
    text-decoration: none;

    &:hover {
        text-decoration: underline;
    }
}

img, svg {
    display: block;
    max-width: 100%;
}

button {
    font-family: inherit;
}

// Focus styles
a:focus-visible,
button:focus-visible {
    outline: 2px solid $color-primary;
    outline-offset: 2px;
}
"#
    .to_string()
}

/// Variables partial for SCSS
pub fn variables_scss() -> String {
    r#"// =============================================================================
// SCSS Variables
// =============================================================================

// Colors
$color-primary: #667eea;
$color-primary-dark: #5a67d8;
$color-secondary: #764ba2;
$color-bg: #f5f5f5;
$color-text: #333333;
$color-text-light: #666666;
$color-white: #ffffff;
$color-black: #1a1a2e;

// Typography
$font-sans: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
$font-mono: 'Fira Code', monospace;

// Spacing
$spacing-xs: 0.25rem;
$spacing-sm: 0.5rem;
$spacing-md: 1rem;
$spacing-lg: 1.5rem;
$spacing-xl: 2rem;
$spacing-2xl: 3rem;

// Border radius
$radius-sm: 0.25rem;
$radius-md: 0.5rem;
$radius-lg: 1rem;
$radius-full: 9999px;

// Shadows
$shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
$shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
$shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1);

// Transitions
$transition-fast: 0.15s ease;
$transition-normal: 0.2s ease;

// Breakpoints
$breakpoint-sm: 640px;
$breakpoint-md: 768px;
$breakpoint-lg: 1024px;
$breakpoint-xl: 1280px;
"#
    .to_string()
}

/// App component SCSS (as a partial)
pub fn app_component_scss() -> String {
    r#"// App component styles
@use '../styles/variables' as *;

.app-container {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}

.app-header {
    text-align: center;
    padding: $spacing-2xl $spacing-xl $spacing-xl;
    background: linear-gradient(135deg, $color-primary 0%, $color-secondary 100%);
    color: $color-white;
}

.app-title {
    font-size: 3rem;
    font-weight: 700;
    margin: 0;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.app-subtitle {
    font-size: 1.25rem;
    opacity: 0.9;
    margin-top: $spacing-sm;
}

.app-content {
    flex: 1;
    padding: $spacing-xl;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
}

.hero {
    background: $color-white;
    border-radius: $radius-lg;
    padding: $spacing-xl;
    text-align: center;
    box-shadow: $shadow-md;
    margin-bottom: $spacing-xl;

    h2 {
        font-size: 2rem;
        margin: 0 0 $spacing-md;
        color: $color-black;
    }

    code {
        background: rgba($color-primary, 0.1);
        padding: $spacing-xs $spacing-sm;
        border-radius: $radius-sm;
        font-family: $font-mono;
    }
}

.features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: $spacing-lg;
}

.feature {
    background: $color-white;
    border-radius: $radius-lg;
    padding: $spacing-lg;
    box-shadow: $shadow-md;
    transition: transform $transition-normal;

    &:hover {
        transform: translateY(-4px);
    }

    h3 {
        font-size: 1.25rem;
        margin: 0 0 $spacing-sm;
    }

    p {
        color: $color-text-light;
        margin: 0;
    }
}

.app-footer {
    text-align: center;
    padding: $spacing-xl;
    background: linear-gradient(135deg, $color-primary 0%, $color-secondary 100%);
    color: $color-white;

    a {
        color: $color-white;
        font-weight: 600;
    }
}
"#
    .to_string()
}

pub fn index_html(name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="A Ferric web application">
    <title>{name}</title>

    <!-- Compiled stylesheet (built from src/styles.scss by ferric build) -->
    <link rel="stylesheet" href="/styles.css">

    <!-- Inline critical styles for loading state -->
    <style>
        .loading {{
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            background: #f5f5f5;
        }}
        .loading__spinner {{
            width: 3rem;
            height: 3rem;
            border: 2px solid transparent;
            border-top-color: #667eea;
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
        }}
        @keyframes spin {{
            to {{ transform: rotate(360deg); }}
        }}
    </style>
</head>
<body>
    <div id="app-root">
        <!-- Loading state (replaced when WASM loads) -->
        <div class="loading">
            <div class="loading__spinner"></div>
        </div>
    </div>

    <script type="module">
        import init from './pkg/{snake_name}.js';

        async function run() {{
            try {{
                await init();
            }} catch (e) {{
                console.error('Failed to initialize:', e);
                document.getElementById('app-root').innerHTML = `
                    <div class="loading" style="flex-direction: column; text-align: center;">
                        <div style="font-size: 2rem; margin-bottom: 1rem;">⚠️</div>
                        <p>Failed to load application</p>
                        <button onclick="location.reload()" style="margin-top: 1rem; padding: 0.5rem 1rem; cursor: pointer;">
                            Reload
                        </button>
                    </div>
                `;
            }}
        }}

        run();
    </script>
</body>
</html>
"#,
        name = name,
        snake_name = crate::utils::to_snake_case(name)
    )
}

pub fn gitignore() -> String {
    r#"# Build outputs
/target/
/dist/
/pkg/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Dependencies
/node_modules/

# Logs
*.log

# Local env
.env
.env.local
"#
    .to_string()
}

pub fn readme(name: &str) -> String {
    format!(
        r#"# {name}

A web application built with [Ferric](https://github.com/pegasusheavy/ferric) - the Rust + WebAssembly framework.

## Getting Started

### Prerequisites

- Rust (latest stable)
- wasm-pack (`cargo install wasm-pack`)
- Ferric CLI (`cargo install --path ../ferric-cli`)

### Development

```bash
# Start development server with live reload
ferric serve

# Build for production
ferric build --release

# Generate a new component
ferric generate component my-component
```

### Project Structure

```
{name}/
├── src/
│   ├── app/           # Main application component
│   ├── components/    # Reusable components
│   ├── services/      # Injectable services
│   └── styles/        # Global styles
├── templates/         # External HTML templates
├── assets/           # Static assets
├── index.html        # Entry point
├── ferric.toml       # Project configuration
└── Cargo.toml        # Rust dependencies
```

## Configuration

The `ferric.toml` file controls the build process:

- **build**: Output directory, optimization level, source maps
- **styles**: SCSS compilation settings, include paths
- **ssr**: Server-side rendering configuration
- **serve**: Development server settings

## Learn More

- [Ferric Documentation](https://github.com/pegasusheavy/ferric)
- [Rust WebAssembly Book](https://rustwasm.github.io/book/)
"#,
        name = name
    )
}

// SSR-specific templates

pub fn server_mod() -> String {
    r#"//! SSR server module

mod routes;

pub use routes::*;
"#
    .to_string()
}

pub fn server_main(name: &str) -> String {
    format!(
        r#"//! {name} SSR Server

use ferric_ssr::{{
    SsrConfig, SsrServer, set_platform, Platform,
    is_server, render_on_server,
}};
use std::collections::HashMap;

mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // Set platform to server
    set_platform(Platform::Server);

    println!("Starting {name} SSR server...");

    let config = SsrConfig {{
        port: 3001,
        hostname: "127.0.0.1".to_string(),
        assets_path: "./dist".to_string(),
        template_path: "./index.html".to_string(),
        initial_state_key: "__FERRIC_SSR_STATE__".to_string(),
        routes: HashMap::new(),
    }};

    let server = SsrServer::new(config, routes::handle_request);
    server.start().await?;

    Ok(())
}}
"#,
        name = name
    )
}

pub fn server_routes(name: &str) -> String {
    format!(
        r#"//! Route handlers for SSR

use ferric_ssr::{{
    SsrResult, RenderOptions, is_server,
    stream::HtmlStream, stream::StreamBuilder,
}};
use hyper::{{Request, body::Incoming}};

use {snake_name}::App;

/// Handle incoming requests
pub fn handle_request(
    req: &Request<Incoming>,
    options: &RenderOptions,
) -> SsrResult<HtmlStream> {{
    let path = req.uri().path();

    match path {{
        "/" => render_home(options),
        _ => render_not_found(),
    }}
}}

fn render_home(options: &RenderOptions) -> SsrResult<HtmlStream> {{
    assert!(is_server(), "This should only run on server");

    let html = App::render_to_string();

    let stream = StreamBuilder::new()
        .start_document("en", "{name}")
        .chunk(&html)
        .end_document()
        .build();

    Ok(stream)
}}

fn render_not_found() -> SsrResult<HtmlStream> {{
    let stream = StreamBuilder::new()
        .start_document("en", "Not Found")
        .chunk("<h1>404 - Page Not Found</h1>")
        .end_document()
        .build();

    Ok(stream)
}}
"#,
        name = name,
        snake_name = to_snake_case(name)
    )
}
