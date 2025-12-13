//! HTTP response types for SSR.

use crate::error::SsrResult;
use crate::state::SerializedState;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::collections::HashMap;

/// An HTML response builder.
#[derive(Debug)]
pub struct HtmlResponse {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl HtmlResponse {
    /// Create a new HTML response with the given body.
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            status: StatusCode::OK,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Create a 200 OK response.
    pub fn ok(body: impl Into<String>) -> Self {
        Self::new(body)
    }

    /// Create a 404 Not Found response.
    pub fn not_found(body: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Create a 500 Internal Server Error response.
    pub fn server_error(body: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Set the status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set cache control header.
    pub fn cache_control(self, value: &str) -> Self {
        self.header("Cache-Control", value)
    }

    /// Set no caching.
    pub fn no_cache(self) -> Self {
        self.cache_control("no-store, no-cache, must-revalidate")
    }

    /// Build the Hyper response.
    pub fn build(self) -> Response<Full<Bytes>> {
        let mut builder = Response::builder()
            .status(self.status)
            .header("Content-Type", "text/html; charset=utf-8");

        for (name, value) in self.headers {
            builder = builder.header(name, value);
        }

        builder
            .body(Full::new(Bytes::from(self.body)))
            .expect("Failed to build response")
    }
}

impl From<HtmlResponse> for Response<Full<Bytes>> {
    fn from(response: HtmlResponse) -> Self {
        response.build()
    }
}

/// A more general SSR response that can be HTML or JSON.
#[derive(Debug)]
pub struct SsrResponse {
    inner: ResponseInner,
}

#[derive(Debug)]
enum ResponseInner {
    Html(HtmlResponse),
    Json {
        status: StatusCode,
        body: String,
    },
    Redirect {
        location: String,
        permanent: bool,
    },
}

impl SsrResponse {
    /// Create an HTML response.
    pub fn html(body: impl Into<String>) -> Self {
        Self {
            inner: ResponseInner::Html(HtmlResponse::ok(body)),
        }
    }

    /// Create an HTML response with state for hydration.
    pub fn html_with_state(html: impl Into<String>, state: &SerializedState) -> Self {
        let html = html.into();
        let full_html = format!(
            r#"{}
<script id="__FERRIC_STATE__" type="application/json">{}</script>"#,
            html,
            state.as_json()
        );
        Self::html(full_html)
    }

    /// Create a JSON response.
    pub fn json<T: serde::Serialize>(data: &T) -> SsrResult<Self> {
        let body = serde_json::to_string(data)?;
        Ok(Self {
            inner: ResponseInner::Json {
                status: StatusCode::OK,
                body,
            },
        })
    }

    /// Create a redirect response.
    pub fn redirect(location: impl Into<String>) -> Self {
        Self {
            inner: ResponseInner::Redirect {
                location: location.into(),
                permanent: false,
            },
        }
    }

    /// Create a permanent redirect response.
    pub fn permanent_redirect(location: impl Into<String>) -> Self {
        Self {
            inner: ResponseInner::Redirect {
                location: location.into(),
                permanent: true,
            },
        }
    }

    /// Create a 404 Not Found response.
    pub fn not_found() -> Self {
        Self {
            inner: ResponseInner::Html(HtmlResponse::not_found(
                "<h1>404 - Not Found</h1>",
            )),
        }
    }

    /// Create a 500 Internal Server Error response.
    pub fn server_error(message: &str) -> Self {
        Self {
            inner: ResponseInner::Html(HtmlResponse::server_error(format!(
                "<h1>500 - Internal Server Error</h1><p>{}</p>",
                html_escape::encode_text(message)
            ))),
        }
    }

    /// Build the Hyper response.
    pub fn build(self) -> Response<Full<Bytes>> {
        match self.inner {
            ResponseInner::Html(html) => html.build(),
            ResponseInner::Json { status, body } => Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(body)))
                .expect("Failed to build response"),
            ResponseInner::Redirect {
                location,
                permanent,
            } => {
                let status = if permanent {
                    StatusCode::PERMANENT_REDIRECT
                } else {
                    StatusCode::TEMPORARY_REDIRECT
                };
                Response::builder()
                    .status(status)
                    .header("Location", location)
                    .body(Full::new(Bytes::new()))
                    .expect("Failed to build response")
            }
        }
    }
}

impl From<SsrResponse> for Response<Full<Bytes>> {
    fn from(response: SsrResponse) -> Self {
        response.build()
    }
}

