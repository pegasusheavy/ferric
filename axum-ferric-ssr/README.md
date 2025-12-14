# Axum Ferric SSR - Ferric Server-Side Rendering for Axum

Seamless integration between Ferric framework's SSR capabilities and the Axum web framework.

## Features

- 🚀 **Fast SSR**: Leverage Ferric's optimized server-side rendering
- 💧 **Hydration**: Automatic client-side hydration support
- 🗜️ **Compression**: Built-in compression middleware
- 📦 **Caching**: In-memory SSR caching with TTL
- 🎯 **Type-safe**: Full Rust type safety
- ⚡ **Async**: Built on Tokio and Hyper for maximum performance

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
axum-ferric-ssr = { path = "../axum-ferric-ssr" }
ferric-ssr = { path = "../ferric-ssr" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Basic Usage

```rust
use axum::{Router, routing::get};
use axum_ferric_ssr::{FerricAxumRenderer, render_handler};
use ferric_ssr::SsrConfig;

#[tokio::main]
async fn main() {
    // Create renderer
    let config = SsrConfig::default();
    let renderer = FerricAxumRenderer::new(config);

    // Create router
    let app = Router::new()
        .route("/", get(render_handler))
        .with_state(renderer);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### With Custom Routes

```rust
use axum::{Router, routing::get, extract::State, response::Html};
use axum_ferric_ssr::FerricAxumRenderer;
use ferric_ssr::SsrConfig;

async fn home(State(renderer): State<FerricAxumRenderer>) -> Html<String> {
    let html = renderer
        .render_with_hydration("home-page", None)
        .await
        .unwrap();
    Html(html)
}

async fn about(State(renderer): State<FerricAxumRenderer>) -> Html<String> {
    let html = renderer
        .render_with_hydration("about-page", None)
        .await
        .unwrap();
    Html(html)
}

#[tokio::main]
async fn main() {
    let renderer = FerricAxumRenderer::new(SsrConfig::default());

    let app = Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .with_state(renderer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### With Props

```rust
use axum::{Router, routing::get, extract::{State, Path}, response::Html};
use axum_ferric_ssr::FerricAxumRenderer;
use serde_json::json;

async fn user_page(
    State(renderer): State<FerricAxumRenderer>,
    Path(user_id): Path<String>,
) -> Html<String> {
    let props = json!({
        "userId": user_id,
    });

    let html = renderer
        .render_with_hydration("user-profile", Some(props))
        .await
        .unwrap();

    Html(html)
}
```

### With Caching

```rust
use axum::{Router, routing::get, middleware as axum_middleware};
use axum_ferric_ssr::{FerricAxumRenderer, middleware::SsrCache};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let renderer = FerricAxumRenderer::new(SsrConfig::default());
    let cache = SsrCache::new(Duration::from_secs(300)); // 5 minute TTL

    let app = Router::new()
        .route("/", get(cached_handler))
        .with_state((renderer, cache));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn cached_handler(
    State((renderer, cache)): State<(FerricAxumRenderer, SsrCache)>,
) -> Html<String> {
    let cache_key = "home";

    // Check cache
    if let Some(cached_html) = cache.get(cache_key).await {
        return Html(cached_html);
    }

    // Render if not cached
    let html = renderer
        .render_with_hydration("app-root", None)
        .await
        .unwrap();

    // Cache the result
    cache.set(cache_key.to_string(), html.clone()).await;

    Html(html)
}
```

### With Compression

```rust
use axum::Router;
use axum_ferric_ssr::create_ssr_router;
use ferric_ssr::SsrConfig;

#[tokio::main]
async fn main() {
    // create_ssr_router includes compression by default
    let app = create_ssr_router(SsrConfig::default());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

## Configuration

```rust
use ferric_ssr::SsrConfig;

let config = SsrConfig {
    enable_hydration: true,
    minify_html: true,
    compression: true,
    cache_ttl_seconds: 300,
    ..Default::default()
};

let renderer = FerricAxumRenderer::new(config);
```

## API Reference

### `FerricAxumRenderer`

Main renderer for Axum integration.

#### Methods

- `new(config: SsrConfig) -> Self` - Creates a new renderer
- `render_to_string(component: &str, props: Option<Value>) -> Result<String>` - Renders without hydration
- `render_with_hydration(component: &str, props: Option<Value>) -> Result<String>` - Renders with hydration support

### `render_handler`

Built-in Axum handler for basic SSR rendering.

### `create_ssr_router`

Creates a pre-configured Axum router with SSR support and compression.

### `SsrResponse`

Response wrapper with SSR-specific functionality.

#### Methods

- `new(html: String) -> Self` - Creates a new response
- `with_status(status: StatusCode) -> Self` - Sets status code
- `with_header(key: HeaderName, value: &str) -> Self` - Adds header

### `middleware::SsrCache`

In-memory cache for rendered pages.

#### Methods

- `new(ttl: Duration) -> Self` - Creates a new cache
- `get(key: &str) -> Option<String>` - Gets cached page
- `set(key: String, html: String)` - Caches a page
- `cleanup()` - Removes expired entries

## Performance Tips

1. **Enable Caching**: Use `SsrCache` for frequently accessed pages
2. **Use Compression**: Enable compression for faster transfers
3. **Minify HTML**: Set `minify_html: true` in config
4. **Lazy Hydration**: Only hydrate interactive components
5. **CDN Integration**: Serve static assets from CDN

## Examples

See the `examples/` directory for complete working examples:

- `basic-ssr` - Simple SSR setup
- `with-routing` - Multiple routes with SSR
- `with-caching` - SSR with caching
- `full-stack` - Complete application with API and SSR

## Comparison with Other Frameworks

| Feature | axum-ssr | Next.js | SvelteKit |
|---------|----------|---------|-----------|
| Language | Rust | JavaScript | JavaScript |
| Type Safety | ✅ Full | ⚠️ TypeScript | ⚠️ TypeScript |
| Performance | ⚡ Excellent | 🟢 Good | 🟢 Good |
| Bundle Size | 📦 Small | 📦 Large | 📦 Medium |
| Memory | 💚 Low | 🟡 Medium | 🟡 Medium |

## License

MIT

