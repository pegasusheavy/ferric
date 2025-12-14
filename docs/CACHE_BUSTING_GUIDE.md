# Cache Busting Guide

This guide explains how to use the cache busting feature in Ferric to ensure browsers don't serve stale assets.

## Overview

Cache busting adds content-based hashes to your asset filenames, forcing browsers to download new versions when files change. For example:

```
styles.css → styles.a1b2c3d4.css
app.js → app.f5e6d7c8.js
```

## Configuration

Add cache busting settings to your `ferric.toml`:

```toml
[build.cache_busting]
# Enable cache busting
enabled = true

# Hash strategy: "md5", "sha256", or "timestamp"
strategy = "md5"

# Length of hash to append
hash_length = 8

# Apply to CSS files
css = true

# Apply to JS/Wasm files
js = true

# Apply to other assets
assets = false
```

## Hash Strategies

### MD5 (Recommended)
- **Fast** content-based hash
- **8-character** hash is sufficient
- Detects any file changes
- Best for most use cases

```toml
strategy = "md5"
hash_length = 8
```

### SHA256
- **Cryptographically secure** hash
- **Slower** but more collision-resistant
- Useful for security-critical assets

```toml
strategy = "sha256"
hash_length = 16
```

### Timestamp
- **Build time** as version
- **Not content-based** (rebuilds change hash even if content unchanged)
- Simpler but less efficient

```toml
strategy = "timestamp"
```

## File Type Controls

Control which types of files get cache busting:

```toml
[build.cache_busting]
enabled = true

# CSS files (styles, tailwind output)
css = true

# JavaScript and WebAssembly files
js = true

# Images, fonts, and other assets
assets = false
```

## How It Works

### 1. Build Process

When you run `ferric build --release`:

1. **Compile** - Code and styles are built normally
2. **Hash** - Enabled file types are hashed
3. **Rename** - Files renamed with hash: `file.hash.ext`
4. **Update HTML** - References in `index.html` are automatically updated

### 2. Filename Format

The hash is inserted before the file extension:

```
Original:        styles.css
With hash:       styles.a1b2c3d4.css

Original:        app_bg.wasm
With hash:       app_bg.f5e6d7c8.wasm
```

### 3. HTML Updates

The build process automatically updates all references in your HTML:

```html
<!-- Before -->
<link rel="stylesheet" href="styles.css">
<script src="pkg/app.js"></script>

<!-- After -->
<link rel="stylesheet" href="styles.a1b2c3d4.css">
<script src="pkg/app.f5e6d7c8.js"></script>
```

## Usage Examples

### Development Build

Cache busting is **disabled** for dev builds:

```bash
ferric build
# No cache busting - faster builds
```

### Production Build

Cache busting is **enabled** for release builds (if configured):

```bash
ferric build --release
# Applies cache busting to configured file types
```

### Disable Cache Busting

```toml
[build.cache_busting]
enabled = false
```

Or only for specific file types:

```toml
[build.cache_busting]
enabled = true
css = true
js = false    # Disable for JS/Wasm
assets = false
```

## CI/CD Integration

Cache busting works automatically in CI/CD pipelines:

```yaml
- name: Build with ferric-cli
  run: |
    cd web
    ferric build --release --target browser
    # Cache busting applied automatically
```

## Best Practices

### 1. Enable for Production

Always enable cache busting for production builds:

```toml
[build.cache_busting]
enabled = true
strategy = "md5"
```

### 2. Use MD5 for Speed

MD5 provides good collision resistance with fast hashing:

```toml
strategy = "md5"
hash_length = 8
```

### 3. Hash CSS and JS

These change frequently and benefit most from cache busting:

```toml
css = true
js = true
assets = false  # Images rarely change
```

### 4. Set Appropriate Cache Headers

Configure your web server to cache hashed files aggressively:

```nginx
# Hashed assets can be cached forever
location ~* \.[0-9a-f]{8}\.(css|js|wasm)$ {
    expires max;
    add_header Cache-Control "public, immutable";
}
```

### 5. Keep Hash Length Reasonable

8 characters provides excellent collision resistance:

```toml
hash_length = 8  # 4.3 billion possible values
```

## Troubleshooting

### Assets Not Loading

**Problem**: 404 errors for hashed assets

**Solution**: Ensure `index.html` was updated correctly:

```bash
# Check build output
cat dist/index.html | grep "styles"
```

### Incorrect Hashes

**Problem**: Hash changes on every build despite no changes

**Solution**: Using `timestamp` strategy? Switch to `md5`:

```toml
strategy = "md5"  # Content-based
```

### Performance Issues

**Problem**: Builds are slow

**Solution**: Use MD5 instead of SHA256:

```toml
strategy = "md5"  # Faster
hash_length = 8
```

## Examples

### Standard Web App

```toml
[build.cache_busting]
enabled = true
strategy = "md5"
hash_length = 8
css = true
js = true
assets = false
```

### High Security App

```toml
[build.cache_busting]
enabled = true
strategy = "sha256"
hash_length = 16
css = true
js = true
assets = true
```

### Simple Versioning

```toml
[build.cache_busting]
enabled = true
strategy = "timestamp"
css = true
js = true
assets = false
```

## Verification

After building, verify cache busting worked:

```bash
# Build with cache busting
ferric build --release

# Check output filenames
ls -la dist/styles/
# Should see: styles.a1b2c3d4.css

# Check HTML references
grep "styles" dist/index.html
# Should see: href="styles.a1b2c3d4.css"
```

## Migration

### From No Cache Busting

1. Add configuration to `ferric.toml`:
   ```toml
   [build.cache_busting]
   enabled = true
   ```

2. Rebuild:
   ```bash
   ferric build --release
   ```

3. Deploy - no code changes needed!

### From Manual Versioning

1. Remove manual version parameters from URLs
2. Enable cache busting in config
3. Rebuild and deploy

## Performance Impact

### Build Time

- **MD5**: +10-50ms per file
- **SHA256**: +20-100ms per file
- **Timestamp**: <1ms (no hashing)

### Runtime

- **No impact** on runtime performance
- Hashing occurs at build time only
- Browsers receive static filenames

## Browser Compatibility

Works with all browsers - it's just renamed files!

- ✅ Chrome/Edge
- ✅ Firefox
- ✅ Safari
- ✅ Mobile browsers
- ✅ All HTTP caching proxies

## FAQ

**Q: Does this work with CDNs?**
A: Yes! CDNs will cache each unique filename separately.

**Q: What about source maps?**
A: Source maps are not currently cache busted.

**Q: Can I use custom hash functions?**
A: Currently supports MD5, SHA256, and timestamp.

**Q: Does it work with dynamic imports?**
A: Yes, but ensure your bundler preserves the import paths.

**Q: What about service workers?**
A: Update your service worker to handle hashed filenames dynamically.

## See Also

- [Build Configuration](./BUILD_CONFIG.md)
- [Deployment Guide](./DEPLOYMENT.md)
- [CI/CD Integration](./CICD.md)

---

Built with 🦀 Rust and ❤️ for the Ferric framework

