//! Response types for Armature-style SSR

use hyper::{Body, Response, StatusCode};
use hyper::header::{self, HeaderValue};

/// SSR response
#[derive(Clone)]
pub struct SsrResponse {
    html: String,
    status: StatusCode,
}

impl SsrResponse {
    /// Create a new SSR response
    pub fn new(html: String) -> Self {
        Self {
            html,
            status: StatusCode::OK,
        }
    }

    /// Get the HTML content
    pub fn html(&self) -> &str {
        &self.html
    }

    /// Set the status code
    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Convert to a Hyper response
    pub fn into_hyper_response(self) -> Response<Body> {
        let mut response = Response::new(Body::from(self.html));
        *response.status_mut() = self.status;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response
    }
}
