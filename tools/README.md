# @ferric/tools

Build tools for Ferric framework projects. Uses SWC for fast TypeScript/JavaScript compilation.

## Features

- **Fast Builds** - SWC-powered TypeScript compilation
- **HTML Generation** - Automatic index.html generation with WASM loading
- **Dev Server** - Built-in development server with hot reload
- **Project Scaffolding** - Initialize new Ferric projects

## Installation

```bash
pnpm add -D @ferric/tools
```

## CLI Commands

### `ferric build`

Build the project for production.

```bash
ferric build [options]

Options:
  -o, --outdir <dir>    Output directory (default: "dist")
  -w, --watch           Watch for changes and rebuild
  --no-minify           Disable minification
  --no-sourcemap        Disable source maps
  -c, --config <file>   Path to ferric.config.js
```

### `ferric dev`

Start development server with hot reload.

```bash
ferric dev [options]

Options:
  -p, --port <port>     Port to listen on (default: "3000")
  -o, --open            Open browser automatically
  -c, --config <file>   Path to ferric.config.js
```

### `ferric init`

Initialize a new Ferric project.

```bash
ferric init [name] [options]

Options:
  -t, --template <template>   Template to use (default: "default")
```

## Configuration

Create a `ferric.config.js` in your project root:

```javascript
import { defineConfig } from '@ferric/tools';

export default defineConfig({
  name: 'my-app',
  entry: './src/main.ts',
  outDir: 'dist',

  html: {
    title: 'My Ferric App',
    meta: {
      description: 'Built with Ferric',
    },
  },

  wasm: {
    pkgDir: './pkg',
    moduleName: 'my_app',
  },

  build: {
    minify: true,
    sourcemap: true,
    target: 'es2022',
  },

  dev: {
    port: 3000,
    open: true,
  },
});
```

## Programmatic API

```typescript
import { build, dev, generateHtml } from '@ferric/tools';

// Build project
await build({
  outdir: 'dist',
  minify: true,
});

// Generate HTML
const html = await generateHtml({
  title: 'My App',
  wasmPath: 'my_app_bg.wasm',
  wasmBindingsPath: 'my_app.js',
});
```

## Generated HTML

The generated `index.html` includes:

- WASM module preloading for faster startup
- Automatic WASM initialization script
- Loading indicator while WASM loads
- SPA-friendly structure
- Default styling reset

### Custom Template

Create a custom `index.html` template:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{title}}</title>
  {{headContent}}
</head>
<body>
  <{{rootSelector}}></{{rootSelector}}>
  {{bodyContent}}
  {{wasmInit}}
</body>
</html>
```

Reference it in your config:

```javascript
export default defineConfig({
  html: {
    template: './src/index.html',
  },
});
```

## Development

```bash
# Build the tools
pnpm build

# Watch mode
pnpm watch

# Type check
pnpm typecheck
```

