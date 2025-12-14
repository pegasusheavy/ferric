# TailwindCSS 4 Integration Guide

Complete guide to using TailwindCSS 4 with Ferric Framework via the `ferric` CLI.

## What's New in TailwindCSS 4?

TailwindCSS 4 introduces major changes:

### ✨ No Configuration File

**Before (v3):**
```javascript
// tailwind.config.js
module.exports = {
  content: ["./src/**/*.{html,js}"],
  theme: {
    extend: {
      colors: {
        'brand': '#ff0000'
      }
    }
  }
}
```

**After (v4):**
```css
/* styles/app.css */
@import "tailwindcss";

@theme {
  --color-brand: #ff0000;
}
```

### 🚀 CSS-First Configuration

All configuration is now done directly in your CSS file using the `@theme` directive:

```css
@theme {
  /* Colors */
  --color-primary-500: #3b82f6;
  --color-secondary-500: #8b5cf6;

  /* Fonts */
  --font-sans: 'Inter', system-ui, sans-serif;
  --font-mono: 'Fira Code', monospace;

  /* Shadows */
  --shadow-custom: 0 4px 14px 0 rgb(0 0 0 / 0.1);

  /* Animations */
  --animate-custom: bounce 1s infinite;
}
```

### 📦 Simplified Installation

**Before (v3):**
```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init
```

**After (v4):**
```bash
npm install -D tailwindcss
# No init command needed!
```

## Using TailwindCSS 4 with Ferric CLI

### Installation

#### Option 1: Initialize New Project with TailwindCSS 4

```bash
# Create new project
ferric new my-app

# Navigate to project
cd my-app

# Initialize TailwindCSS 4
ferric tailwind --init

# Install dependencies
npm install
```

#### Option 2: Add to Existing Project

```bash
# In your existing Ferric project
ferric tailwind --init

# Install dependencies
npm install
```

### Project Structure

After initialization, your project will have:

```
my-app/
├── styles/
│   └── app.css          # TailwindCSS 4 entry point
├── dist/
│   └── styles.css       # Compiled output
├── package.json         # With TailwindCSS 4
└── src/
    └── ...
```

### The CSS Entry Point

`styles/app.css`:
```css
@import "tailwindcss";

@theme {
  /* Your custom theme configuration */
}

/* Your custom CSS */
```

## CLI Commands

### Build CSS Once

```bash
ferric tailwind
```

With custom input/output:
```bash
ferric tailwind -i styles/app.css -o dist/output.css
```

### Watch Mode (Development)

```bash
ferric tailwind --watch
```

### Minified Production Build

```bash
ferric tailwind --minify
```

### All Options

```bash
ferric tailwind \
  --input styles/app.css \
  --output dist/styles.css \
  --watch \
  --minify
```

## Rust Theme Example

Here's the complete Rust-themed configuration from the docs:

```css
@import "tailwindcss";
@plugin "@tailwindcss/typography";

@theme {
  /* Rust Orange Palette */
  --color-rust-50: #fef6ee;
  --color-rust-100: #fdecd7;
  --color-rust-200: #fbd5ae;
  --color-rust-300: #f8b87a;
  --color-rust-400: #f59144;
  --color-rust-500: #f2711e;
  --color-rust-600: #e35314;
  --color-rust-700: #bc3c13;
  --color-rust-800: #963117;
  --color-rust-900: #792a16;
  --color-rust-950: #411309;

  /* Ferric Orange Palette */
  --color-ferric-50: #fef5ee;
  --color-ferric-100: #fde9d7;
  --color-ferric-200: #fad0ae;
  --color-ferric-300: #f7af7a;
  --color-ferric-400: #f38344;
  --color-ferric-500: #f0641e;
  --color-ferric-600: #e14914;
  --color-ferric-700: #bb3513;
  --color-ferric-800: #952c17;
  --color-ferric-900: #782716;
  --color-ferric-950: #401109;

  /* Custom Colors */
  --color-crab: #ce422b;
  --color-gear: #5c5c5c;

  /* Fonts */
  --font-rust: 'Fira Sans', system-ui, sans-serif;
  --font-mono: 'Fira Code', monospace;

  /* Shadows */
  --shadow-rust: 0 4px 14px 0 rgb(206 66 43 / 0.2);
  --shadow-rust-lg: 0 10px 40px 0 rgb(206 66 43 / 0.25);
}

/* Custom Components */
@layer components {
  .btn-rust {
    background: linear-gradient(135deg, var(--color-rust-500) 0%, var(--color-crab) 100%);
    color: white;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    font-weight: 600;
  }
}
```

## Usage in HTML

```html
<!DOCTYPE html>
<html>
<head>
  <link href="/dist/styles.css" rel="stylesheet">
</head>
<body>
  <!-- Use standard Tailwind classes -->
  <div class="bg-rust-500 text-white p-4">
    Rust-themed content
  </div>

  <!-- Use custom components -->
  <button class="btn-rust">
    Click Me
  </button>
</body>
</html>
```

## Integration with Ferric Build Pipeline

### Development Workflow

1. **Start dev server with CSS watch:**
```bash
# Terminal 1: Watch and compile CSS
ferric tailwind --watch

# Terminal 2: Serve project
ferric serve --open
```

2. **Or use npm scripts:**
```json
{
  "scripts": {
    "dev": "ferric tailwind --watch & ferric serve",
    "build": "ferric tailwind --minify && ferric build --release"
  }
}
```

### Production Build

```bash
# Build optimized CSS
ferric tailwind --minify --output dist/styles.min.css

# Build Wasm
ferric build --release
```

## Advanced Configuration

### Using Plugins

```css
@import "tailwindcss";

/* Official plugins */
@plugin "@tailwindcss/typography";
@plugin "@tailwindcss/forms";
@plugin "@tailwindcss/container-queries";

@theme {
  /* Your theme */
}
```

### Custom Utilities

```css
@layer utilities {
  .text-gradient-rust {
    background: linear-gradient(135deg, var(--color-rust-500) 0%, var(--color-crab) 100%);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
}
```

### Responsive Variants

All standard Tailwind responsive variants work:

```html
<div class="text-sm md:text-base lg:text-lg xl:text-xl">
  Responsive text
</div>
```

### Dark Mode

```css
@theme {
  --color-background: white;
  --color-text: black;

  @media (prefers-color-scheme: dark) {
    --color-background: black;
    --color-text: white;
  }
}
```

## Migration from TailwindCSS 3

### 1. Remove Config File

```bash
rm tailwind.config.js
rm postcss.config.js  # If using PostCSS
```

### 2. Update package.json

```json
{
  "devDependencies": {
    "tailwindcss": "^4.0.0"  // Remove postcss, autoprefixer
  }
}
```

### 3. Convert Config to @theme

**Before (tailwind.config.js):**
```javascript
module.exports = {
  theme: {
    extend: {
      colors: {
        'brand': {
          500: '#ff0000'
        }
      }
    }
  }
}
```

**After (styles/app.css):**
```css
@theme {
  --color-brand-500: #ff0000;
}
```

### 4. Update Build Commands

Replace any build scripts:

```json
{
  "scripts": {
    "build:css": "ferric tailwind --minify"
  }
}
```

## Performance Tips

### 1. Production Minification

Always minify for production:
```bash
ferric tailwind --minify
```

### 2. Watch Mode Debouncing

The CLI automatically debounces file changes to prevent excessive rebuilds.

### 3. Selective Imports

Only import what you need:
```css
/* Instead of full import */
@import "tailwindcss";

/* You can selectively import */
@import "tailwindcss/preflight";
@import "tailwindcss/utilities";
```

### 4. CDN for Development

For quick prototyping, use the CDN:
```html
<script src="https://cdn.tailwindcss.com?plugins=typography"></script>
```

## Troubleshooting

### CSS Not Compiling

1. Check that npx is available:
```bash
npx --version
```

2. Verify TailwindCSS 4 is installed:
```bash
npm list tailwindcss
```

3. Check input file exists:
```bash
ls -la styles/app.css
```

### Classes Not Working

1. Verify CSS is linked in HTML:
```html
<link href="/dist/styles.css" rel="stylesheet">
```

2. Check browser console for 404 errors

3. Rebuild CSS:
```bash
ferric tailwind
```

### Watch Mode Not Detecting Changes

1. Restart watch mode:
```bash
# Press Ctrl+C, then restart
ferric tailwind --watch
```

2. Check file permissions
3. Try absolute paths

## Examples

### Ferric Component with Tailwind

```rust
use ferric_core::prelude::*;

#[component]
struct MyButton {
    label: String,
}

impl Component for MyButton {
    fn render(&self) -> Html {
        html! {
            <button class="btn-rust hover:shadow-lg transition-all">
                {&self.label}
            </button>
        }
    }
}
```

### Responsive Card

```html
<div class="card-rust p-4 md:p-6 lg:p-8">
  <h2 class="text-lg md:text-xl lg:text-2xl text-gradient-rust">
    Responsive Card
  </h2>
  <p class="text-gear mt-2">
    Automatically adapts to screen size
  </p>
</div>
```

## Resources

- [TailwindCSS 4 Documentation](https://tailwindcss.com/docs)
- [Ferric CLI Documentation](./CLI_GUIDE.md)
- [Rust Theme Example](../web/styles/rust-theme.css)

## License

MIT

