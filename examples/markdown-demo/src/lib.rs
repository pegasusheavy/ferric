//! Markdown rendering demo for Ferric.

use ferric_core::di::Injector;
use ferric_markdown::prelude::*;
use wasm_bindgen::prelude::*;
use std::rc::Rc;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"Markdown Demo Starting...".into());

    // Setup DI
    let injector = Rc::new(Injector::root());

    // Configure markdown with GitHub Flavored Markdown
    MarkdownModule::provide_github(&injector);

    // Examples
    basic_example(&injector);
    gfm_example(&injector);
    custom_config_example();
}

/// Basic markdown rendering example.
fn basic_example(injector: &Injector) {
    let service = injector.resolve::<MarkdownService>().unwrap();

    let markdown = r#"
# Hello, Ferric Markdown!

This is a **basic** example of rendering markdown to HTML on the fly.

## Features

- Fast parsing with `pulldown-cmark`
- XSS protection with HTML sanitization
- GitHub Flavored Markdown support
"#;

    let html = service.render(markdown);
    web_sys::console::log_1(&format!("Basic Example:\n{}", html).into());
}

/// GitHub Flavored Markdown example with tables and task lists.
fn gfm_example(injector: &Injector) {
    let service = injector.resolve::<MarkdownService>().unwrap();

    let markdown = r#"
# GitHub Flavored Markdown

## Tables
| Feature | Supported |
|---------|-----------|
| Tables  | ✅        |
| ~~Strikethrough~~ | ✅ |
| Task Lists | ✅    |

## Strikethrough
~~This text is deleted~~

## Task Lists
- [x] Parse markdown
- [x] Render to HTML
- [x] Sanitize output
- [ ] Add syntax highlighting
"#;

    let html = service.render(markdown);
    web_sys::console::log_1(&format!("GFM Example:\n{}", html).into());
}

/// Custom configuration example.
fn custom_config_example() {
    // Create service with custom config
    let config = MarkdownConfig {
        enable_gfm: true,
        enable_tables: true,
        sanitize_html: true,
        css_class_prefix: "custom-".to_string(),
        ..Default::default()
    };

    let service = MarkdownService::new(config);

    let markdown = r#"
# Custom Configuration

This example uses a custom CSS class prefix.

| Column 1 | Column 2 |
|----------|----------|
| Data 1   | Data 2   |
"#;

    let html = service.render(markdown);
    web_sys::console::log_1(&format!("Custom Config Example:\n{}", html).into());
}
