//! reqwest implementation for native targets

use crate::{Body, Error, Headers, Request, Response, Result};
use std::time::Duration;

/// Execute a request using reqwest
pub async fn execute(request: Request) -> Result<Response> {
    let client = reqwest::Client::new();
    execute_with_client(&client, request).await
}

/// Execute a request using an existing reqwest client
pub async fn execute_with_client(client: &reqwest::Client, request: Request) -> Result<Response> {
    // Build the reqwest request
    let method: reqwest::Method = match request.method {
        crate::Method::Get => reqwest::Method::GET,
        crate::Method::Post => reqwest::Method::POST,
        crate::Method::Put => reqwest::Method::PUT,
        crate::Method::Delete => reqwest::Method::DELETE,
        crate::Method::Patch => reqwest::Method::PATCH,
        crate::Method::Head => reqwest::Method::HEAD,
        crate::Method::Options => reqwest::Method::OPTIONS,
        crate::Method::Connect => reqwest::Method::CONNECT,
        crate::Method::Trace => reqwest::Method::TRACE,
    };

    let mut builder = client.request(method, request.url.as_str());

    // Set headers
    for (key, value) in request.headers.iter() {
        builder = builder.header(key, value);
    }

    // Set timeout
    if let Some(timeout_ms) = request.timeout_ms {
        builder = builder.timeout(Duration::from_millis(timeout_ms as u64));
    }

    // Set body
    match request.body {
        Body::Empty => {}
        Body::Bytes(bytes) => {
            builder = builder.body(bytes);
        }
        Body::Text(text) => {
            builder = builder.body(text);
        }
        #[cfg(feature = "json")]
        Body::Json(json) => {
            builder = builder
                .header("content-type", "application/json")
                .body(json);
        }
        Body::Form(fields) => {
            builder = builder.form(&fields);
        }
    }

    // Execute the request
    let response = builder.send().await?;

    // Extract response data
    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    // Extract headers
    let mut headers = Headers::new();
    for (key, value) in response.headers().iter() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.as_str(), v);
        }
    }

    // Get final URL
    let final_url = response.url().to_string();

    // Read body
    let body = response.bytes().await?.to_vec();

    Ok(Response::new(status, status_text, headers, body, final_url))
}

/// Create a configured reqwest client
pub fn create_client(timeout_ms: Option<u32>) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();

    if let Some(timeout) = timeout_ms {
        builder = builder.timeout(Duration::from_millis(timeout as u64));
    }

    builder.build().map_err(|e| Error::Request(e.to_string()))
}

