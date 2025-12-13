//! HTTP response types

use crate::{Error, Headers, Result};

/// HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code
    status_code: u16,
    /// Status text (e.g., "OK", "Not Found")
    pub status_text: String,
    /// Response headers
    pub headers: Headers,
    /// Response body as bytes
    body: Vec<u8>,
    /// The final URL (after redirects)
    pub url: String,
}

impl Response {
    /// Create a new response
    pub(crate) fn new(
        status: u16,
        status_text: String,
        headers: Headers,
        body: Vec<u8>,
        url: String,
    ) -> Self {
        Self {
            status_code: status,
            status_text,
            headers,
            body,
            url,
        }
    }

    /// Get the HTTP status code
    pub fn status(&self) -> u16 {
        self.status_code
    }

    /// Check if the response status is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// Check if the response is informational (1xx)
    pub fn is_informational(&self) -> bool {
        (100..200).contains(&self.status_code)
    }

    /// Check if the response is a redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status_code)
    }

    /// Check if the response is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status_code)
    }

    /// Check if the response is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status_code)
    }

    /// Get the response body as bytes
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    /// Consume the response and return the body as bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.body
    }

    /// Get the response body as text
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| Error::Response(format!("Invalid UTF-8: {}", e)))
    }

    /// Consume the response and return the body as text
    pub fn into_text(self) -> Result<String> {
        String::from_utf8(self.body)
            .map_err(|e| Error::Response(format!("Invalid UTF-8: {}", e)))
    }

    /// Parse the response body as JSON (requires `json` feature)
    #[cfg(feature = "json")]
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(|e| Error::Json(e.to_string()))
    }

    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Get the Content-Type header
    pub fn content_type(&self) -> Option<&String> {
        self.headers.get("content-type")
    }

    /// Get the Content-Length header
    pub fn content_length(&self) -> Option<usize> {
        self.headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// Return an error if the response status is not successful
    pub fn error_for_status(self) -> Result<Self> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(Error::Status {
                code: self.status_code,
                message: self.status_text.clone(),
            })
        }
    }

    /// Return a reference error if the response status is not successful
    pub fn error_for_status_ref(&self) -> Result<&Self> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(Error::Status {
                code: self.status_code,
                message: self.status_text.clone(),
            })
        }
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {} {}", self.status_code, self.status_text)
    }
}

