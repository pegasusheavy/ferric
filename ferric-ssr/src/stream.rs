//! Streaming HTML responses.

use crate::error::SsrResult;
use crate::render::{RenderContext, Renderable};
use futures::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// A stream of HTML chunks for progressive rendering.
    pub struct HtmlStream {
        chunks: Vec<String>,
        index: usize,
    }
}

impl HtmlStream {
    /// Create a new HTML stream from chunks.
    pub fn new(chunks: Vec<String>) -> Self {
        Self { chunks, index: 0 }
    }

    /// Create a stream with a shell header, content, and footer.
    pub fn with_shell(header: String, content: String, footer: String) -> Self {
        Self::new(vec![header, content, footer])
    }
}

impl Stream for HtmlStream {
    type Item = SsrResult<String>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.index < this.chunks.len() {
            let chunk = this.chunks[*this.index].clone();
            *this.index += 1;
            Poll::Ready(Some(Ok(chunk)))
        } else {
            Poll::Ready(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.chunks.len() - self.index;
        (remaining, Some(remaining))
    }
}

/// Builder for constructing streaming HTML responses.
#[derive(Debug, Default)]
pub struct StreamBuilder {
    chunks: Vec<String>,
}

impl StreamBuilder {
    /// Create a new stream builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an HTML chunk.
    pub fn chunk(mut self, html: impl Into<String>) -> Self {
        self.chunks.push(html.into());
        self
    }

    /// Add the doctype and opening HTML tags.
    pub fn start_document(self, lang: &str, title: &str) -> Self {
        self.chunk(format!(
            r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body>"#,
            lang,
            html_escape::encode_text(title)
        ))
    }

    /// Add the closing tags.
    pub fn end_document(self) -> Self {
        self.chunk("</body></html>")
    }

    /// Add a rendered component.
    pub fn component<T: Renderable>(self, component: &T) -> SsrResult<Self> {
        let mut ctx = RenderContext::new();
        let html = component.render(&mut ctx)?;
        Ok(self.chunk(html))
    }

    /// Build the stream.
    pub fn build(self) -> HtmlStream {
        HtmlStream::new(self.chunks)
    }
}

/// Render a component to a streaming response.
pub fn render_to_stream<T: Renderable>(
    component: &T,
    shell_header: &str,
    shell_footer: &str,
) -> SsrResult<HtmlStream> {
    let mut ctx = RenderContext::new();
    let content = component.render(&mut ctx)?;

    Ok(HtmlStream::with_shell(
        shell_header.to_string(),
        content,
        shell_footer.to_string(),
    ))
}

/// Create a streaming response with suspense boundaries.
///
/// This allows parts of the page to be sent immediately while
/// slower parts are rendered and streamed later.
pub struct SuspenseStream {
    /// The initial HTML to send immediately.
    pub initial: String,
    /// Deferred chunks with their placeholder IDs.
    pub deferred: Vec<DeferredChunk>,
}

/// A deferred chunk that will replace a placeholder.
pub struct DeferredChunk {
    /// The placeholder ID in the initial HTML.
    pub placeholder_id: String,
    /// The HTML content to insert.
    pub content: String,
}

impl SuspenseStream {
    /// Create a new suspense stream.
    pub fn new(initial: String) -> Self {
        Self {
            initial,
            deferred: Vec::new(),
        }
    }

    /// Add a deferred chunk.
    pub fn defer(mut self, placeholder_id: impl Into<String>, content: impl Into<String>) -> Self {
        self.deferred.push(DeferredChunk {
            placeholder_id: placeholder_id.into(),
            content: content.into(),
        });
        self
    }

    /// Convert to an HtmlStream.
    pub fn into_stream(self) -> HtmlStream {
        let mut chunks = vec![self.initial];

        // Add script tags that will replace placeholders
        for deferred in self.deferred {
            let script = format!(
                r#"<script>
(function() {{
    var placeholder = document.getElementById('{}');
    if (placeholder) {{
        var template = document.createElement('template');
        template.innerHTML = '{}';
        placeholder.replaceWith(template.content);
    }}
}})();
</script>"#,
                deferred.placeholder_id,
                deferred.content.replace('\'', "\\'").replace('\n', "\\n")
            );
            chunks.push(script);
        }

        HtmlStream::new(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_html_stream() {
        let stream = HtmlStream::new(vec![
            "<html>".to_string(),
            "<body>".to_string(),
            "</body></html>".to_string(),
        ]);

        let chunks: Vec<_> = stream.collect().await;
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].as_ref().unwrap(), "<html>");
        assert_eq!(chunks[2].as_ref().unwrap(), "</body></html>");
    }

    #[tokio::test]
    async fn test_stream_builder() {
        let stream = StreamBuilder::new()
            .start_document("en", "Test")
            .chunk("<div>Content</div>")
            .end_document()
            .build();

        let chunks: Vec<_> = stream.collect().await;
        assert_eq!(chunks.len(), 3);
    }
}

