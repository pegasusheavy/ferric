//! Static file server with SPA support
//!
//! Inspired by Armature's controller-based routing pattern.

use super::{DevServerConfig, ReloadNotifier};
use anyhow::Result;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tower_http::services::ServeDir;

/// Shared server state
#[derive(Clone)]
pub struct ServerState {
    pub config: DevServerConfig,
    pub reload_notifier: Option<ReloadNotifier>,
}

/// Static file server with hot reload injection
pub struct FileServer {
    state: Arc<ServerState>,
}

impl FileServer {
    /// Create a new file server
    pub fn new(config: DevServerConfig, reload_notifier: Option<ReloadNotifier>) -> Self {
        Self {
            state: Arc::new(ServerState {
                config,
                reload_notifier,
            }),
        }
    }

    /// Start serving files
    pub async fn serve(self, addr: SocketAddr) -> Result<()> {
        let state = self.state.clone();
        let static_dir = state.config.static_dir.clone();

        // Build router with Armature-inspired structure
        let app = Router::new()
            // Health check endpoint
            .route("/__health", get(health_check))
            // Hot reload script endpoint
            .route("/__hot-reload.js", get(hot_reload_script))
            // Fallback to static files with SPA support
            .fallback_service(
                ServeDir::new(&static_dir)
                    .fallback(get(spa_fallback))
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                inject_hot_reload,
            ))
            .layer(middleware::from_fn(logging_middleware))
            .with_state(state);

        // Start server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Hot reload script handler
async fn hot_reload_script(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let port = state.config.port + 1;
    let host = &state.config.host;

    let script = format!(
        r#"
(function() {{
    const ws = new WebSocket('ws://{}:{}');

    ws.onopen = function() {{
        console.log('[Ferric] Hot reload connected');
    }};

    ws.onmessage = function(event) {{
        const data = JSON.parse(event.data);
        if (data.type === 'reload') {{
            console.log('[Ferric] Reloading...', data.path);

            // Handle different file types
            if (data.path && data.path.endsWith('.css')) {{
                // Hot reload CSS without full page refresh
                const links = document.querySelectorAll('link[rel="stylesheet"]');
                links.forEach(link => {{
                    const href = link.getAttribute('href');
                    if (href) {{
                        link.setAttribute('href', href.split('?')[0] + '?t=' + Date.now());
                    }}
                }});
            }} else {{
                // Full page reload for other files
                location.reload();
            }}
        }}
    }};

    ws.onclose = function() {{
        console.log('[Ferric] Hot reload disconnected, reconnecting...');
        setTimeout(() => location.reload(), 1000);
    }};

    ws.onerror = function(err) {{
        console.error('[Ferric] Hot reload error:', err);
    }};
}})();
"#,
        host, port
    );

    (
        [(header::CONTENT_TYPE, "application/javascript")],
        script,
    )
}

/// SPA fallback - serve index.html for unmatched routes
async fn spa_fallback(
    State(state): State<Arc<ServerState>>,
    uri: Uri,
) -> impl IntoResponse {
    let index_path = state.config.static_dir.join("index.html");

    match fs::read_to_string(&index_path).await {
        Ok(content) => {
            // Inject hot reload script if enabled
            let content = if state.config.live_reload {
                inject_script(&content)
            } else {
                content
            };
            Html(content).into_response()
        }
        Err(_) => {
            // Return 404 page
            let not_found = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>404 - Not Found</title>
    <style>
        body {{
            font-family: system-ui, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: #f8fafc;
            color: #334155;
        }}
        .container {{ text-align: center; }}
        h1 {{ font-size: 4rem; margin: 0; color: #f97316; }}
        p {{ color: #64748b; }}
        code {{ background: #e2e8f0; padding: 0.25rem 0.5rem; border-radius: 0.25rem; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>404</h1>
        <p>Could not find <code>{}</code></p>
        <p>Make sure your app is built: <code>ferric build</code></p>
    </div>
</body>
</html>"#,
                uri.path()
            );
            (StatusCode::NOT_FOUND, Html(not_found)).into_response()
        }
    }
}

/// Middleware to inject hot reload script into HTML responses
async fn inject_hot_reload(
    State(state): State<Arc<ServerState>>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let response = next.run(request).await;

    // Only inject into HTML responses
    if !state.config.live_reload {
        return response;
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") {
        return response;
    }

    // For HTML responses, the script is injected via the SPA fallback
    // This middleware handles static HTML files
    response
}

/// Logging middleware
async fn logging_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    use console::style;

    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let status = response.status();
    let duration = start.elapsed();

    // Skip logging for hot reload endpoints
    if uri.path().starts_with("/__") {
        return response;
    }

    let status_style = if status.is_success() {
        style(status.as_u16()).green()
    } else if status.is_redirection() {
        style(status.as_u16()).yellow()
    } else {
        style(status.as_u16()).red()
    };

    println!(
        "  {} {} {} {}",
        status_style,
        style(method).dim(),
        uri.path(),
        style(format!("{:?}", duration)).dim()
    );

    response
}

/// Inject hot reload script into HTML content
fn inject_script(html: &str) -> String {
    if html.contains("/__hot-reload.js") {
        return html.to_string();
    }

    let script = r#"<script src="/__hot-reload.js"></script>"#;

    if let Some(pos) = html.rfind("</body>") {
        let mut result = html.to_string();
        result.insert_str(pos, &format!("\n    {}\n  ", script));
        result
    } else if let Some(pos) = html.rfind("</html>") {
        let mut result = html.to_string();
        result.insert_str(pos, &format!("\n  {}\n", script));
        result
    } else {
        format!("{}\n{}", html, script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_script() {
        let html = "<html><body><h1>Test</h1></body></html>";
        let result = inject_script(html);
        assert!(result.contains("/__hot-reload.js"));
        assert!(result.contains("</body>"));
    }

    #[test]
    fn test_inject_script_no_body() {
        let html = "<html><h1>Test</h1></html>";
        let result = inject_script(html);
        assert!(result.contains("/__hot-reload.js"));
    }
}

//!
//! Inspired by Armature's controller-based routing pattern.

use super::{DevServerConfig, ReloadNotifier};
use anyhow::Result;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tower_http::services::ServeDir;

/// Shared server state
#[derive(Clone)]
pub struct ServerState {
    pub config: DevServerConfig,
    pub reload_notifier: Option<ReloadNotifier>,
}

/// Static file server with hot reload injection
pub struct FileServer {
    state: Arc<ServerState>,
}

impl FileServer {
    /// Create a new file server
    pub fn new(config: DevServerConfig, reload_notifier: Option<ReloadNotifier>) -> Self {
        Self {
            state: Arc::new(ServerState {
                config,
                reload_notifier,
            }),
        }
    }

    /// Start serving files
    pub async fn serve(self, addr: SocketAddr) -> Result<()> {
        let state = self.state.clone();
        let static_dir = state.config.static_dir.clone();

        // Build router with Armature-inspired structure
        let app = Router::new()
            // Health check endpoint
            .route("/__health", get(health_check))
            // Hot reload script endpoint
            .route("/__hot-reload.js", get(hot_reload_script))
            // Fallback to static files with SPA support
            .fallback_service(
                ServeDir::new(&static_dir)
                    .fallback(get(spa_fallback))
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                inject_hot_reload,
            ))
            .layer(middleware::from_fn(logging_middleware))
            .with_state(state);

        // Start server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Hot reload script handler
async fn hot_reload_script(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let port = state.config.port + 1;
    let host = &state.config.host;

    let script = format!(
        r#"
(function() {{
    const ws = new WebSocket('ws://{}:{}');

    ws.onopen = function() {{
        console.log('[Ferric] Hot reload connected');
    }};

    ws.onmessage = function(event) {{
        const data = JSON.parse(event.data);
        if (data.type === 'reload') {{
            console.log('[Ferric] Reloading...', data.path);

            // Handle different file types
            if (data.path && data.path.endsWith('.css')) {{
                // Hot reload CSS without full page refresh
                const links = document.querySelectorAll('link[rel="stylesheet"]');
                links.forEach(link => {{
                    const href = link.getAttribute('href');
                    if (href) {{
                        link.setAttribute('href', href.split('?')[0] + '?t=' + Date.now());
                    }}
                }});
            }} else {{
                // Full page reload for other files
                location.reload();
            }}
        }}
    }};

    ws.onclose = function() {{
        console.log('[Ferric] Hot reload disconnected, reconnecting...');
        setTimeout(() => location.reload(), 1000);
    }};

    ws.onerror = function(err) {{
        console.error('[Ferric] Hot reload error:', err);
    }};
}})();
"#,
        host, port
    );

    (
        [(header::CONTENT_TYPE, "application/javascript")],
        script,
    )
}

/// SPA fallback - serve index.html for unmatched routes
async fn spa_fallback(
    State(state): State<Arc<ServerState>>,
    uri: Uri,
) -> impl IntoResponse {
    let index_path = state.config.static_dir.join("index.html");

    match fs::read_to_string(&index_path).await {
        Ok(content) => {
            // Inject hot reload script if enabled
            let content = if state.config.live_reload {
                inject_script(&content)
            } else {
                content
            };
            Html(content).into_response()
        }
        Err(_) => {
            // Return 404 page
            let not_found = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>404 - Not Found</title>
    <style>
        body {{
            font-family: system-ui, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: #f8fafc;
            color: #334155;
        }}
        .container {{ text-align: center; }}
        h1 {{ font-size: 4rem; margin: 0; color: #f97316; }}
        p {{ color: #64748b; }}
        code {{ background: #e2e8f0; padding: 0.25rem 0.5rem; border-radius: 0.25rem; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>404</h1>
        <p>Could not find <code>{}</code></p>
        <p>Make sure your app is built: <code>ferric build</code></p>
    </div>
</body>
</html>"#,
                uri.path()
            );
            (StatusCode::NOT_FOUND, Html(not_found)).into_response()
        }
    }
}

/// Middleware to inject hot reload script into HTML responses
async fn inject_hot_reload(
    State(state): State<Arc<ServerState>>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let response = next.run(request).await;

    // Only inject into HTML responses
    if !state.config.live_reload {
        return response;
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") {
        return response;
    }

    // For HTML responses, the script is injected via the SPA fallback
    // This middleware handles static HTML files
    response
}

/// Logging middleware
async fn logging_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    use console::style;

    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let status = response.status();
    let duration = start.elapsed();

    // Skip logging for hot reload endpoints
    if uri.path().starts_with("/__") {
        return response;
    }

    let status_style = if status.is_success() {
        style(status.as_u16()).green()
    } else if status.is_redirection() {
        style(status.as_u16()).yellow()
    } else {
        style(status.as_u16()).red()
    };

    println!(
        "  {} {} {} {}",
        status_style,
        style(method).dim(),
        uri.path(),
        style(format!("{:?}", duration)).dim()
    );

    response
}

/// Inject hot reload script into HTML content
fn inject_script(html: &str) -> String {
    if html.contains("/__hot-reload.js") {
        return html.to_string();
    }

    let script = r#"<script src="/__hot-reload.js"></script>"#;

    if let Some(pos) = html.rfind("</body>") {
        let mut result = html.to_string();
        result.insert_str(pos, &format!("\n    {}\n  ", script));
        result
    } else if let Some(pos) = html.rfind("</html>") {
        let mut result = html.to_string();
        result.insert_str(pos, &format!("\n  {}\n", script));
        result
    } else {
        format!("{}\n{}", html, script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_script() {
        let html = "<html><body><h1>Test</h1></body></html>";
        let result = inject_script(html);
        assert!(result.contains("/__hot-reload.js"));
        assert!(result.contains("</body>"));
    }

    #[test]
    fn test_inject_script_no_body() {
        let html = "<html><h1>Test</h1></html>";
        let result = inject_script(html);
        assert!(result.contains("/__hot-reload.js"));
    }
}

