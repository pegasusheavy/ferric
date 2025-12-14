# Ferric Markdown

Client-side markdown to HTML compilation for the Ferric framework.

## Features

- ✅ **Fast Parsing**: Uses `pulldown-cmark` for efficient markdown parsing
- ✅ **HTML Sanitization**: Built-in XSS protection with `ammonia`
- ✅ **GitHub Flavored Markdown**: Full GFM support including tables, strikethrough, and task lists
- ✅ **Syntax Highlighting**: Optional code syntax highlighting (feature-gated)
- ✅ **Injectable Service**: Fully integrated with Ferric's DI system
- ✅ **Template Pipes**: Use markdown directly in templates
- ✅ **Customizable**: Configure rendering options via DI tokens
- ✅ **WASM Optimized**: Works efficiently in the browser

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ferric-markdown = { path = "../ferric-markdown" }
```

With syntax highlighting:

```toml
[dependencies]
ferric-markdown = { path = "../ferric-markdown", features = ["syntax-highlighting"] }
```

## Quick Start

### Basic Usage

```rust
use ferric_markdown::{MarkdownService, MarkdownModule};
use ferric::di::Injector;

// Setup DI
let injector = Injector::root();
MarkdownModule::provide(&injector);

// Use the service
let md_service = injector.resolve::<MarkdownService>().unwrap();
let html = md_service.render("# Hello, **World**!");
```

### In Templates

```html
<!-- Basic markdown rendering -->
<div [innerHTML]="content | markdown"></div>

<!-- Inline rendering (no paragraph tags) -->
<span [innerHTML]="text | markdown:'inline'"></span>

<!-- Wrapped in container -->
<div [innerHTML]="content | markdown:'wrapped'"></div>
```

### In Components

```rust
use ferric_markdown::MarkdownService;
use ferric::prelude::*;

#[component(selector = "markdown-viewer")]
struct MarkdownViewerComponent {
    markdown_service: Rc<MarkdownService>,
    content: Signal<String>,
}

impl Injectable for MarkdownViewerComponent {
    fn create(injector: &Injector) -> Self {
        Self {
            markdown_service: injector.resolve_required::<MarkdownService>(),
            content: signal(String::new()),
        }
    }
}

impl MarkdownViewerComponent {
    fn render_content(&self) -> String {
        self.markdown_service.render(&self.content.get())
    }
}
```

## Configuration

### GitHub Flavored Markdown

```rust
let injector = Injector::root();
MarkdownModule::provide_github(&injector);
```

### Minimal Configuration

```rust
let injector = Injector::root();
MarkdownModule::provide_minimal(&injector);
```

### Custom Configuration

```rust
use ferric_markdown::{MarkdownModule, markdown_tokens};

let injector = Injector::root();

// Configure via tokens
injector.register_token(&markdown_tokens::MARKDOWN_ENABLE_GFM, true);
injector.register_token(&markdown_tokens::MARKDOWN_ENABLE_TABLES, true);
injector.register_token(&markdown_tokens::MARKDOWN_CSS_CLASS_PREFIX, "custom-".to_string());

MarkdownModule::provide(&injector);
```

### Programmatic Configuration

```rust
use ferric_markdown::{MarkdownConfig, MarkdownService};

let config = MarkdownConfig {
    sanitize_html: true,
    enable_gfm: true,
    enable_tables: true,
    enable_strikethrough: true,
    enable_tasklists: true,
    enable_syntax_highlighting: true,
    ..Default::default()
};

let service = MarkdownService::new(config);
```

## Configuration Tokens

- `MARKDOWN_SANITIZE_HTML` - Enable HTML sanitization (default: true)
- `MARKDOWN_ALLOW_DANGEROUS_HTML` - Allow unsafe HTML (default: false)
- `MARKDOWN_ENABLE_GFM` - Enable GitHub Flavored Markdown (default: true)
- `MARKDOWN_ENABLE_TABLES` - Enable table support (default: true)
- `MARKDOWN_ENABLE_STRIKETHROUGH` - Enable strikethrough (default: true)
- `MARKDOWN_ENABLE_TASKLISTS` - Enable task lists (default: true)
- `MARKDOWN_ENABLE_FOOTNOTES` - Enable footnotes (default: true)
- `MARKDOWN_ENABLE_SYNTAX_HIGHLIGHTING` - Enable syntax highlighting (default: false)
- `MARKDOWN_SYNTAX_THEME` - Syntax highlighting theme
- `MARKDOWN_CSS_CLASS_PREFIX` - CSS class prefix for generated elements (default: "md-")

## Examples

### Rendering Markdown

```rust
let service = MarkdownService::default();

// Basic markdown
let html = service.render("# Hello\n\nThis is **bold**.");

// Inline markdown
let renderer = MarkdownRenderer::default();
let inline_html = renderer.render_inline("**bold** text");
```

### GitHub Flavored Markdown

```markdown
# GitHub Flavored Markdown

## Tables
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |

## Strikethrough
~~This is deleted text~~

## Task Lists
- [x] Completed task
- [ ] Incomplete task

## Autolinks
https://example.com
```

### Code Blocks with Syntax Highlighting

```markdown
\```rust
fn main() {
    println!("Hello, world!");
}
\```
```

## Security

By default, all HTML output is sanitized using `ammonia` to prevent XSS attacks. This removes dangerous elements like `<script>`, `<iframe>`, and event handlers.

### Safe by Default

```rust
let service = MarkdownService::default();
let safe_html = service.render("# Safe Content");
// Output is sanitized
```

### Unsafe Mode (Not Recommended)

```rust
// ⚠️ Only use with trusted content!
let injector = Injector::root();
MarkdownModule::provide_unsafe(&injector);
```

## API Reference

### MarkdownService

Main service for rendering markdown:

```rust
impl MarkdownService {
    pub fn new(config: MarkdownConfig) -> Self;
    pub fn default() -> Self;
    pub fn render(&self, markdown: &str) -> String;
    pub fn render_with_options(&self, markdown: &str, options: &RenderOptions) -> String;
}
```

### MarkdownPipe

Pipe for templates:

```rust
impl Pipe for MarkdownPipe {
    fn name(&self) -> &'static str;
    fn transform(&self, value: &str, args: &PipeArgs) -> String;
}
```

### MarkdownModule

DI module:

```rust
impl MarkdownModule {
    pub fn provide(injector: &Injector);
    pub fn provide_github(injector: &Injector);
    pub fn provide_minimal(injector: &Injector);
    pub fn provide_with_config<F>(injector: &Injector, configure: F);
}
```

## Performance

- **Parsing**: Uses `pulldown-cmark`, one of the fastest markdown parsers
- **Sanitization**: `ammonia` provides efficient HTML cleaning
- **WASM Optimized**: Compiled to WebAssembly for client-side use
- **Lazy Loading**: Service is only instantiated when needed via DI

## License

MIT

