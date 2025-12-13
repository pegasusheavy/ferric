//! Fetch API implementation for WASM targets

use crate::{Credentials, Error, Headers, Request, Response, Result};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Execute a request using the Fetch API
pub async fn execute(request: Request) -> Result<Response> {
    let window = web_sys::window().ok_or_else(|| Error::Network("No window object".to_string()))?;

    // Build request options
    let opts = web_sys::RequestInit::new();
    opts.set_method(request.method.as_str());

    // Set credentials mode
    let credentials = match request.credentials {
        Credentials::Omit => web_sys::RequestCredentials::Omit,
        Credentials::SameOrigin => web_sys::RequestCredentials::SameOrigin,
        Credentials::Include => web_sys::RequestCredentials::Include,
    };
    opts.set_credentials(credentials);

    // Set headers
    let headers = web_sys::Headers::new().map_err(|e| Error::Request(format!("{:?}", e)))?;
    for (key, value) in request.headers.iter() {
        headers
            .append(key, value)
            .map_err(|e| Error::InvalidHeader(format!("{:?}", e)))?;
    }

    // Set Content-Type if body has one and not already set
    if let Some(ct) = request.body.content_type() {
        if !request.headers.contains("content-type") {
            headers
                .append("Content-Type", ct)
                .map_err(|e| Error::InvalidHeader(format!("{:?}", e)))?;
        }
    }

    opts.set_headers(&headers);

    // Set body if not empty
    if !request.body.is_empty() {
        let body_bytes = request.body.into_bytes();
        let uint8_array = js_sys::Uint8Array::from(body_bytes.as_slice());
        opts.set_body(&uint8_array);
    }

    // Create the fetch request
    let url = request.url.as_str();
    let fetch_request =
        web_sys::Request::new_with_str_and_init(url, &opts).map_err(|e| Error::Request(format!("{:?}", e)))?;

    // Execute fetch with optional timeout
    let response_value = if let Some(timeout_ms) = request.timeout_ms {
        fetch_with_timeout(&window, &fetch_request, timeout_ms).await?
    } else {
        let promise = window.fetch_with_request(&fetch_request);
        JsFuture::from(promise)
            .await
            .map_err(|e| Error::Network(format!("{:?}", e)))?
    };

    // Parse the response
    let fetch_response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| Error::Response("Invalid response object".to_string()))?;

    // Extract status
    let status = fetch_response.status();
    let status_text = fetch_response.status_text();

    // Extract headers
    let response_headers = parse_headers(&fetch_response.headers())?;

    // Extract body
    let body = read_body(&fetch_response).await?;

    // Get final URL
    let final_url = fetch_response.url();

    Ok(Response::new(
        status,
        status_text,
        response_headers,
        body,
        final_url,
    ))
}

/// Fetch with timeout using AbortController
async fn fetch_with_timeout(
    window: &web_sys::Window,
    request: &web_sys::Request,
    timeout_ms: u32,
) -> Result<JsValue> {
    let controller =
        web_sys::AbortController::new().map_err(|e| Error::Request(format!("{:?}", e)))?;

    let signal = controller.signal();

    // Clone request with abort signal
    let opts = web_sys::RequestInit::new();
    opts.set_signal(Some(&signal));

    let request_with_signal = web_sys::Request::new_with_request_and_init(request, &opts)
        .map_err(|e| Error::Request(format!("{:?}", e)))?;

    // Create timeout promise
    let timeout_promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let controller_clone = controller.clone();
        let closure = Closure::once(Box::new(move || {
            controller_clone.abort();
            resolve.call0(&JsValue::NULL).ok();
        }) as Box<dyn FnOnce()>);

        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                timeout_ms as i32,
            )
            .ok();

        closure.forget();
    });

    // Race between fetch and timeout
    let fetch_promise = window.fetch_with_request(&request_with_signal);

    let result = js_sys::Promise::race(&js_sys::Array::of2(
        &fetch_promise,
        &timeout_promise,
    ));

    let value = JsFuture::from(result)
        .await
        .map_err(|e| {
            let error_str = format!("{:?}", e);
            if error_str.contains("abort") {
                Error::Timeout
            } else {
                Error::Network(error_str)
            }
        })?;

    // Check if we got a timeout (null value)
    if value.is_null() || value.is_undefined() {
        return Err(Error::Timeout);
    }

    Ok(value)
}

/// Parse headers from web_sys::Headers
fn parse_headers(web_headers: &web_sys::Headers) -> Result<Headers> {
    let mut headers = Headers::new();

    // Use entries() which returns a js_sys::Iterator
    let entries_iter = web_headers.entries();

    // Iterate using the iterator protocol
    loop {
        let next = entries_iter.next().map_err(|e| Error::Response(format!("{:?}", e)))?;

        if next.done() {
            break;
        }

        let arr: js_sys::Array = next.value().into();
        let key = arr.get(0).as_string().unwrap_or_default();
        let value = arr.get(1).as_string().unwrap_or_default();
        headers.insert(key, value);
    }

    Ok(headers)
}

/// Read the response body as bytes
async fn read_body(response: &web_sys::Response) -> Result<Vec<u8>> {
    let array_buffer_promise = response
        .array_buffer()
        .map_err(|e| Error::Response(format!("{:?}", e)))?;

    let array_buffer = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| Error::Response(format!("{:?}", e)))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}

