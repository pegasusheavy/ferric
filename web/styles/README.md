# Ferric Rust Theme for TailwindCSS

A comprehensive Rust-inspired TailwindCSS theme for the Ferric Framework documentation website.

## 🎨 Color Palette

### Rust Orange (Primary)
```css
rust: {
  50: '#fef6ee',   /* Lightest tint */
  100: '#fdecd7',
  200: '#fbd5ae',
  300: '#f8b87a',
  400: '#f59144',
  500: '#f2711e',  /* Primary Rust orange */
  600: '#e35314',
  700: '#bc3c13',
  800: '#963117',
  900: '#792a16',
  950: '#411309',  /* Darkest shade */
}
```

### Ferric Orange (Secondary)
```css
ferric: {
  50: '#fef5ee',
  100: '#fde9d7',
  200: '#fad0ae',
  300: '#f7af7a',
  400: '#f38344',
  500: '#f0641e',  /* Ferric brand color */
  600: '#e14914',
  700: '#bb3513',
  800: '#952c17',
  900: '#782716',
  950: '#401109',
}
```

### Crab Red (Accent)
```css
crab: {
  DEFAULT: '#ce422b',  /* Ferris the crab color */
  dark: '#a33621',
  light: '#e85c47',
}
```

### Gear Gray (Neutral)
```css
gear: {
  DEFAULT: '#5c5c5c',  /* Industrial gear gray */
  dark: '#2d2d2d',
  light: '#8c8c8c',
}
```

## 🧩 Components

### Buttons

```html
<!-- Primary Rust Button -->
<button class="btn-rust">Click Me</button>

<!-- Outline Button -->
<button class="btn-rust-outline">Outline</button>

<!-- Ferric Button -->
<button class="btn-ferric">Ferric Style</button>

<!-- Crab Button -->
<button class="btn-crab">Crab Accent</button>
```

### Cards

```html
<!-- Rust Card -->
<div class="card-rust">
  <div class="card-header-rust">Title</div>
  <div class="card-body">Content</div>
</div>

<!-- Ferric Card -->
<div class="card-ferric">
  <div class="card-header-ferric">Title</div>
  <div class="card-body">Content</div>
</div>
```

### Badges

```html
<span class="badge-rust">Beginner</span>
<span class="badge-ferric">Essential</span>
<span class="badge-crab">Popular</span>
```

### Links

```html
<a href="#" class="link-rust">Rust Link</a>
<a href="#" class="link-ferric">Ferric Link</a>
```

## 🎯 Layout Components

### Headers

```html
<header class="header-rust">
  <h1>Rust-themed Header</h1>
</header>

<header class="header-ferric">
  <h1>Ferric-themed Header</h1>
</header>
```

### Navigation

```html
<nav class="nav-rust">
  <button class="nav-item-rust">Item</button>
  <button class="nav-item-rust nav-item-active">Active</button>
</nav>
```

### Feature Grid

```html
<div class="feature-grid">
  <div class="feature-card">
    <div class="feature-icon">🚀</div>
    <h3>Feature Title</h3>
    <p>Description</p>
  </div>
</div>
```

## 📝 Documentation Components

### Sidebar

```html
<aside class="doc-sidebar">
  <a href="#" class="doc-link doc-link-active">Active Link</a>
  <a href="#" class="doc-link">Link</a>
</aside>
```

### Content

```html
<article class="doc-content">
  <!-- Prose content with automatic styling -->
</article>
```

### Table of Contents

```html
<nav class="toc">
  <h4 class="toc-title">On This Page</h4>
  <ul class="toc-list">
    <li><a href="#" class="toc-link">Section 1</a></li>
    <li><a href="#" class="toc-link">Section 2</a></li>
  </ul>
</nav>
```

### Breadcrumb

```html
<nav class="breadcrumb">
  <span class="breadcrumb-item">Home</span>
  <span class="breadcrumb-separator">›</span>
  <span class="breadcrumb-item">Docs</span>
</nav>
```

## 🔍 Interactive Components

### Search Box

```html
<div class="search-box">
  <svg class="search-icon"><!-- icon --></svg>
  <input type="text" class="search-input" placeholder="Search...">
</div>
```

## 🚨 Alerts

```html
<div class="alert-rust">Rust-themed alert</div>
<div class="alert-info">Info alert</div>
<div class="alert-success">Success alert</div>
<div class="alert-warning">Warning alert</div>
```

## 💻 Code Blocks

```html
<div class="code-rust">
  <div class="code-header">
    <span>example.rs</span>
    <button>Copy</button>
  </div>
  <pre><code>fn main() { }</code></pre>
</div>
```

### Syntax Highlighting Classes

```html
<code>
  <span class="rust-keyword">fn</span>
  <span class="rust-function">main</span>
  <span class="rust-string">"Hello"</span>
  <span class="rust-comment">// comment</span>
  <span class="rust-type">String</span>
  <span class="rust-number">42</span>
</code>
```

## ✨ Utilities

### Gradients

```html
<!-- Text gradients -->
<h1 class="text-gradient-rust">Gradient Text</h1>
<h1 class="text-gradient-ferric">Ferric Gradient</h1>

<!-- Background gradients -->
<div class="bg-rust-gradient">Content</div>
<div class="bg-ferric-gradient">Content</div>
```

### Effects

```html
<!-- Glow effects -->
<div class="glow-rust">Glowing content</div>
<div class="glow-ferric">Ferric glow</div>

<!-- Gear pattern background -->
<div class="bg-gears">Patterned background</div>
```

### Custom Scrollbar

```html
<div class="scrollbar-rust overflow-auto">
  Long scrollable content...
</div>
```

## 🎭 Animations

### Ferris Wave

```html
<span class="animate-ferris-wave">🦀</span>
```

### Rust Pulse

```html
<div class="animate-rust-pulse">Pulsing element</div>
```

### Gear Spin

```html
<div class="animate-gear-spin">⚙️</div>
```

### Loading Spinner

```html
<div class="spinner-rust"></div>
```

## 🏗️ Build Commands

```bash
# Install dependencies
npm install

# Build production CSS
npm run build:css

# Watch for changes (development)
npm run watch:css

# Development mode
npm run dev
```

## 📦 Installation

### 1. Install Dependencies

```bash
npm install -D tailwindcss @tailwindcss/typography autoprefixer postcss
```

### 2. Configure Tailwind

Copy `tailwind.config.js` to your project root.

### 3. Import Theme

```css
/* In your main CSS file */
@import './styles/rust-theme.css';
```

### 4. Build

```bash
npx tailwindcss -i ./styles/rust-theme.css -o ./dist/output.css
```

## 🎨 Customization

### Extending Colors

Add custom colors to the `extend` section in `tailwind.config.js`:

```javascript
theme: {
  extend: {
    colors: {
      'custom-rust': '#ff6b35',
    }
  }
}
```

### Adding Components

Add new components in `rust-theme.css`:

```css
@layer components {
  .my-component {
    @apply bg-rust-500 text-white p-4 rounded-lg;
  }
}
```

## 🌐 Browser Support

- Chrome/Edge: ✅ Full support
- Firefox: ✅ Full support
- Safari: ✅ Full support
- Mobile: ✅ Responsive design

## 📄 License

MIT License - Feel free to use in your own projects!

## 🦀 Credits

Inspired by:
- Rust programming language branding
- Ferris the crab 🦀
- Industrial gear aesthetics
- Modern web design trends

## 🔗 Links

- [TailwindCSS Documentation](https://tailwindcss.com/)
- [Ferric Framework](https://github.com/username/ferric)
- [Rust Programming Language](https://www.rust-lang.org/)

