# Ferric Documentation Website

This directory contains the GitHub Pages website for Ferric framework documentation.

## Structure

```
web/
├── index.html              # Main landing page
├── docs.html               # Documentation hub
├── benchmarks.html         # Performance benchmarks results
├── run-benchmarks.html     # Interactive benchmark runner
├── docs/                   # Generated documentation pages
│   ├── getting-started.html
│   ├── quick-start.html
│   ├── components.html
│   ├── async-support.html
│   ├── di-architecture.html
│   ├── file-transfer.html
│   ├── http-client.html
│   ├── macros.html
│   ├── decorators.html
│   └── benchmarking.html
├── styles/                 # Stylesheets
│   └── docs.css
└── pkg/                    # Generated WASM packages (after build)
```

## Building

### Generate Documentation HTML

From the project root:

```bash
bash scripts/generate-docs-html.sh
```

This converts markdown files from `docs/` into styled HTML pages in `web/docs/`.

### Build Benchmark Runner

```bash
cd benchmarks/todo-comparison/benchmark-runner
wasm-pack build --target web --out-dir ../../../web/pkg
```

## Development

### Local Server

```bash
cd web
python3 -m http.server 8000
```

Then visit: `http://localhost:8000`

### Live Reload (optional)

```bash
npm install -g live-server
cd web
live-server
```

## Deployment

Deployment to GitHub Pages is automated via GitHub Actions:

- **Trigger**: Push to `main` branch or changes to `docs/` or `web/`
- **Workflow**: `.github/workflows/docs.yml`
- **URL**: `https://your-username.github.io/ferric/`

Manual deployment:

```bash
# Build everything
bash scripts/generate-docs-html.sh
cd benchmarks/todo-comparison/benchmark-runner
wasm-pack build --target web --out-dir ../../../web/pkg
cd ../../..

# Deploy
git subtree push --prefix web origin gh-pages
```

## Features

### 📚 Documentation Hub (`docs.html`)

- **Organized by Topic**: Core concepts, reactivity, templates, routing, forms, HTTP, etc.
- **Search Functionality**: Filter documentation by keywords
- **Quick Links**: Jump to commonly accessed pages
- **Badge System**: Identify guides, APIs, and new features
- **Responsive Design**: Mobile-friendly layout

### ⚡ Benchmarks (`benchmarks.html`)

- **Performance Comparison**: Ferric vs React TodoMVC
- **Visual Charts**: Bar charts for each operation
- **Detailed Tables**: Complete benchmark results
- **Real-time Data**: Updates from localStorage
- **Summary Statistics**: Average times, performance gains

### 🏃 Benchmark Runner (`run-benchmarks.html`)

- **Interactive Execution**: Run benchmarks in your browser
- **Configurable**: Adjust operations and iterations
- **Progress Tracking**: Real-time progress bars
- **Live Logging**: See benchmark execution details
- **Results Preview**: Quick summary before viewing full results

### 📄 Generated Docs (`docs/*.html`)

- **Markdown Rendering**: Using marked.js
- **Syntax Highlighting**: Code blocks with highlight.js
- **Sidebar Navigation**: Easy navigation between guides
- **Styled Layout**: Professional documentation design
- **Responsive**: Works on all devices

## Updating Documentation

### 1. Edit Markdown Files

Edit files in the `docs/` directory:

```bash
vim docs/QUICK_START.md
vim docs/COMPONENTS_GUIDE.md
# etc.
```

### 2. Regenerate HTML

```bash
bash scripts/generate-docs-html.sh
```

### 3. Commit and Push

```bash
git add docs/ web/docs/
git commit -m "docs: update documentation"
git push
```

GitHub Actions will automatically deploy the changes.

## Customization

### Styling

Edit `web/styles/docs.css` to customize the documentation appearance.

### Templates

Edit `scripts/generate-docs-html.sh` to modify the HTML template used for documentation pages.

### Benchmark Data

Benchmark results are stored in `localStorage`. To update sample data, edit `web/benchmarks.html`:

```javascript
const sampleData = {
    // Update benchmark results here
};
```

## Dependencies

### Runtime

- **marked.js**: Markdown parsing (CDN)
- **highlight.js**: Syntax highlighting (CDN)

### Build

- Bash (for script execution)
- wasm-pack (for WASM builds)
- Python 3 (for local dev server, optional)

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Contributing

1. Edit markdown files in `docs/`
2. Run `bash scripts/generate-docs-html.sh`
3. Test locally with `python3 -m http.server 8000`
4. Submit a PR

## License

MIT

