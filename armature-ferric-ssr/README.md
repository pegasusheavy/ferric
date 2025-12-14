# Armature Ferric SSR

Ferric SSR integration module for the Armature framework.

## Overview

`armature-ferric-ssr` seamlessly integrates Ferric's powerful server-side rendering capabilities with the Armature web framework, providing an Angular-like development experience with Rust's performance and safety.

## Features

- 🎨 **Angular-Inspired**: Familiar decorator-based API for Angular/NestJS developers
- 🚀 **Fast SSR**: Leverage Ferric's optimized server-side rendering
- 💧 **Hydration Strategies**: Support for full and partial hydration
- 🔌 **DI Integration**: Seamless integration with Armature's dependency injection
- 🛣️ **Router Integration**: Works with Armature's routing system
- ⚡ **Streaming**: Stream rendered HTML for better performance
- 🎯 **Type-safe**: Full Rust type safety throughout

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
armature-ferric-ssr = { path = "../armature-ferric-ssr" }
armature-core = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Basic Setup

```rust
use armature_ferric_ssr::prelude::*;
use armature_core::prelude::*;

#[ssr_controller]
#[controller("/")]
struct AppController {
    ssr_service: Arc<SsrService>,
}

impl AppController {
    #[get("/")]
    async fn index(&self) -> SsrResponse {
        self.ssr_service
            .render_component("app-root")
            .with_hydration(HydrationStrategy::Full)
            .await
    }
}

#[module]
struct AppModule;

impl Module for AppModule {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![Box::new(SsrModule::new())]
    }

    fn controllers(&self) -> Vec<Box<dyn Controller>> {
        vec![Box::new(AppController::default())]
    }
}

#[tokio::main]
async fn main() {
    let app = Armature::builder()
        .module(AppModule)
        .build()
        .await;

    app.listen("0.0.0.0:3000").await;
}
```

### With Props

```rust
#[get("/users/:id")]
async fn user_profile(&self, id: Path<String>) -> SsrResponse {
    let props = json!({
        "userId": id.0,
        "fetchOnLoad": true,
    });

    self.ssr_service
        .render_component("user-profile")
        .with_props(props)
        .with_hydration(HydrationStrategy::Full)
        .await
}
```

### Partial Hydration (Islands)

```rust
#[get("/")]
async fn index(&self) -> SsrResponse {
    self.ssr_service
        .render_component("app-root")
        .with_hydration(HydrationStrategy::Partial(PartialHydration {
            islands: vec![
                Island {
                    selector: "interactive-widget".to_string(),
                    priority: IslandPriority::High,
                    loading: IslandLoading::Immediate,
                },
            ],
        }))
        .await
}
```

### With Configuration

```rust
#[module]
struct AppModule;

impl Module for AppModule {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        let config = SsrConfigBuilder::new()
            .enable_hydration(true)
            .minify_html(true)
            .cache_ttl(Duration::from_secs(300))
            .build();

        vec![Box::new(SsrModule::with_config(config))]
    }

    // ... controllers
}
```

## Advanced Features

### SSR Middleware

```rust
use armature_ferric_ssr::middleware::SsrMiddleware;

#[module]
struct AppModule;

impl Module for AppModule {
    fn middleware(&self) -> Vec<Box<dyn Middleware>> {
        vec![
            Box::new(SsrMiddleware::new()),
        ]
    }
}
```

### Custom Shell Template

```rust
let config = SsrConfigBuilder::new()
    .shell_template(r#"
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8">
                <title>{{title}}</title>
                {{styles}}
            </head>
            <body>
                {{content}}
                {{scripts}}
            </body>
        </html>
    "#)
    .build();
```

### Caching Strategy

```rust
use armature_ferric_ssr::cache::{SsrCache, CacheStrategy};

#[injectable]
struct CachedSsrService {
    ssr_service: Arc<SsrService>,
    cache: Arc<SsrCache>,
}

impl CachedSsrService {
    async fn render_cached(&self, component: &str) -> SsrResponse {
        let cache_key = format!("ssr:{}", component);

        if let Some(cached) = self.cache.get(&cache_key).await {
            return cached;
        }

        let response = self.ssr_service
            .render_component(component)
            .with_hydration(HydrationStrategy::Full)
            .await;

        self.cache.set(cache_key, response.clone()).await;
        response
    }
}
```

### Error Handling

```rust
#[get("/")]
async fn index(&self) -> Result<SsrResponse, SsrError> {
    self.ssr_service
        .render_component("app-root")
        .with_hydration(HydrationStrategy::Full)
        .await
        .map_err(|e| {
            eprintln!("SSR Error: {:?}", e);
            SsrError::RenderFailed(e.to_string())
        })
}
```

## API Reference

### `SsrService`

Main service for server-side rendering.

#### Methods

- `render_component(selector: &str) -> SsrBuilder` - Start building an SSR response
- `render_to_string(selector: &str) -> Result<String>` - Render directly to string

### `SsrBuilder`

Builder for configuring SSR rendering.

#### Methods

- `with_props(props: Value) -> Self` - Add props to component
- `with_hydration(strategy: HydrationStrategy) -> Self` - Set hydration strategy
- `with_cache_key(key: String) -> Self` - Set cache key
- `await -> Result<SsrResponse>` - Execute rendering

### `SsrResponse`

HTTP response containing rendered HTML.

#### Methods

- `html() -> &str` - Get rendered HTML
- `status(code: u16) -> Self` - Set status code
- `header(key: &str, value: &str) -> Self` - Add header

### `SsrModule`

Armature module providing SSR functionality.

#### Methods

- `new() -> Self` - Create with default config
- `with_config(config: SsrConfig) -> Self` - Create with custom config

### `HydrationStrategy`

Enum defining hydration behavior.

#### Variants

- `None` - No hydration (static HTML only)
- `Full` - Hydrate entire application
- `Partial(PartialHydration)` - Island architecture

## Configuration

### `SsrConfig`

```rust
pub struct SsrConfig {
    pub enable_hydration: bool,
    pub minify_html: bool,
    pub compression: bool,
    pub cache_ttl_seconds: u64,
    pub shell_template: Option<String>,
}
```

### `SsrConfigBuilder`

```rust
let config = SsrConfigBuilder::new()
    .enable_hydration(true)
    .minify_html(true)
    .compression(true)
    .cache_ttl(Duration::from_secs(300))
    .shell_template(custom_template)
    .build();
```

## Performance Tips

1. **Use Partial Hydration**: Only hydrate interactive components
2. **Enable Caching**: Cache rendered pages with appropriate TTL
3. **Minify HTML**: Reduce payload size
4. **Compression**: Enable gzip/brotli
5. **CDN**: Serve static assets from CDN
6. **Streaming**: Use streaming for large pages

## Comparison with Other Frameworks

| Feature | armature-ferric-ssr | Next.js | SvelteKit |
|---------|---------------------|---------|-----------|
| Language | Rust | JavaScript | JavaScript |
| Type Safety | ✅ Full | ⚠️ TypeScript | ⚠️ TypeScript |
| Performance | ⚡ Excellent | 🟢 Good | 🟢 Good |
| Memory | 💚 Low | 🟡 Medium | 🟡 Medium |
| Decorators | ✅ Native | ❌ No | ❌ No |
| DI System | ✅ Built-in | ❌ No | ❌ No |

## Migration from Angular Universal

```typescript
// Angular Universal
@Component({ selector: 'app-root' })
export class AppComponent { }

@NgModule({
  imports: [
    BrowserModule.withServerTransition({ appId: 'app' }),
    ServerModule,
  ],
})
export class AppModule { }
```

```rust
// Armature Ferric SSR
#[component(selector = "app-root")]
struct AppComponent { }

#[module]
struct AppModule;

impl Module for AppModule {
    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![Box::new(SsrModule::new())]
    }
}
```

## Examples

See the `examples/armature-ssr-demo/` directory for complete working examples:

- Basic SSR setup
- Partial hydration
- Caching strategies
- Error handling
- Integration with Armature routing

## Troubleshooting

### Component Not Found

Ensure your component is registered:

```rust
ferric_core::register_component::<AppComponent>();
```

### Hydration Mismatch

Check that server and client render the same output:
- Same props
- Same state initialization
- Same environment variables

### Performance Issues

- Enable caching
- Use partial hydration
- Minify HTML
- Enable compression

## Resources

- [Ferric SSR Documentation](../docs/FERRIC_SSR_GUIDE.md)
- [Armature Documentation](https://armature.rs)
- [SSR Integrations Guide](../docs/SSR_INTEGRATIONS_GUIDE.md)

## License

Apache-2.0
