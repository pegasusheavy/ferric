# Cache Busting Integration Guide

This guide explains how cache busting automatically wires up assets in your HTML during the build process.

## Automatic Integration

Cache busting is **fully automatic** in release builds. No manual intervention needed!

```bash
ferric build --release
# ✓ Compiles CSS and JS
# ✓ Generates content hashes
# ✓ Renames files with hashes
# ✓ Updates all HTML references automatically
```

## What Gets Processed

### 1. Directories Scanned

The build process automatically scans and processes:

```
dist/
├── index.html          # Updated with all hashed references
├── styles.css          # → styles.a1b2c3d4.css
├── pkg/               # WASM and JS files
│   ├── app.js         # → app.f5e6d7c8.js
│   └── app_bg.wasm    # → app_bg.e9f0a1b2.wasm
├── styles/            # Additional stylesheets
│   └── theme.css      # → theme.c3d4e5f6.css
└── assets/            # Images, fonts (if enabled)
    ├── logo.png       # → logo.b4c5d6e7.png
    └── font.woff2     # → font.d7e8f9a0.woff2
```

### 2. File Types Hashed

Configured in `ferric.toml`:

```toml
[build.cache_busting]
css = true     # .css files
js = true      # .js, .wasm files
assets = false # .png, .jpg, .svg, .woff, .woff2, etc.
```

## HTML Reference Patterns

All these patterns are automatically detected and updated:

### Basic References

```html
<!-- Before build -->
<link rel="stylesheet" href="styles.css">
<script src="app.js"></script>

<!-- After build -->
<link rel="stylesheet" href="styles.a1b2c3d4.css">
<script src="app.f5e6d7c8.js"></script>
```

### Path Variations

```html
<!-- Relative paths -->
<link href="./styles.css">          → href="./styles.a1b2c3d4.css"
<script src="./app.js"></script>    → src="./app.f5e6d7c8.js"

<!-- Absolute paths -->
<link href="/styles.css">            → href="/styles.a1b2c3d4.css"
<script src="/app.js"></script>      → src="/app.f5e6d7c8.js"

<!-- In subdirectories -->
<link href="styles/theme.css">       → href="styles/theme.c3d4e5f6.css"
<script src="pkg/app.js"></script>   → src="pkg/app.f5e6d7c8.js"
```

### Module Imports

```html
<script type="module">
  // Before
  import init from './pkg/app.js';
  import { render } from './app.js';
  
  // After (automatically updated)
  import init from './pkg/app.f5e6d7c8.js';
  import { render } from './app.c9d0e1f2.js';
</script>
```

### Preload Hints

```html
<!-- Before -->
<link rel="preload" href="styles.css" as="style">
<link rel="prefetch" href="admin.js">
<link rel="modulepreload" href="pkg/app.js">

<!-- After -->
<link rel="preload" href="styles.a1b2c3d4.css" as="style">
<link rel="prefetch" href="admin.e5f6a7b8.js">
<link rel="modulepreload" href="pkg/app.f5e6d7c8.js">
```

### Asset References

```html
<!-- When assets = true -->
<img src="logo.png">                 → src="logo.b4c5d6e7.png"
<img src="assets/logo.png">          → src="assets/logo.b4c5d6e7.png"
```

## Build Output

During build, you'll see detailed output:

```bash
ferric build --release

→ Building ferric-app in release mode...
  → Compiling styles...
  ✓ Styles compiled

  → Building browser target...
    ✓ Browser build complete

  → Applying cache busting...
    → Processing pkg/ directory...
      ✓ Hashed 4 files in pkg/
    → Processing styles/ directory...
      ✓ Hashed 2 files in styles/
      ✓ Hashed styles.css
    → Updating index.html references...
    ✓ Updated 7 asset references in index.html
      → styles.css → styles.a1b2c3d4.css
      → app.js → app.f5e6d7c8.js
      → app_bg.wasm → app_bg.e9f0a1b2.wasm
      ... and 4 more

✓ Build complete!
```

## Configuration Examples

### Minimal (Default)

Hashes CSS and JS only:

```toml
[build.cache_busting]
enabled = true
```

### Everything

Hash all asset types:

```toml
[build.cache_busting]
enabled = true
css = true
js = true
assets = true
```

### Custom Strategy

Use SHA256 for higher security:

```toml
[build.cache_busting]
enabled = true
strategy = "sha256"
hash_length = 16
```

### Selective

Only CSS files:

```toml
[build.cache_busting]
enabled = true
css = true
js = false
assets = false
```

## Verification

After building, verify the integration:

```bash
# Check hashed files exist
ls -la dist/
ls -la dist/pkg/

# Check HTML was updated
cat dist/index.html | grep -E "\.(css|js|wasm)"
# Should show hashed filenames

# Test in browser
ferric serve
# Open http://localhost:8080
# Check Network tab - all assets should have hashes
```

## Advanced Usage

### Custom HTML Structure

Works with any HTML structure:

```html
<!DOCTYPE html>
<html>
<head>
  <!-- Main styles -->
  <link rel="stylesheet" href="styles.css">
  
  <!-- Theme styles -->
  <link rel="stylesheet" href="styles/theme.css">
  
  <!-- Preload WASM -->
  <link rel="modulepreload" href="pkg/app.js">
</head>
<body>
  <!-- App container -->
  <div id="app"></div>
  
  <!-- Main script -->
  <script type="module">
    import init, { start } from './pkg/app.js';
    
    async function main() {
      await init();
      start();
    }
    
    main();
  </script>
</body>
</html>
```

All references are automatically updated!

### Multiple HTML Files

Process additional HTML files:

```rust
// In custom build script
use ferric_cli::cache_busting::update_html_references;
use std::collections::HashMap;

let mappings: HashMap<String, String> = /* your mappings */;

// Update multiple HTML files
for html_file in &["index.html", "admin.html", "about.html"] {
    let path = format!("dist/{}", html_file);
    update_html_references(Path::new(&path), &mappings)?;
}
```

### Manual Control

Disable automatic updates and do it manually:

```toml
[build.cache_busting]
enabled = false  # Disable auto-processing
```

Then use the API directly:

```rust
use ferric_cli::cache_busting::{
    apply_cache_busting,
    Strategy,
    update_html_references,
};

// Hash a specific file
let (new_path, original) = apply_cache_busting(
    Path::new("dist/styles.css"),
    Strategy::Md5,
    8,
)?;

// Update HTML
let mut mappings = HashMap::new();
mappings.insert(original, new_path.file_name().unwrap().to_str().unwrap().to_string());
update_html_references(Path::new("dist/index.html"), &mappings)?;
```

## Troubleshooting

### Assets Not Loading (404)

**Problem**: Browser shows 404 for assets

**Check**:
```bash
# Verify files were hashed
ls dist/pkg/
ls dist/styles/

# Check HTML was updated
grep -n "\.css\|\.js" dist/index.html
```

**Solution**: Ensure paths in HTML match hashed filenames

### References Not Updated

**Problem**: HTML still has original filenames

**Check**:
```bash
# View build output
ferric build --release 2>&1 | grep "cache busting"
```

**Solution**:
1. Check `cache_busting.enabled = true`
2. Verify you're building in release mode
3. Check file paths match exactly

### Partial Updates

**Problem**: Some references updated, others not

**Common Cause**: Path mismatch

```html
<!-- Won't match if file is in pkg/ -->
<script src="app.js"></script>

<!-- Will match -->
<script src="pkg/app.js"></script>
```

**Solution**: Use correct relative paths in HTML

## Best Practices

### 1. Use Consistent Paths

Keep paths consistent between dev and production:

```html
<!-- Good - works in dev and prod -->
<link href="styles.css">
<script src="pkg/app.js"></script>

<!-- Bad - confusing -->
<link href="/static/styles.css">
```

### 2. Preload Critical Assets

Help browsers discover hashed files early:

```html
<link rel="modulepreload" href="pkg/app.js">
<link rel="preload" href="styles.css" as="style">
```

### 3. Use Module Imports

Modern ES modules work seamlessly:

```html
<script type="module">
  import init from './pkg/app.js';
  await init();
</script>
```

### 4. Test Locally

Always test with cache busting enabled:

```bash
# Build with cache busting
ferric build --release

# Serve and test
cd dist && python3 -m http.server 8080
```

### 5. Monitor Build Output

Watch for warnings during build:

```bash
ferric build --release 2>&1 | tee build.log
```

## Integration with CI/CD

Cache busting works automatically in CI/CD:

```yaml
# .github/workflows/deploy.yml
- name: Build with ferric-cli
  run: |
    cd web
    ferric build --release --target browser
    # Cache busting applied automatically!

- name: Verify cache busting
  run: |
    # Check hashed files exist
    test -f web/dist/styles.*.css || exit 1
    test -f web/dist/pkg/app.*.js || exit 1
    
    # Verify HTML updated
    grep -q "\..*\.css" web/dist/index.html || exit 1
```

## Performance Impact

Cache busting adds minimal overhead:

| Step | Time Added |
|------|------------|
| Hashing files | ~10-50ms per file |
| Renaming | <1ms per file |
| HTML updates | ~5-10ms |
| **Total** | **<100ms** |

Negligible compared to:
- CSS compilation: 1-5s
- WASM compilation: 5-30s
- Optimization: 10-60s

## Monitoring

Track cache busting effectiveness:

```javascript
// In your app
performance.mark('app-start');

window.addEventListener('load', () => {
  performance.mark('app-loaded');
  performance.measure('load-time', 'app-start', 'app-loaded');
  
  const measure = performance.getEntriesByName('load-time')[0];
  console.log(`App loaded in ${measure.duration}ms`);
  
  // Check cache hits
  const resources = performance.getEntriesByType('resource');
  const cached = resources.filter(r => r.transferSize === 0);
  console.log(`Cached: ${cached.length}/${resources.length} resources`);
});
```

## See Also

- [Cache Busting Guide](./CACHE_BUSTING_GUIDE.md) - Complete feature documentation
- [Build Configuration](./BUILD_CONFIG.md) - All build options
- [Deployment Guide](./DEPLOYMENT.md) - Production deployment
- [Performance Guide](./PERFORMANCE.md) - Optimization strategies

---

Built with 🦀 Rust and ❤️ for the Ferric framework

