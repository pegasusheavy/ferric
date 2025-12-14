//! Markdown pipe for templates.

use crate::{MarkdownService, MarkdownConfig};
use ferric_core::pipes::{Pipe, PipeArgs};
use ferric_core::di::{Injectable, Injector};

/// Pipe for rendering markdown in templates.
///
/// ## Usage
///
/// ```html
/// <!-- Basic usage -->
/// <div [innerHTML]="content | markdown"></div>
///
/// <!-- Inline rendering -->
/// <span [innerHTML]="text | markdown:'inline'"></span>
/// ```
pub struct MarkdownPipe {
    config: MarkdownConfig,
}

impl MarkdownPipe {
    pub fn new(config: MarkdownConfig) -> Self {
        Self { config }
    }

    fn service(&self) -> MarkdownService {
        MarkdownService::new(self.config.clone())
    }
}

impl Pipe for MarkdownPipe {
    fn name(&self) -> &'static str {
        "markdown"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let service = self.service();

        if args.is_empty() {
            // Default rendering
            service.render(value)
        } else {
            // Check for 'inline' mode
            let mode = args.get(0).map(|s| s).unwrap_or("");

            match mode {
                "inline" => {
                    let renderer = crate::renderer::MarkdownRenderer::new(&self.config);
                    renderer.render_inline(value)
                }
                "wrapped" => {
                    let renderer = crate::renderer::MarkdownRenderer::new(&self.config);
                    renderer.render_wrapped(value)
                }
                _ => service.render(value),
            }
        }
    }
}

impl Injectable for MarkdownPipe {
    fn create(injector: &Injector) -> Self {
        let config = injector
            .resolve::<MarkdownConfig>()
            .map(|c| (*c).clone())
            .unwrap_or_default();

        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferric_core::di::Injector;

    #[test]
    fn test_markdown_pipe() {
        let injector = Injector::root();
        injector.register_singleton::<MarkdownService>();
        injector.register_singleton::<MarkdownPipe>();

        let pipe = injector.resolve::<MarkdownPipe>().unwrap();
        let args = PipeArgs::new();
        let result = pipe.transform("# Hello", &args);

        assert!(result.contains("<h1>"));
        assert!(result.contains("Hello"));
    }
}

