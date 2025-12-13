//! Code block component with syntax highlighting styling

/// Render a code block with Rust-like syntax highlighting
pub fn code_block(code: &str, language: &str) -> String {
    let escaped = html_escape(code);
    format!(
        r#"<div class="relative group">
            <div class="absolute right-3 top-3 opacity-0 group-hover:opacity-100 transition-opacity">
                <button class="px-2 py-1 text-xs text-slate-400 hover:text-white bg-slate-700 hover:bg-slate-600 rounded transition-colors" onclick="navigator.clipboard.writeText(this.closest('.group').querySelector('code').textContent)">
                    Copy
                </button>
            </div>
            <pre class="overflow-x-auto rounded-xl bg-slate-900 p-4 text-sm"><code class="language-{language} text-slate-300">{escaped}</code></pre>
        </div>"#,
        language = language,
        escaped = escaped
    )
}

/// Render an inline code span
pub fn inline_code(code: &str) -> String {
    format!(
        r#"<code class="px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-orange-600 dark:text-orange-400 text-sm font-mono">{}</code>"#,
        html_escape(code)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}


/// Render a code block with Rust-like syntax highlighting
pub fn code_block(code: &str, language: &str) -> String {
    let escaped = html_escape(code);
    format!(
        r#"<div class="relative group">
            <div class="absolute right-3 top-3 opacity-0 group-hover:opacity-100 transition-opacity">
                <button class="px-2 py-1 text-xs text-slate-400 hover:text-white bg-slate-700 hover:bg-slate-600 rounded transition-colors" onclick="navigator.clipboard.writeText(this.closest('.group').querySelector('code').textContent)">
                    Copy
                </button>
            </div>
            <pre class="overflow-x-auto rounded-xl bg-slate-900 p-4 text-sm"><code class="language-{language} text-slate-300">{escaped}</code></pre>
        </div>"#,
        language = language,
        escaped = escaped
    )
}

/// Render an inline code span
pub fn inline_code(code: &str) -> String {
    format!(
        r#"<code class="px-1.5 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-orange-600 dark:text-orange-400 text-sm font-mono">{}</code>"#,
        html_escape(code)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

