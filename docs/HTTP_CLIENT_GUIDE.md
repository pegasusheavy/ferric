# HTTP Client Guide

Complete guide to making HTTP requests in Ferric.

## Overview

The Ferric HTTP client provides:
- ✅ **Cross-platform**: Works on WASM and native targets
- ✅ **Type-safe**: Strongly typed requests and responses
- ✅ **Injectable**: Integrated with DI system
- ✅ **Progress Tracking**: Monitor uploads and downloads
- ✅ **Interceptors**: Modify requests and responses
- ✅ **Cancellation**: Cancel in-flight requests
- ✅ **Retry Logic**: Automatic retry with backoff

## Quick Start

### Basic Request

```rust
use ferric_http::Client;

async fn fetch_data() -> Result<String, Box<dyn std::error::Error>> {
    let response = Client::new()
        .get("https://api.example.com/data")
        .send()
        .await?;

    Ok(response.text()?)
}
```

### JSON Requests

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn create_user() -> Result<User, Box<dyn std::error::Error>> {
    let new_user = CreateUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let response = Client::new()
        .post("https://api.example.com/users")
        .json(&new_user)?
        .send()
        .await?;

    Ok(response.json()?)
}
```

## Using with Dependency Injection

### Setup

```rust
use ferric::prelude::*;
use ferric_http::prelude::*;

fn setup_http(injector: &Injector) {
    // Provide HTTP client
    HttpModule::provide(&injector);

    // Configure base URL
    injector.register_token(&BASE_URL, "https://api.example.com".to_string());

    // Configure timeout
    injector.register_token(&TIMEOUT_MS, 30000);

    // Configure authentication
    injector.register_token(&BEARER_TOKEN, "your-token".to_string());
}
```

### Injectable Service

```rust
#[injectable]
struct UserService {
    http: Rc<InjectableHttpClient>,
}

impl Injectable for UserService {
    fn create(injector: &Injector) -> Self {
        Self {
            http: injector.resolve_required::<InjectableHttpClient>(),
        }
    }
}

impl UserService {
    async fn get_user(&self, id: u32) -> Result<User, Box<dyn std::error::Error>> {
        let response = self.http
            .get(&format!("/users/{}", id))
            .send()
            .await?;

        Ok(response.json()?)
    }

    async fn create_user(&self, user: CreateUser) -> Result<User, Box<dyn std::error::Error>> {
        let response = self.http
            .post("/users")
            .json(&user)?
            .send()
            .await?;

        Ok(response.json()?)
    }
}
```

## Request Methods

### GET

```rust
let response = client.get("/api/data").send().await?;
```

### POST

```rust
let response = client
    .post("/api/users")
    .json(&data)?
    .send()
    .await?;
```

### PUT

```rust
let response = client
    .put("/api/users/1")
    .json(&updated_user)?
    .send()
    .await?;
```

### DELETE

```rust
let response = client.delete("/api/users/1").send().await?;
```

### PATCH

```rust
let response = client
    .patch("/api/users/1")
    .json(&partial_update)?
    .send()
    .await?;
```

## Request Configuration

### Headers

```rust
let response = client
    .get("/api/data")
    .header("Authorization", "Bearer token")
    .header("X-Custom-Header", "value")
    .send()
    .await?;
```

### Query Parameters

```rust
let response = client
    .get("/api/users")
    .query("page", "1")
    .query("size", "10")
    .query("sort", "name")
    .send()
    .await?;
```

### Timeout

```rust
let response = client
    .get("/api/slow-endpoint")
    .timeout_ms(5000) // 5 seconds
    .send()
    .await?;
```

### Authentication

```rust
// Bearer token
let response = client
    .get("/api/protected")
    .bearer_auth("your-token")
    .send()
    .await?;

// Basic auth
let response = client
    .get("/api/protected")
    .basic_auth("username", Some("password"))
    .send()
    .await?;
```

## File Uploads

See [File Transfer Guide](FILE_TRANSFER_GUIDE.md) for complete details.

```rust
use ferric_http::FileUpload;

async fn upload_file(file: web_sys::File) -> Result<(), Box<dyn std::error::Error>> {
    let response = FileUpload::new("https://api.example.com/upload", &file.name())
        .field_name("document")
        .header("Authorization", "Bearer token")
        .on_progress(|progress| {
            println!("Uploading: {:.1}%", progress.percent());
        })
        .file(file)
        .send()
        .await?;

    println!("Upload complete: {}", response.status());
    Ok(())
}
```

## File Downloads

```rust
use ferric_http::FileDownload;

async fn download_file() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = FileDownload::new("https://example.com/large-file.zip")
        .filename("archive.zip")
        .on_progress(|progress| {
            if let Some(percent) = progress.percent() {
                println!("Downloaded: {:.1}%", percent);
            }
        })
        .download()
        .await?;

    Ok(bytes)
}
```

## Interceptors

### Request Interceptor

```rust
use ferric_http::Interceptor;

fn logging_interceptor(request: &mut Request) -> Result<(), Error> {
    println!("Request: {} {}", request.method, request.url);
    Ok(())
}

// Use with client
let client = Client::builder()
    .interceptor(logging_interceptor)
    .build()?;
```

### Response Interceptor

```rust
fn status_checker(response: &Response) -> Result<(), Error> {
    if !response.is_success() {
        return Err(Error::Response(format!("Status: {}", response.status())));
    }
    Ok(())
}
```

### Authentication Interceptor

```rust
let client = Client::builder()
    .interceptor(bearer_interceptor("your-token"))
    .build()?;
```

## Error Handling

```rust
use ferric_http::Error;

match client.get("/api/data").send().await {
    Ok(response) => {
        if response.is_success() {
            let data: MyData = response.json()?;
            println!("Success: {:?}", data);
        } else {
            eprintln!("HTTP Error: {}", response.status());
        }
    }
    Err(Error::Timeout) => {
        eprintln!("Request timed out");
    }
    Err(Error::Network(msg)) => {
        eprintln!("Network error: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

## Cancellation

```rust
use ferric_http::CancellationToken;

let token = CancellationToken::new();
let token_clone = token.clone();

// Start request
let request = async move {
    client
        .get("/api/slow-endpoint")
        .send()
        .await
};

// Cancel from elsewhere
token.cancel();
```

## Retry Logic

```rust
use ferric_http::{RetryConfig, RetryStrategy};

let config = RetryConfig {
    max_retries: 3,
    strategy: RetryStrategy::ExponentialBackoff {
        initial_delay_ms: 1000,
        max_delay_ms: 10000,
        multiplier: 2.0,
    },
};

let client = Client::builder()
    .retry_config(config)
    .build()?;
```

## Progress Tracking

```rust
use ferric_http::Progress;

let progress = Progress::new()
    .on_download(|info| {
        println!("Downloaded: {:.1}%", info.percent());
    });

let response = client
    .get("https://example.com/large-file")
    .with_progress(progress)
    .send()
    .await?;
```

## Best Practices

### 1. Use Injectable HTTP Client

```rust
// ✅ Good
#[injectable]
struct ApiService {
    http: Rc<InjectableHttpClient>,
}

// ❌ Bad
#[injectable]
struct ApiService {
    // Creating client on each call
}
```

### 2. Handle Errors Properly

```rust
// ✅ Good
async fn load_data(&self) -> Result<Data, AppError> {
    self.http.get("/data").send().await
        .map_err(|e| AppError::Http(e))?
        .json()
        .map_err(|e| AppError::Parse(e))
}

// ❌ Bad
async fn load_data(&self) -> Data {
    self.http.get("/data").send().await.unwrap().json().unwrap()
}
```

### 3. Use Timeouts

```rust
// ✅ Good
let response = client
    .get("/slow-api")
    .timeout_ms(5000)
    .send()
    .await?;
```

### 4. Cancel Requests on Component Destroy

```rust
impl OnDestroy for MyComponent {
    fn fe_on_destroy(&self) {
        self.cancel_token.cancel();
    }
}
```

## Examples

See [examples/http-demo](../examples/http-demo/) for complete examples.

## API Reference

- `Client` - HTTP client
- `Request` - HTTP request builder
- `Response` - HTTP response
- `FileUpload` - File upload builder
- `FileDownload` - File download builder
- `Progress` - Progress tracking
- `Interceptor` - Request/response middleware
- `RetryConfig` - Retry configuration

## License

MIT

