# Chunked App Example

Demonstrates WASM code splitting with multiple lazy-loaded chunks.

## Structure

```
chunked-app/
├── core/          # Main chunk (app shell)
├── admin/         # Admin chunk (lazy-loaded)
└── dashboard/     # Dashboard chunk (lazy-loaded)
```

## Features

- **Main Chunk**: App shell, router, home page (~100KB)
- **Admin Chunk**: Admin panel, lazy-loaded on route (~50KB)
- **Dashboard Chunk**: Analytics, lazy-loaded when idle (~75KB)

## Building

```bash
# Build with chunking
cd examples/chunked-app
ferric build --release

# Output structure
dist/
├── index.html
├── pkg/
│   ├── main/         # Always loaded
│   ├── admin/        # Loaded on-demand
│   └── dashboard/    # Loaded on-demand
└── chunk-manifest.json
```

## Running

```bash
# Serve locally
ferric serve

# Open http://localhost:8080
# Click "Load Admin Chunk" to see lazy loading in action
```

## Benefits

| Scenario | Without Chunking | With Chunking |
|----------|------------------|---------------|
| Initial Load | 225 KB | 100 KB (-55%) |
| Admin Access | 225 KB | 150 KB |
| Analytics Only | 225 KB | 175 KB |

## Configuration

See `ferric.toml` for chunking configuration:

```toml
[build.chunking]
enabled = true

[[build.chunking.chunks]]
name = "admin"
module = "admin"
preload = "route"
```

## Code Structure

### Main Chunk (core/)
- App initialization
- Router setup
- Home page component
- Chunk loading APIs

### Admin Chunk (admin/)
- Admin panel components
- User management
- Settings interface

### Dashboard Chunk (dashboard/)
- Analytics components
- Chart rendering
- Report generation

## Testing

Open browser DevTools Network tab to see:
1. **Initial load**: Only main chunk
2. **Click "Load Admin"**: Admin chunk loads
3. **Wait for idle**: Dashboard chunk preloads

## See Also

- [WASM Chunking Guide](../../docs/WASM_CHUNKING_GUIDE.md)
- [Build Configuration](../../docs/BUILD_CONFIG.md)

