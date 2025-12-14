//! Development server module using Armature framework
//!
//! Provides a static file server with hot reload for Ferric development.
//! Built on the Armature HTTP framework (https://github.com/pegasusheavy/armature).

use crate::config::FerricConfig;
use anyhow::Result;
use armature::{
    HmrConfig, HmrManager, HttpRequest, HttpResponse, StaticAssetServer,
    StaticAssetsConfig,
};
use console::style;
use std::path::PathBuf;
use std::sync::Arc;

/// Development server configuration
#[derive(Debug, Clone)]
pub struct DevServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Host to bind to
    pub host: String,
    /// Directory to serve static files from
    pub static_dir: PathBuf,
    /// Enable live reload
    pub live_reload: bool,
    /// WebSocket port for hot reload
    pub ws_port: u16,
    /// Directories to watch for changes
    pub watch_dirs: Vec<PathBuf>,
    /// File extensions to watch
    pub watch_extensions: Vec<String>,
    /// Open browser on start
    pub open_browser: bool,
    /// SSR mode
    pub ssr: bool,
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            static_dir: PathBuf::from("dist"),
            live_reload: true,
            ws_port: 3001,
            watch_dirs: vec![
                PathBuf::from("src"),
                PathBuf::from("templates"),
                PathBuf::from("assets"),
            ],
            watch_extensions: vec![
                "rs".to_string(),
                "html".to_string(),
                "css".to_string(),
                "scss".to_string(),
                "js".to_string(),
                "wasm".to_string(),
            ],
            open_browser: false,
            ssr: false,
        }
    }
}

impl DevServerConfig {
    /// Create from ferric.toml configuration
    pub fn from_ferric_config(config: &FerricConfig) -> Self {
        Self {
            port: config.serve.port,
            host: config.serve.host.clone(),
            static_dir: PathBuf::from(&config.build.out_dir),
            live_reload: config.serve.live_reload,
            ws_port: config.serve.port + 1,
            watch_dirs: vec![
                PathBuf::from(&config.build.src_dir),
                PathBuf::from("templates"),
                PathBuf::from("assets"),
            ],
            watch_extensions: vec![
                "rs".to_string(),
                "html".to_string(),
                "css".to_string(),
                "scss".to_string(),
            ],
            open_browser: config.serve.open,
            ssr: config.ssr.enabled,
        }
    }
}

/// Start the Armature-based development server
pub async fn start_dev_server(config: DevServerConfig) -> Result<()> {
    println!(
        "\n  {}  {} v{}",
        style("⚡").cyan(),
        style("Ferric Dev Server").bold(),
        env!("CARGO_PKG_VERSION")
    );
    println!("  {}  Powered by Armature", style("🦾").dim());
    println!();

    // Ensure static directory exists
    std::fs::create_dir_all(&config.static_dir)?;

    // Configure static asset server (development mode - no caching)
    let static_config = StaticAssetsConfig::new(&config.static_dir)
        .development()
        .with_fallback("index.html")
        .with_cors(true);

    let static_server = StaticAssetServer::new(static_config)
        .map_err(|e| anyhow::anyhow!("Failed to create static server: {}", e))?;

    // Setup HMR if enabled
    let hmr_manager = if config.live_reload {
        let mut hmr_config = HmrConfig::new()
            .websocket_port(config.ws_port)
            .verbose(false);

        // Add watch paths
        for path in &config.watch_dirs {
            hmr_config = hmr_config.watch_path(path.clone());
        }

        // Also watch the static directory
        hmr_config = hmr_config.watch_path(config.static_dir.clone());

        // Add extensions
        for ext in &config.watch_extensions {
            hmr_config = hmr_config.watch_extension(ext.clone());
        }

        let manager = HmrManager::new(hmr_config);
        manager.start_watching().await.map_err(|e| {
            anyhow::anyhow!("Failed to start HMR: {}", e)
        })?;

        println!(
            "  {}  Hot reload enabled on ws://{}:{}",
            style("↻").yellow(),
            config.host,
            config.ws_port
        );

        Some(Arc::new(manager))
    } else {
        None
    };

    println!(
        "  {}  Serving static files from {}",
        style("📁").blue(),
        style(config.static_dir.display()).dim()
    );

    let addr = format!("{}:{}", config.host, config.port);

    println!();
    println!(
        "  {}  Server running at {}",
        style("✓").green().bold(),
        style(format!("http://{}", addr)).cyan().underlined()
    );
    println!();
    println!("  Press {} to stop", style("Ctrl+C").dim());
    println!();

    // Open browser if configured
    if config.open_browser {
        let url = format!("http://{}", addr);
        let _ = open::that(&url);
    }

    // Start the WebSocket server for HMR in background
    if let Some(ref hmr) = hmr_manager {
        let hmr_clone = hmr.clone();
        let ws_host = config.host.clone();
        let ws_port = config.ws_port;
        tokio::spawn(async move {
            if let Err(e) = run_hmr_websocket_server(ws_host, ws_port, hmr_clone).await {
                eprintln!("HMR WebSocket error: {}", e);
            }
        });
    }

    // Run the HTTP server
    run_http_server(config.port, static_server, hmr_manager).await
}

/// Run the main HTTP server
async fn run_http_server(
    port: u16,
    static_server: StaticAssetServer,
    hmr_manager: Option<Arc<HmrManager>>,
) -> Result<()> {
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::body::Incoming as IncomingBody;
    use hyper::Request;
    use hyper_util::rt::TokioIo;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    let static_server = Arc::new(static_server);

    loop {
        let (stream, client_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let static_server = static_server.clone();
        let hmr_manager = hmr_manager.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req: Request<IncomingBody>| {
                let static_server = static_server.clone();
                let hmr_manager = hmr_manager.clone();
                async move {
                    handle_request(req, static_server, hmr_manager).await
                }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                // Ignore common connection reset errors
                let err_str = err.to_string();
                if !err_str.contains("connection reset") && !err_str.contains("broken pipe") {
                    eprintln!("Error serving connection from {}: {}", client_addr, err);
                }
            }
        });
    }
}

/// Handle an incoming HTTP request
async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    static_server: Arc<StaticAssetServer>,
    hmr_manager: Option<Arc<HmrManager>>,
) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, hyper::Error> {
    use bytes::Bytes;
    use console::style;
    use http_body_util::Full;
    use std::time::Instant;

    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Handle health check
    if path == "/__health" {
        return Ok(hyper::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(r#"{"status":"ok","server":"ferric-dev"}"#)))
            .unwrap());
    }

    // Create Armature HttpRequest
    let armature_req = HttpRequest::new(method.clone(), path.clone());

    // Serve static file
    let response = match static_server.serve(&armature_req).await {
        Ok(mut resp) => {
            // Inject HMR script into HTML responses
            if let Some(ref hmr) = hmr_manager
                && let Some(content_type) = resp.headers.get("Content-Type")
                    && content_type.contains("text/html") {
                        let html = String::from_utf8_lossy(&resp.body).to_string();
                        let injected = armature::inject_hmr_script(html, hmr).await;
                        resp.body = injected.into_bytes();
                    }
            resp
        }
        Err(e) => {
            // Create error response
            let (status, body) = match &e {
                armature::Error::NotFound(_) => (404, format!("Not Found: {}", path)),
                armature::Error::Forbidden(_) => (403, "Forbidden".to_string()),
                _ => (500, format!("Internal Server Error: {}", e)),
            };

            let mut resp = HttpResponse::new(status);
            resp.headers
                .insert("Content-Type".to_string(), "text/plain".to_string());
            resp.body = body.into_bytes();
            resp
        }
    };

    let duration = start.elapsed();
    let status = response.status;

    // Log request (skip assets and health checks)
    if !path.starts_with("/__") && !is_asset_path(&path) {
        let status_style = if status < 400 {
            style(status).green()
        } else if status < 500 {
            style(status).yellow()
        } else {
            style(status).red()
        };

        println!(
            "  {} {} {} {}",
            status_style,
            style(&method).dim(),
            &path,
            style(format!("{:?}", duration)).dim()
        );
    }

    // Convert to hyper response
    let mut builder = hyper::Response::builder().status(status);
    for (key, value) in response.headers {
        builder = builder.header(key, value);
    }

    Ok(builder
        .body(Full::new(Bytes::from(response.body)))
        .unwrap())
}

/// Check if path is a common asset
fn is_asset_path(path: &str) -> bool {
    let asset_extensions = [
        ".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".woff", ".woff2", ".ttf",
        ".eot", ".map", ".wasm",
    ];
    asset_extensions.iter().any(|ext| path.ends_with(ext))
}

/// Run the HMR WebSocket server
async fn run_hmr_websocket_server(
    host: String,
    port: u16,
    hmr_manager: Arc<HmrManager>,
) -> Result<()> {
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let hmr = hmr_manager.clone();
        let mut event_rx = hmr.subscribe();

        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WebSocket handshake error: {}", e);
                    return;
                }
            };

            let (mut write, mut read) = ws_stream.split();

            // Send welcome message
            let welcome = serde_json::json!({
                "type": "connected",
                "message": "Ferric HMR connected"
            });
            let _ = write
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    welcome.to_string(),
                ))
                .await;

            loop {
                tokio::select! {
                    // Forward HMR events to client
                    event = event_rx.recv() => {
                        match event {
                            Ok(hmr_event) => {
                                let msg = serde_json::json!({
                                    "type": match hmr_event.extension.as_deref() {
                                        Some("css") | Some("scss") => "css-update",
                                        Some("js") | Some("ts") => "js-update",
                                        _ => "full-reload"
                                    },
                                    "path": hmr_event.path.to_string_lossy(),
                                    "kind": format!("{:?}", hmr_event.kind)
                                });
                                if write.send(tokio_tungstenite::tungstenite::Message::Text(msg.to_string())).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    // Handle client messages (ping/pong)
                    msg = read.next() => {
                        match msg {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data))) => {
                                if write.send(tokio_tungstenite::tungstenite::Message::Pong(data)).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) | None => break,
                            _ => {}
                        }
                    }
                }
            }
        });
    }
}
