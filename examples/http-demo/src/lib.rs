//! Comprehensive HTTP client demonstration.
//!
//! This example shows all the features of the ferric-http client:
//! - Basic HTTP methods (GET, POST, PUT, DELETE, PATCH)
//! - JSON serialization/deserialization
//! - Request/response interceptors
//! - Progress tracking
//! - Request cancellation
//! - Retry logic with exponential backoff
//! - Error handling
//! - Request builders

use ferric_http::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// ============================================================================
// Example Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: u32,
    user_id: u32,
    title: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

// ============================================================================
// Example 1: Basic HTTP Methods
// ============================================================================

async fn example_basic_requests() -> Result<()> {
    web_sys::console::log_1(&"=== Example 1: Basic HTTP Methods ===".into());

    let client = Client::new();

    // GET request
    let response = client
        .get("https://jsonplaceholder.typicode.com/users/1")
        .send()
        .await?;

    web_sys::console::log_1(&format!("GET status: {}", response.status()).into());
    let user: User = response.json()?;
    web_sys::console::log_1(&format!("User: {:?}", user).into());

    // POST request with JSON
    let new_user = CreateUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let response = client
        .post("https://jsonplaceholder.typicode.com/users")
        .json(&new_user)?
        .send()
        .await?;

    web_sys::console::log_1(&format!("POST status: {}", response.status()).into());

    // PUT request
    let updated_user = User {
        id: 1,
        name: "Alice Updated".to_string(),
        email: "alice.updated@example.com".to_string(),
    };

    let _response = client
        .put("https://jsonplaceholder.typicode.com/users/1")
        .json(&updated_user)?
        .send()
        .await?;

    // DELETE request
    let _response = client
        .delete("https://jsonplaceholder.typicode.com/users/1")
        .send()
        .await?;

    Ok(())
}

// ============================================================================
// Example 2: Client Builder with Configuration
// ============================================================================

async fn example_client_builder() -> Result<()> {
    web_sys::console::log_1(&"=== Example 2: Client Builder ===".into());

    let client = Client::builder()
        .base_url("https://jsonplaceholder.typicode.com")
        .timeout(30) // 30 seconds
        .default_header("User-Agent", "Ferric-HTTP-Example/1.0")
        .default_header("Accept", "application/json")
        .build()?;

    // Relative URLs now work with base_url
    let response = client.get("/users").send().await?;
    web_sys::console::log_1(&format!("Fetched {} users", response.text()?.len()).into());

    Ok(())
}

// ============================================================================
// Example 3: Interceptors
// ============================================================================

async fn example_interceptors() -> Result<()> {
    web_sys::console::log_1(&"=== Example 3: Interceptors ===".into());

    let client = Client::builder()
        .base_url("https://jsonplaceholder.typicode.com")
        .interceptor(logging_interceptor())  // Log all requests/responses
        .interceptor(bearer_interceptor("fake-token-12345"))  // Add auth header
        .interceptor(json_content_type_interceptor())  // Auto-add JSON content-type
        .build()?;

    let response = client.get("/posts/1").send().await?;
    // Interceptors will have logged the request and added headers

    web_sys::console::log_1(&format!("Response: {}", response.status()).into());

    Ok(())
}

// ============================================================================
// Example 4: Progress Tracking
// ============================================================================

async fn example_progress() -> Result<()> {
    web_sys::console::log_1(&"=== Example 4: Progress Tracking ===".into());

    let client = Client::new();

    let progress = Progress::new()
        .on_download(|info| {
            web_sys::console::log_1(&format!(
                "Downloaded: {:.1}% ({} / {} bytes)",
                info.percent(),
                info.loaded,
                info.total.unwrap_or(0)
            ).into());
        });

    let response = client
        .get("https://jsonplaceholder.typicode.com/photos")
        .with_progress(progress)
        .send()
        .await?;

    web_sys::console::log_1(&format!("Download complete: {} bytes", response.text()?.len()).into());

    Ok(())
}

// ============================================================================
// Example 5: Request Cancellation
// ============================================================================

async fn example_cancellation() -> Result<()> {
    web_sys::console::log_1(&"=== Example 5: Request Cancellation ===".into());

    let client = Client::new();
    let (token, trigger) = CancellationToken::new();

    // Spawn a task to cancel after 100ms
    let trigger_clone = trigger.clone();
    spawn_local(async move {
        // In a real app, you might cancel based on user action
        gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
        trigger_clone.cancel();
        web_sys::console::log_1(&"Request cancelled!".into());
    });

    // This request might be cancelled
    let result = client
        .get("https://jsonplaceholder.typicode.com/posts")
        .with_cancellation(token)
        .send()
        .await;

    match result {
        Ok(response) => web_sys::console::log_1(&format!("Request succeeded: {}", response.status()).into()),
        Err(Error::Cancelled) => web_sys::console::log_1(&"Request was cancelled!".into()),
        Err(e) => web_sys::console::log_1(&format!("Error: {}", e).into()),
    }

    Ok(())
}

// ============================================================================
// Example 6: Retry Logic
// ============================================================================

async fn example_retry() -> Result<()> {
    web_sys::console::log_1(&"=== Example 6: Retry Logic ===".into());

    let client = Client::new();

    // Retry with exponential backoff
    let response = client
        .get("https://jsonplaceholder.typicode.com/posts/1")
        .with_retry(RetryPolicy::exponential().max_retries(3))
        .send()
        .await?;

    web_sys::console::log_1(&format!("Got response after retries: {}", response.status()).into());

    // Custom retry configuration
    let retry_config = RetryConfig {
        max_retries: 5,
        strategy: RetryStrategy::Exponential {
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            jitter: 0.1,
        },
        retry_on_status: vec![408, 429, 500, 502, 503, 504],
    };

    let response = client
        .get("https://jsonplaceholder.typicode.com/users")
        .with_retry_config(retry_config)
        .send()
        .await?;

    web_sys::console::log_1(&format!("Custom retry complete: {} users", response.text()?.len()).into());

    Ok(())
}

// ============================================================================
// Example 7: Error Handling
// ============================================================================

async fn example_error_handling() -> Result<()> {
    web_sys::console::log_1(&"=== Example 7: Error Handling ===".into());

    let client = Client::new();

    // Handle 404
    match client.get("https://jsonplaceholder.typicode.com/users/999999").send().await {
        Ok(response) => {
            if response.is_success() {
                web_sys::console::log_1(&"User found".into());
            } else {
                web_sys::console::log_1(&format!("Error: {}", response.status()).into());
            }
        }
        Err(e) => {
            web_sys::console::log_1(&format!("Request failed: {}", e).into());
        }
    }

    // Network error handling
    match client.get("https://invalid-domain-that-doesnt-exist-12345.com").send().await {
        Ok(_) => web_sys::console::log_1(&"Unexpected success".into()),
        Err(Error::Network(msg)) => web_sys::console::log_1(&format!("Network error: {}", msg).into()),
        Err(e) => web_sys::console::log_1(&format!("Other error: {}", e).into()),
    }

    Ok(())
}

// ============================================================================
// Example 8: Convenience Functions
// ============================================================================

async fn example_convenience_functions() -> Result<()> {
    web_sys::console::log_1(&"=== Example 8: Convenience Functions ===".into());

    // Simple GET
    let response = get("https://jsonplaceholder.typicode.com/posts/1").await?;
    web_sys::console::log_1(&format!("GET: {}", response.status()).into());

    // POST with JSON
    let new_post = Post {
        id: 0,
        user_id: 1,
        title: "My Post".to_string(),
        body: "Post content".to_string(),
    };

    let response = post_json("https://jsonplaceholder.typicode.com/posts", &new_post).await?;
    web_sys::console::log_1(&format!("POST JSON: {}", response.status()).into());

    // Other methods
    let _response = put("https://jsonplaceholder.typicode.com/posts/1").await?;
    let _response = patch("https://jsonplaceholder.typicode.com/posts/1").await?;
    let _response = delete("https://jsonplaceholder.typicode.com/posts/1").await?;

    Ok(())
}

// ============================================================================
// Example 9: Headers Management
// ============================================================================

async fn example_headers() -> Result<()> {
    web_sys::console::log_1(&"=== Example 9: Headers Management ===".into());

    let client = Client::new();

    let response = client
        .get("https://jsonplaceholder.typicode.com/users")
        .header("X-Custom-Header", "my-value")
        .header("Accept-Language", "en-US")
        .send()
        .await?;

    // Read response headers
    if let Some(content_type) = response.headers().get("content-type") {
        web_sys::console::log_1(&format!("Content-Type: {}", content_type).into());
    }

    Ok(())
}

// ============================================================================
// Example 10: Advanced - Combined Features
// ============================================================================

async fn example_advanced() -> Result<()> {
    web_sys::console::log_1(&"=== Example 10: Advanced Features ===".into());

    // Create a fully-configured client
    let client = Client::builder()
        .base_url("https://jsonplaceholder.typicode.com")
        .timeout(60)
        .interceptor(logging_interceptor())
        .interceptor(bearer_interceptor("my-api-token"))
        .default_header("X-App-Version", "1.0.0")
        .build()?;

    let (token, trigger) = CancellationToken::new();

    let progress = Progress::new()
        .on_download(|info| {
            if let Some(total) = info.total {
                web_sys::console::log_1(&format!("Progress: {:.0}%", info.percent()).into());
            }
        });

    // Complex request with all features
    let response = client
        .get("/posts")
        .header("X-Request-ID", "unique-id-123")
        .with_progress(progress)
        .with_cancellation(token)
        .with_retry(RetryPolicy::exponential().max_retries(3))
        .send()
        .await?;

    let posts: Vec<Post> = response.json()?;
    web_sys::console::log_1(&format!("Fetched {} posts", posts.len()).into());

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[wasm_bindgen(start)]
pub fn main() -> std::result::Result<(), JsValue> {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"🚀 Ferric HTTP Demo Starting...".into());

    // Run all examples
    spawn_local(async {
        if let Err(e) = run_all_examples().await {
            web_sys::console::error_1(&format!("Example failed: {}", e).into());
        }
    });

    Ok(())
}

async fn run_all_examples() -> Result<()> {
    example_basic_requests().await?;
    example_client_builder().await?;
    example_interceptors().await?;
    example_progress().await?;
    example_cancellation().await?;
    example_retry().await?;
    example_error_handling().await?;
    example_convenience_functions().await?;
    example_headers().await?;
    example_advanced().await?;

    web_sys::console::log_1(&"✅ All examples completed!".into());
    Ok(())
}

