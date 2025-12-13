//! Hyper-based HTTP server for SSR.

use crate::config::SsrConfig;
use crate::error::SsrResult;
use crate::response::SsrResponse;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Type alias for route handlers.
pub type RouteHandler = Arc<
    dyn Fn(Request<Incoming>) -> Pin<Box<dyn Future<Output = SsrResult<SsrResponse>> + Send>>
        + Send
        + Sync,
>;

/// SSR server built on Hyper.
pub struct SsrServer {
    config: SsrConfig,
    routes: HashMap<(Method, String), RouteHandler>,
    fallback: Option<RouteHandler>,
}

impl SsrServer {
    /// Create a new SSR server with the given configuration.
    pub fn new(config: SsrConfig) -> Self {
        Self {
            config,
            routes: HashMap::new(),
            fallback: None,
        }
    }

    /// Create a new SSR server with default configuration.
    pub fn default_config() -> Self {
        Self::new(SsrConfig::default())
    }

    /// Add a GET route.
    pub fn get<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = SsrResult<SsrResponse>> + Send + 'static,
    {
        self.routes.insert(
            (Method::GET, path.to_string()),
            Arc::new(move |req| Box::pin(handler(req))),
        );
        self
    }

    /// Add a POST route.
    pub fn post<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = SsrResult<SsrResponse>> + Send + 'static,
    {
        self.routes.insert(
            (Method::POST, path.to_string()),
            Arc::new(move |req| Box::pin(handler(req))),
        );
        self
    }

    /// Add a route for any method.
    pub fn route<F, Fut>(mut self, method: Method, path: &str, handler: F) -> Self
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = SsrResult<SsrResponse>> + Send + 'static,
    {
        self.routes.insert(
            (method, path.to_string()),
            Arc::new(move |req| Box::pin(handler(req))),
        );
        self
    }

    /// Set a fallback handler for unmatched routes.
    pub fn fallback<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = SsrResult<SsrResponse>> + Send + 'static,
    {
        self.fallback = Some(Arc::new(move |req| Box::pin(handler(req))));
        self
    }

    /// Get the server's bind address.
    pub fn addr(&self) -> SocketAddr {
        self.config.bind_addr
    }

    /// Run the server.
    pub async fn run(self) -> SsrResult<()> {
        let addr = self.config.bind_addr;
        let listener = TcpListener::bind(addr).await?;

        println!("SSR server listening on http://{}", addr);

        let service = SsrService::new(self.routes, self.fallback, self.config);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let service = service.clone();

            tokio::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    /// Run the server and return a handle for graceful shutdown.
    pub async fn run_with_shutdown(
        self,
        shutdown: impl Future<Output = ()> + Send + 'static,
    ) -> SsrResult<()> {
        let addr = self.config.bind_addr;
        let listener = TcpListener::bind(addr).await?;

        println!("SSR server listening on http://{}", addr);

        let service = SsrService::new(self.routes, self.fallback, self.config);

        tokio::select! {
            _ = async {
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let io = TokioIo::new(stream);
                            let service = service.clone();

                            tokio::spawn(async move {
                                if let Err(err) = http1::Builder::new()
                                    .serve_connection(io, service)
                                    .await
                                {
                                    eprintln!("Error serving connection: {:?}", err);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Error accepting connection: {:?}", e);
                        }
                    }
                }
            } => {}
            _ = shutdown => {
                println!("Shutting down SSR server...");
            }
        }

        Ok(())
    }
}

/// The inner Hyper service data.
struct SsrServiceInner {
    routes: HashMap<(Method, String), RouteHandler>,
    fallback: Option<RouteHandler>,
    config: SsrConfig,
}

/// The Hyper service implementation.
pub struct SsrService {
    inner: Arc<SsrServiceInner>,
}

impl SsrService {
    fn new(
        routes: HashMap<(Method, String), RouteHandler>,
        fallback: Option<RouteHandler>,
        config: SsrConfig,
    ) -> Self {
        Self {
            inner: Arc::new(SsrServiceInner {
                routes,
                fallback,
                config,
            }),
        }
    }
}

impl Clone for SsrService {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Service<Request<Incoming>> for SsrService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        // Find matching route
        let handler = self
            .inner
            .routes
            .get(&(method.clone(), path.clone()))
            .cloned()
            .or_else(|| self.inner.fallback.clone());

        let dev_mode = self.inner.config.dev_mode;

        Box::pin(async move {
            let response = match handler {
                Some(handler) => match handler(req).await {
                    Ok(resp) => resp.build(),
                    Err(err) => {
                        eprintln!("Handler error: {:?}", err);
                        if dev_mode {
                            SsrResponse::server_error(&err.to_string()).build()
                        } else {
                            SsrResponse::server_error("Internal Server Error").build()
                        }
                    }
                },
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "text/html")
                    .body(Full::new(Bytes::from("<h1>404 - Not Found</h1>")))
                    .unwrap(),
            };

            Ok(response)
        })
    }
}

/// Helper to extract query parameters from a request.
pub fn query_params(req: &Request<Incoming>) -> HashMap<String, String> {
    req.uri()
        .query()
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to extract path from a request.
pub fn path(req: &Request<Incoming>) -> &str {
    req.uri().path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let server = SsrServer::default_config()
            .get("/", |_req| async { Ok(SsrResponse::html("<h1>Home</h1>")) })
            .get("/about", |_req| async {
                Ok(SsrResponse::html("<h1>About</h1>"))
            });

        assert_eq!(server.routes.len(), 2);
    }

    #[test]
    fn test_config() {
        let config = SsrConfig::new().bind("0.0.0.0:8080").dev(true);

        let server = SsrServer::new(config);
        assert_eq!(server.addr().port(), 8080);
    }
}
