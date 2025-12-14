# SSR Integrations Guide

Complete guide to using Ferric's server-side rendering with various Rust web frameworks.

## Overview

Ferric provides SSR integrations for all major Rust web frameworks that use Hyper underneath:

| Framework | Crate | Status | Best For |
|-----------|-------|--------|----------|
| **Axum** | `axum-ferric-ssr` | ✅ Stable | Modern, type-safe APIs |
| **Warp** | `warp-ferric-ssr` | ✅ Stable | Filter-based routing |
| **Rocket** | `rocket-ferric-ssr` | ✅ Stable | Convention over configuration |
| **Poem** | `poem-ferric-ssr` | ✅ Stable | OpenAPI, modern design |
| **Armature** | `armature-ferric-ssr` | ✅ Stable | Angular-inspired, full-stack |

## Quick Comparison

### Performance

All integrations use Hyper underneath and have similar performance characteristics:

- **Throughput**: 50K-100K+ requests/second
- **Latency**: <1ms for cached pages, 1-5ms for rendered pages
- **Memory**: ~10-50MB base, scales with concurrent requests

### Framework Philosophy

#### Axum
- **Style**: Builder pattern, type-safe extractors
- **Best for**: Teams wanting compile-time safety
- **Learning curve**: Medium

#### Warp
- **Style**: Filter composition
- **Best for**: Functional programming enthusiasts
- **Learning curve**: Steep

#### Rocket
- **Style**: Annotations, conventions
- **Best for**: Rails/Django developers
- **Learning curve**: Easy

#### Poem
- **Style**: Modern async, OpenAPI-first
- **Best for**: API-heavy applications
- **Learning curve**: Easy-Medium

#### Armature
- **Style**: Angular-inspired decorators
- **Best for**: Full-stack TypeScript developers
- **Learning curve**: Easy (if familiar with Angular)

## Installation

### Axum

```toml
[dependencies]
axum-ferric-ssr = { path = "../axum-ferric-ssr" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

### Warp

```toml
[dependencies]
warp-ferric-ssr = { path = "../warp-ferric-ssr" }
warp = "0.3"
tokio = { version = "1", features = ["full"] }
```

### Rocket

```toml
[dependencies]
rocket-ferric-ssr = { path = "../rocket-ferric-ssr" }
rocket = "0.5"
```

### Poem

```toml
[dependencies]
poem-ferric-ssr = { path = "../poem-ferric-ssr" }
poem = "3.0"
tokio = { version = "1", features = ["full"] }
```

## Usage Examples

### Axum

```rust
use axum::{Router, routing::get, extract::State, response::Html};
use axum_ssr::FerricAxumRenderer;
use ferric_ssr::SsrConfig;

async fn home(State(renderer): State<FerricAxumRenderer>) -> Html<String> {
    let html = renderer
        .render_with_hydration("app-root", None)
        .await
        .unwrap();
    Html(html)
}

#[tokio::main]
async fn main() {
    let renderer = FerricAxumRenderer::new(SsrConfig::default());

    let app = Router::new()
        .route("/", get(home))
        .with_state(renderer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### Warp

```rust
use warp::Filter;
use warp_ssr::{FerricWarpRenderer, with_renderer};
use ferric_ssr::SsrConfig;

#[tokio::main]
async fn main() {
    let renderer = FerricWarpRenderer::new(SsrConfig::default());

    let routes = warp::path::end()
        .and(with_renderer(renderer))
        .and_then(|renderer: FerricWarpRenderer| async move {
            let html = renderer.render_with_hydration("app-root", None).await
                .map_err(|_| warp::reject::not_found())?;
            Ok::<_, warp::Rejection>(warp::reply::html(html))
        });

    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}
```

### Rocket

```rust
use rocket::{get, routes, State};
use rocket::response::content::RawHtml;
use rocket_ssr::FerricRocketRenderer;
use ferric_ssr::SsrConfig;

#[get("/")]
async fn index(renderer: &State<FerricRocketRenderer>) -> RawHtml<String> {
    let html = renderer.render_with_hydration("app-root", None).await.unwrap();
    RawHtml(html)
}

#[rocket::launch]
fn rocket() -> _ {
    let renderer = FerricRocketRenderer::new(SsrConfig::default());

    rocket::build()
        .manage(renderer)
        .mount("/", routes![index])
}
```

### Poem

```rust
use poem::{Route, get, handler, web::Html, Server, listener::TcpListener};
use poem_ssr::FerricPoemRenderer;
use ferric_ssr::SsrConfig;

#[handler]
async fn index(renderer: poem::web::Data<&FerricPoemRenderer>) -> Html<String> {
    let html = renderer.render_with_hydration("app-root", None).await.unwrap();
    Html(html)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let renderer = FerricPoemRenderer::new(SsrConfig::default());

    let app = Route::new()
        .at("/", get(index))
        .data(renderer);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
```

## Advanced Features

### Caching

All integrations support in-memory caching:

```rust
// Axum
use axum_ssr::middleware::SsrCache;
let cache = SsrCache::new(Duration::from_secs(300));

// Warp
use warp_ssr::middleware::SsrCache;
let cache = SsrCache::new(Duration::from_secs(300));

// Rocket
use rocket_ssr::cache::SsrCache;
let cache = SsrCache::new(Duration::from_secs(300));

// Poem
use poem_ssr::middleware::SsrCache;
let cache = SsrCache::new(Duration::from_secs(300));
```

### Compression

Enable compression in your configuration:

```rust
let config = SsrConfig {
    compression: true,
    minify_html: true,
    ..Default::default()
};
```

### Props Passing

Pass props to components:

```rust
use serde_json::json;

let props = json!({
    "userId": user_id,
    "title": "Welcome",
});

renderer.render_with_hydration("user-profile", Some(props)).await
```

## Migration Guide

### From Next.js

| Next.js | Ferric SSR |
|---------|------------|
| `getServerSideProps` | Render with props |
| `getStaticProps` | Pre-render with caching |
| API routes | Standard framework routes |
| Middleware | Framework middleware |

### From SvelteKit

| SvelteKit | Ferric SSR |
|-----------|------------|
| `load` function | Render with props |
| `+page.server.ts` | SSR handler |
| `hooks.server.ts` | Middleware |

## Performance Tips

1. **Enable Caching**: Use `SsrCache` for frequently accessed pages
2. **Compression**: Enable gzip/brotli compression
3. **Minification**: Set `minify_html: true`
4. **CDN**: Serve static assets from CDN
5. **Connection Pooling**: Use connection pooling for databases
6. **Lazy Hydration**: Only hydrate interactive components
7. **Streaming**: Use streaming SSR for large pages

## Deployment

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-app /usr/local/bin/
CMD ["my-app"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ferric-ssr-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ferric-ssr
  template:
    metadata:
      labels:
        app: ferric-ssr
    spec:
      containers:
      - name: app
        image: my-ferric-app:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: info
```

## Troubleshooting

### Compilation Errors

Ensure all dependencies are compatible:

```bash
cargo update
cargo build
```

### Runtime Errors

Check logs:

```bash
RUST_LOG=debug cargo run
```

### Performance Issues

1. Enable release mode: `cargo build --release`
2. Profile with `cargo flamegraph`
3. Check for memory leaks with `valgrind`

## Examples

See the `examples/` directory for complete working examples:

- `axum-ferric-ssr-demo` - Full Axum application
- `warp-ferric-ssr-demo` - Filter-based routing
- `rocket-ferric-ssr-demo` - Annotated routes
- `poem-ferric-ssr-demo` - OpenAPI integration

## Resources

- [Ferric SSR Documentation](./FERRIC_SSR_GUIDE.md)
- [Axum Documentation](https://docs.rs/axum/)
- [Warp Documentation](https://docs.rs/warp/)
- [Rocket Documentation](https://rocket.rs/)
- [Poem Documentation](https://docs.rs/poem/)

## License

MIT

