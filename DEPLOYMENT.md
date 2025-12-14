# Deployment Guide

This document explains how the Ferric documentation site is built and deployed to GitHub Pages.

## Branch Structure

### `develop` (or `main`)
- Contains all source code
- Rust components and Wasm code
- TailwindCSS source files
- Documentation markdown files
- Build scripts and CI/CD workflows

### `gh-pages`
- **Separate orphan branch** with no git history from develop
- Contains only built assets ready to serve
- Automatically updated by GitHub Actions
- Should **not** be edited manually

## Build Process

### Automated Deployment (Recommended)

GitHub Actions automatically builds and deploys on every push to `develop`:

1. **Checkout** the develop branch
2. **Install** Rust toolchain and wasm-pack
3. **Install** Node.js dependencies
4. **Build CSS** using TailwindCSS 4
5. **Build Wasm** application with wasm-pack
6. **Generate** documentation HTML from markdown
7. **Deploy** to gh-pages branch

**Workflow:** `.github/workflows/gh-pages-deploy.yml`

### Manual Deployment

If you need to build and deploy manually:

```bash
# Build the site
bash scripts/build-for-gh-pages.sh

# Switch to gh-pages branch
git checkout gh-pages

# Clear old content (except .git)
rm -rf * .github

# Copy new build
cp -r deploy/* .

# Commit and push
git add -A
git commit -m "Deploy documentation site"
git push origin gh-pages

# Switch back to develop
git checkout develop
```

## What Gets Deployed

The `gh-pages` branch contains:

```
gh-pages/
├── index.html              # Main entry point (Ferric app)
├── pkg/                    # Wasm module and JS bindings
│   ├── ferric_docs_site.js
│   ├── ferric_docs_site_bg.wasm
│   └── ...
├── styles/                 # Built CSS
│   └── dist/
│       └── rust-theme.min.css
├── docs/                   # Generated documentation HTML
│   ├── getting-started.html
│   ├── components.html
│   └── ...
├── assets/                 # Static assets (images, etc)
├── .nojekyll              # Disable Jekyll processing
└── README.md              # gh-pages branch README
```

## Build Script

**Location:** `scripts/build-for-gh-pages.sh`

This script:
- ✅ Checks prerequisites (wasm-pack, node)
- ✅ Installs npm dependencies
- ✅ Builds CSS with TailwindCSS 4
- ✅ Builds Wasm with wasm-pack
- ✅ Generates documentation HTML
- ✅ Prepares deployment directory
- ✅ Creates `.nojekyll` file
- ✅ Generates deployment README

## GitHub Pages Configuration

### Repository Settings

1. Go to **Settings** → **Pages**
2. Set **Source** to: Deploy from a branch
3. Set **Branch** to: `gh-pages` / `(root)`
4. Click **Save**

### Custom Domain (Optional)

To use a custom domain:

1. Add `CNAME` file in deployment:
   ```bash
   echo "ferric.rs" > deploy/CNAME
   ```

2. Configure DNS:
   ```
   CNAME record: www.ferric.rs → pegasusheavy.github.io
   A records for apex domain:
   185.199.108.153
   185.199.109.153
   185.199.110.153
   185.199.111.153
   ```

3. Enable **Enforce HTTPS** in repository settings

## URL Structure

- **Live site:** `https://pegasusheavy.github.io/ferric/`
- **Custom domain:** `https://ferric.rs/` (if configured)

### Routes (Client-side routing)

- `/` - Home page
- `/docs` - Documentation list
- `/docs/:slug` - Individual documentation
- `/benchmarks` - Performance benchmarks
- `/examples` - Code examples

## Build Requirements

### Development Environment

- **Rust**: 1.70+ with `wasm32-unknown-unknown` target
- **wasm-pack**: Latest version
- **Node.js**: 18+ (for TailwindCSS 4)
- **npm**: For dependency management

### CI Environment (GitHub Actions)

All requirements are automatically installed by the workflow.

## Troubleshooting

### Build Fails in CI

Check the Actions tab for error logs:
- **Rust errors**: Check `web/Cargo.toml` dependencies
- **Wasm errors**: Verify wasm-pack version
- **CSS errors**: Check TailwindCSS 4 syntax
- **Deploy errors**: Check branch permissions

### Local Build Issues

```bash
# Clean and rebuild
rm -rf web/pkg web/node_modules
cd web && npm install && cd ..
bash scripts/build-for-gh-pages.sh
```

### Site Not Updating

1. Check if workflow completed successfully
2. Clear browser cache (Ctrl+Shift+R)
3. Wait a few minutes for GitHub Pages CDN to update
4. Check if gh-pages branch was updated

### 404 Errors

- Ensure `.nojekyll` file exists in gh-pages
- Check that all files are in root of gh-pages branch
- Verify GitHub Pages is enabled in settings

## Performance Optimization

### Wasm Optimization

The build uses `--release` mode which:
- Optimizes for size (`opt-level = "z"`)
- Enables LTO (Link Time Optimization)
- Sets `codegen-units = 1`

### CSS Optimization

TailwindCSS 4 with `--minify`:
- Removes unused styles
- Minifies output
- Compresses with gzip-compatible output

### Caching Strategy

GitHub Pages automatically sets:
- Long cache times for Wasm/CSS
- Browser caching headers
- CDN edge caching

## Monitoring

### Site Analytics

To add analytics, include in `web/index.html`:

```html
<!-- Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA-XXXXX"></script>

<!-- or Plausible -->
<script defer data-domain="ferric.rs" src="https://plausible.io/js/script.js"></script>
```

### Uptime Monitoring

Consider using:
- UptimeRobot
- Pingdom
- GitHub Pages Status: https://www.githubstatus.com/

## Security

### Content Security Policy

Add to `web/index.html`:

```html
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self';
               script-src 'self' 'wasm-unsafe-eval';
               style-src 'self' 'unsafe-inline' fonts.googleapis.com;
               font-src 'self' fonts.gstatic.com;">
```

### HTTPS

GitHub Pages automatically provides HTTPS. Always:
- Enforce HTTPS in repository settings
- Use HTTPS URLs in links
- Set `Strict-Transport-Security` header if using custom domain

## Maintenance

### Updating Dependencies

```bash
# Update Rust dependencies
cargo update

# Update Node dependencies
cd web && npm update && cd ..

# Test build
bash scripts/build-for-gh-pages.sh
```

### Regenerating Documentation

```bash
# Run doc generator
bash scripts/generate-docs-html.sh

# Rebuild and deploy
bash scripts/build-for-gh-pages.sh
```

## License

The deployment scripts and configuration are part of the Ferric project and are available under the MIT license.

---

For questions or issues, please open an issue on GitHub: https://github.com/pegasusheavy/ferric/issues

