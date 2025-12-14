#!/bin/bash
# Generate HTML pages from markdown documentation

DOCS_DIR="docs"
WEB_DOCS_DIR="web/docs"
TEMPLATE_START='<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{TITLE}} - Ferric Documentation</title>
    <link rel="stylesheet" href="../styles/docs.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css">
    <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
</head>
<body>
    <nav class="sidebar">
        <a href="../docs.html">← Back to Docs</a>
        <h3>Navigation</h3>
        <ul>
            <li><a href="getting-started.html">Getting Started</a></li>
            <li><a href="quick-start.html">Quick Start</a></li>
            <li><a href="components.html">Components</a></li>
            <li><a href="async-support.html">Async Support</a></li>
            <li><a href="di-architecture.html">DI Architecture</a></li>
            <li><a href="file-transfer.html">File Transfers</a></li>
            <li><a href="http-client.html">HTTP Client</a></li>
            <li><a href="macros.html">Macros</a></li>
            <li><a href="decorators.html">Decorators</a></li>
            <li><a href="benchmarking.html">Benchmarking</a></li>
        </ul>
    </nav>

    <main class="content">
        <div id="markdown-content"></div>
    </main>

    <script>
        const markdown = `{{MARKDOWN_CONTENT}}`;
        const html = marked.parse(markdown);
        document.getElementById("markdown-content").innerHTML = html;

        // Highlight code blocks
        document.querySelectorAll("pre code").forEach((block) => {
            hljs.highlightElement(block);
        });
    </script>
</body>
</html>'

mkdir -p "$WEB_DOCS_DIR"

# Function to generate HTML from markdown
generate_html() {
    local md_file=$1
    local html_file=$2
    local title=$3

    # Escape backticks and backslashes for JavaScript
    local content=$(cat "$md_file" | sed 's/\\/\\\\/g' | sed 's/`/\\`/g')

    # Generate HTML
    echo "$TEMPLATE_START" | sed "s/{{TITLE}}/$title/g" | sed "s|{{MARKDOWN_CONTENT}}|$content|g" > "$html_file"

    echo "✅ Generated: $html_file"
}

# Generate HTML pages from markdown files
generate_html "$DOCS_DIR/QUICK_START.md" "$WEB_DOCS_DIR/quick-start.html" "Quick Start"
generate_html "$DOCS_DIR/COMPONENTS_GUIDE.md" "$WEB_DOCS_DIR/components.html" "Components Guide"
generate_html "$DOCS_DIR/ASYNC_SUPPORT_GUIDE.md" "$WEB_DOCS_DIR/async-support.html" "Async Support"
generate_html "$DOCS_DIR/DI_ARCHITECTURE.md" "$WEB_DOCS_DIR/di-architecture.html" "DI Architecture"
generate_html "$DOCS_DIR/FILE_TRANSFER_GUIDE.md" "$WEB_DOCS_DIR/file-transfer.html" "File Transfers"
generate_html "$DOCS_DIR/HTTP_CLIENT_GUIDE.md" "$WEB_DOCS_DIR/http-client.html" "HTTP Client"
generate_html "$DOCS_DIR/MACROS_QUICK_REFERENCE.md" "$WEB_DOCS_DIR/macros.html" "Macros Reference"
generate_html "$DOCS_DIR/ANGULAR_DECORATORS_GUIDE.md" "$WEB_DOCS_DIR/decorators.html" "Decorators"
generate_html "$DOCS_DIR/BENCHMARKING_GUIDE.md" "$WEB_DOCS_DIR/benchmarking.html" "Benchmarking"

echo ""
echo "📚 Documentation HTML generation complete!"
echo "Total pages: $(ls -1 $WEB_DOCS_DIR/*.html 2>/dev/null | wc -l)"

