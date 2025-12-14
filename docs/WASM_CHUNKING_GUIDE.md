# WASM Code Splitting and Chunking Guide

This guide explains how to split your Ferric WASM application into multiple chunks for optimized loading.

## Why Chunk WASM?

**Benefits:**
- **Faster Initial Load**: Only load essential code upfront
- **On-Demand Loading**: Load features when needed
- **Better Caching**: Independent chunks cache separately
- **Reduced Bandwidth**: Users only download what they use
- **Route-Based Splitting**: Load chunks per route

**Use Cases:**
- Large applications with distinct features (admin panel, dashboard, reports)
- Route-based code splitting (different pages)
- Optional features (advanced tools, rarely-used components)
- Third-party integrations (analytics, chat widgets)

## Approaches to WASM Chunking

### 1. Multiple Crates (Recommended)

Split your application into multiple Rust crates, each compiled to a separate WASM module.

**Structure:**
```
my-app/
├── Cargo.toml          # Workspace
├── core/               # Main application
│   ├── Cargo.toml
│   └── src/lib.rs
├── admin/              # Admin panel chunk
│   ├── Cargo.toml
│   └── src/lib.rs
└── dashboard/          # Dashboard chunk
    ├── Cargo.toml
    └── src/lib.rs
```

**Workspace Cargo.toml:**
```toml
[workspace]
members = ["core", "admin", "dashboard"]
```

### 2. Feature Flags

Use Cargo features to conditionally compile code:

```toml
[features]
default = ["core"]
core = []
admin = ["dep:admin-module"]
dashboard = ["dep:dashboard-module"]
```

### 3. Dynamic Imports (wasm-bindgen)

Use `wasm-bindgen`'s dynamic import support:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn load_admin_module() -> Result<JsValue, JsValue> {
    let module = wasm_bindgen_futures::JsFuture::from(
        js_sys::eval("import('./admin.js')")
            .unwrap()
            .dyn_into::<js_sys::Promise>()
            .unwrap()
    ).await?;
    
    Ok(module)
}
```

## Configuration

Configure chunking in `ferric.toml`:

```toml
[build.chunking]
enabled = true

# Define lazy-loaded chunks
[[build.chunking.chunks]]
name = "admin"
module = "admin"  # Crate name or module path
priority = 50
preload = "route"  # Load when route activated

[[build.chunking.chunks]]
name = "dashboard"
module = "dashboard"
priority = 80
preload = "idle"  # Load when browser is idle

[[build.chunking.chunks]]
name = "analytics"
module = "analytics"
priority = 10
preload = "manual"  # Load manually
```

## Preload Strategies

### Immediate
Load right after main chunk:
```toml
preload = "immediate"
```

### Idle  
Load when browser is idle (using `requestIdleCallback`):
```toml
preload = "idle"
```

### On Route
Load when specific route is activated:
```toml
preload = "route"
```

### On Interaction
Load on first user interaction:
```toml
preload = "interaction"
```

### Manual
Load programmatically:
```toml
preload = "manual"
```

## Build Process

When chunking is enabled:

```bash
ferric build --release --target browser
```

**Output:**
```
dist/
├── index.html
├── pkg/
│   ├── main/           # Main chunk (always loaded)
│   │   ├── main.js
│   │   └── main_bg.wasm
│   ├── admin/          # Admin chunk (lazy)
│   │   ├── admin.js
│   │   └── admin_bg.wasm
│   └── dashboard/      # Dashboard chunk (lazy)
│       ├── dashboard.js
│       └── dashboard_bg.wasm
└── chunk-manifest.json # Chunk metadata
```

## Usage in Code

### Loading Chunks Dynamically

Ferric automatically injects a chunk loader:

```javascript
// In your HTML or app
window.FerricChunks.loadChunk('admin')
  .then(() => {
    console.log('Admin chunk loaded!');
    // Initialize admin features
  })
  .catch(err => {
    console.error('Failed to load admin chunk:', err);
  });
```

### Preloading Chunks

```javascript
// Preload without initializing
window.FerricChunks.preloadChunk('dashboard');
```

### Checking Chunk Status

```javascript
// Check if chunk is loaded
if (window.FerricChunks.loaded.has('admin')) {
  // Admin is ready
}
```

## Integration with Router

### Route-Based Chunking

Define chunks for specific routes:

```rust
use ferric_router::*;

let routes = vec![
    Route::new("/")
        .component::<HomeComponent>(),
    
    Route::new("/admin")
        .lazy_load("admin")  // Load admin chunk
        .component::<AdminComponent>(),
    
    Route::new("/dashboard")
        .lazy_load("dashboard")
        .component::<DashboardComponent>(),
];
```

### Guard with Chunk Loading

Ensure chunk is loaded before activating route:

```rust
use ferric_core::router::guard::*;

#[guard]
async fn ensure_admin_chunk() -> bool {
    // Load admin chunk before allowing access
    load_chunk("admin").await.is_ok()
}
```

## Optimization Tips

### 1. Chunk Size

Keep chunks between 50KB - 200KB:
- Too small: Many HTTP requests
- Too large: Defeats purpose of chunking

### 2. Shared Dependencies

Put shared code in the main chunk:
```rust
// core/src/lib.rs (main chunk)
pub mod shared_components;
pub mod shared_utils;

// admin/src/lib.rs (lazy chunk)
use core::shared_components;
```

### 3. Priority Ordering

Assign priorities based on user flow:
```toml
[[build.chunking.chunks]]
name = "dashboard"
priority = 90  # High - likely first page visited

[[build.chunking.chunks]]
name = "reports"
priority = 50  # Medium - used occasionally

[[build.chunking.chunks]]
name = "admin"
priority = 10  # Low - rarely accessed
```

### 4. Lazy Routes

Use router lazy loading:
```rust
Route::new("/admin/*")
    .lazy_children("admin")
    .guard(auth_guard)
```

## Example: Multi-Module App

### Project Structure

```
my-app/
├── Cargo.toml
├── ferric.toml
├── index.html
├── core/              # Main chunk
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── app.rs
│       └── home.rs
├── admin/             # Admin chunk
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── admin_panel.rs
└── analytics/         # Analytics chunk
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── charts.rs
```

### Workspace Cargo.toml

```toml
[workspace]
members = ["core", "admin", "analytics"]

[workspace.dependencies]
ferric-core = { path = "../ferric-core" }
wasm-bindgen = "0.2"
```

### Core Chunk (core/Cargo.toml)

```toml
[package]
name = "my-app-core"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
ferric-core = { workspace = true }
wasm-bindgen = { workspace = true }
```

### Admin Chunk (admin/Cargo.toml)

```toml
[package]
name = "my-app-admin"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
my-app-core = { path = "../core" }
ferric-core = { workspace = true }
wasm-bindgen = { workspace = true }
```

### ferric.toml Configuration

```toml
[build.chunking]
enabled = true

[[build.chunking.chunks]]
name = "admin"
module = "admin"
priority = 50
preload = "route"

[[build.chunking.chunks]]
name = "analytics"
module = "analytics"
priority = 30
preload = "idle"
```

### Loading in Routes

```rust
// core/src/lib.rs
use ferric_core::router::*;

pub fn create_router() -> Router {
    Router::new(vec![
        Route::new("/")
            .component::<HomeComponent>(),
        
        Route::new("/admin")
            .lazy_load_chunk("admin")
            .component_from_chunk("admin", "AdminPanel"),
        
        Route::new("/analytics")
            .lazy_load_chunk("analytics")
            .component_from_chunk("analytics", "AnalyticsView"),
    ])
}
```

## Manual Chunking Example

For more control, manually split your code:

```rust
// main.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    // Initialize core app
    init_core_app();
}

#[wasm_bindgen]
pub async fn load_admin() -> Result<(), JsValue> {
    // Dynamically import admin module
    let admin_module = js_sys::eval(
        "import('./pkg/admin/admin.js')"
    )?;
    
    let promise = admin_module.dyn_into::<js_sys::Promise>()?;
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    
    Ok(())
}
```

## Measuring Impact

### Check Chunk Sizes

```bash
ferric build --release
ls -lh dist/pkg/*/
```

### Analyze with wasm-pack

```bash
wasm-pack build --target web --profiling
wasm-opt --print-stack-ir dist/pkg/main/main_bg.wasm
```

### Browser DevTools

Check Network tab to see chunks loading:
- Initial load: main chunk only
- On route change: lazy chunk loads

## Best Practices

### 1. Start with Routes

Chunk by major routes/features:
```
- Main: App shell + home page
- Admin: Admin panel
- Dashboard: Analytics dashboard
- Settings: User settings
```

### 2. Measure First

Profile before optimizing:
```bash
# Check main bundle size
ls -lh dist/pkg/main_bg.wasm

# If > 500KB, consider chunking
```

### 3. Balance Granularity

- **Too few chunks**: Large initial download
- **Too many chunks**: HTTP overhead, complexity

Sweet spot: 3-5 chunks for medium apps

### 4. Common Patterns

```toml
# Pattern 1: Public + Admin
[[build.chunking.chunks]]
name = "admin"
module = "admin"
preload = "route"

# Pattern 2: Core + Features
[[build.chunking.chunks]]
name = "advanced-features"
module = "features"
preload = "idle"

# Pattern 3: Locale Bundles
[[build.chunking.chunks]]
name = "i18n-es"
module = "i18n/es"
preload = "manual"
```

## Limitations

### Current Limitations

1. **Shared Dependencies**: Duplicated across chunks (until Rust supports WASM modules better)
2. **Build Time**: Increases with number of chunks
3. **Complexity**: More moving parts to manage

### Future Improvements

- WASM Component Model support
- Better shared dependency handling
- Automatic chunk generation based on routes
- Tree shaking across chunks

## Alternative: Single WASM + Lazy Components

If multi-crate chunking is too complex, use component-level lazy loading:

```rust
use ferric_core::component::*;

// Load component only when needed
async fn load_admin_panel() -> Result<ComponentRef, JsValue> {
    // Component is compiled in main bundle but not initialized
    create_component_lazy::<AdminPanel>().await
}
```

## Comparison

| Approach | Bundle Size | Load Time | Complexity |
|----------|-------------|-----------|------------|
| Single WASM | 100% | Slow | Low |
| 2-3 Chunks | 30% + 70% | Fast | Medium |
| Route-Based | 20% + 80% | Fastest | High |

## Migration Guide

### Step 1: Identify Chunks

Analyze your app:
```bash
# Find large modules
cargo bloat --release --crates
```

### Step 2: Extract to Crates

Create separate crates for large features:
```bash
ferric new admin --lib
ferric new dashboard --lib
```

### Step 3: Configure Chunking

Update `ferric.toml`:
```toml
[build.chunking]
enabled = true

[[build.chunking.chunks]]
name = "admin"
module = "admin"
preload = "route"
```

### Step 4: Update Routes

Use lazy loading in routes:
```rust
Route::new("/admin")
    .lazy_load_chunk("admin")
```

### Step 5: Build and Test

```bash
ferric build --release
ferric serve  # Test locally
```

## Debugging

### Check Chunk Manifest

```bash
cat dist/chunk-manifest.json
```

Should show:
```json
{
  "chunks": {
    "main": {
      "wasm_file": "main_bg.wasm",
      "size_kb": 245
    },
    "admin": {
      "wasm_file": "admin_bg.wasm",
      "size_kb": 127
    }
  }
}
```

### Verify Loading

Open browser console:
```javascript
// Check loader
console.log(window.FerricChunks);

// Test loading
await FerricChunks.loadChunk('admin');
```

### Network Tab

Watch chunks load:
1. Initial: main_bg.wasm
2. On route: admin_bg.wasm

## Performance Monitoring

Track chunk loading performance:

```javascript
performance.mark('chunk-load-start');
await FerricChunks.loadChunk('admin');
performance.mark('chunk-load-end');

performance.measure(
  'chunk-load',
  'chunk-load-start',
  'chunk-load-end'
);

const measure = performance.getEntriesByName('chunk-load')[0];
console.log(`Chunk loaded in ${measure.duration}ms`);
```

## Example Application

See `examples/chunked-app` for a complete example with:
- Main chunk (app shell + home)
- Admin chunk (admin panel)
- Dashboard chunk (analytics)
- Manual and automatic loading
- Route integration

## See Also

- [Build Configuration](./BUILD_CONFIG.md)
- [Router Guide](./ROUTER_GUIDE.md)
- [Deployment Guide](./DEPLOYMENT.md)
- [Performance Guide](./PERFORMANCE.md)

---

Built with 🦀 Rust and ❤️ for the Ferric framework

