# File Upload & Download Tracking Guide

Comprehensive guide to tracking file uploads and downloads in the Ferric HTTP client.

## Overview

The Ferric HTTP client provides robust file transfer tracking with:

- ✅ **Upload Tracking**: Monitor file uploads with progress callbacks
- ✅ **Download Tracking**: Track download progress with speed and ETA
- ✅ **Batch Management**: Manage multiple concurrent transfers
- ✅ **Platform Support**: Works on both WASM and native targets
- ✅ **Detailed Metrics**: Speed, ETA, percentage, and stage tracking

## File Uploads

### Basic Upload

```rust
use ferric_http::FileUpload;

// WASM: Upload from browser
#[cfg(target_arch = "wasm32")]
async fn upload_file(file: web_sys::File) -> Result<(), Box<dyn std::error::Error>> {
    let response = FileUpload::new("https://api.example.com/upload", "document.pdf")
        .field_name("document")
        .file(file)
        .send()
        .await?;

    println!("Uploaded successfully! Status: {}", response.status());
    Ok(())
}

// Native: Upload from file path
#[cfg(not(target_arch = "wasm32"))]
async fn upload_file() -> Result<(), Box<dyn std::error::Error>> {
    let response = FileUpload::new("https://api.example.com/upload", "document.pdf")
        .field_name("document")
        .file_path("/path/to/document.pdf")
        .send()
        .await?;

    println!("Uploaded successfully! Status: {}", response.status());
    Ok(())
}
```

### Upload with Progress Tracking

```rust
use ferric_http::{FileUpload, UploadProgress, UploadStage};

async fn upload_with_progress(file: web_sys::File) -> Result<(), Box<dyn std::error::Error>> {
    let response = FileUpload::new("https://api.example.com/upload", "large-file.zip")
        .field_name("file")
        .header("Authorization", "Bearer token")
        .on_progress(|progress: &UploadProgress| {
            match &progress.stage {
                UploadStage::Starting => {
                    println!("Starting upload: {}", progress.filename);
                }
                UploadStage::Reading => {
                    println!("Reading file from disk...");
                }
                UploadStage::Uploading => {
                    println!("Uploading: {:.1}%", progress.percent());
                    println!("Speed: {}/s", progress.speed_formatted());

                    if let Some(eta) = progress.estimated_seconds {
                        println!("ETA: {:.0} seconds", eta);
                    }
                }
                UploadStage::Complete => {
                    println!("✅ Upload complete!");
                }
                UploadStage::Failed(err) => {
                    eprintln!("❌ Upload failed: {}", err);
                }
                UploadStage::Cancelled => {
                    println!("⚠️  Upload cancelled");
                }
            }
        })
        .file(file)
        .send()
        .await?;

    Ok(())
}
```

### Upload Progress Fields

```rust
pub struct UploadProgress {
    /// File name being uploaded
    pub filename: String,
    /// Total bytes to upload
    pub total_bytes: u64,
    /// Bytes uploaded so far
    pub uploaded_bytes: u64,
    /// Upload speed in bytes per second
    pub speed_bps: f64,
    /// Estimated time remaining in seconds
    pub estimated_seconds: Option<f64>,
    /// Current stage of upload
    pub stage: UploadStage,
}

impl UploadProgress {
    /// Get upload percentage (0.0 to 100.0)
    pub fn percent(&self) -> f64;

    /// Check if upload is complete
    pub fn is_complete(&self) -> bool;

    /// Check if upload failed
    pub fn is_failed(&self) -> bool;
}
```

## File Downloads

### Basic Download

```rust
use ferric_http::FileDownload;

// Download to memory
async fn download_file() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = FileDownload::new("https://example.com/large-file.zip")
        .filename("archive.zip")
        .download()
        .await?;

    println!("Downloaded {} bytes", bytes.len());
    Ok(bytes)
}

// Download and save to file (native only)
#[cfg(not(target_arch = "wasm32"))]
async fn download_and_save() -> Result<String, Box<dyn std::error::Error>> {
    let path = FileDownload::new("https://example.com/file.zip")
        .filename("archive.zip")
        .save_to("/path/to/save/archive.zip")
        .download_and_save()
        .await?;

    println!("Saved to: {}", path);
    Ok(path)
}

// Trigger browser download (WASM only)
#[cfg(target_arch = "wasm32")]
async fn download_in_browser() -> Result<(), Box<dyn std::error::Error>> {
    FileDownload::new("https://example.com/file.pdf")
        .filename("document.pdf")
        .download_in_browser()
        .await?;

    Ok(())
}
```

### Download with Progress Tracking

```rust
use ferric_http::{FileDownload, DownloadProgress, DownloadStage};

async fn download_with_progress() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = FileDownload::new("https://example.com/large-file.zip")
        .filename("archive.zip")
        .on_progress(|progress: &DownloadProgress| {
            match &progress.stage {
                DownloadStage::Starting => {
                    println!("Starting download...");
                }
                DownloadStage::Headers => {
                    println!("Headers received:");
                    println!("  Content-Type: {}",
                        progress.content_type.as_ref().unwrap_or(&"unknown".to_string()));
                    println!("  Size: {}",
                        progress.total_size_formatted().unwrap_or("unknown".to_string()));
                }
                DownloadStage::Downloading => {
                    if let Some(percent) = progress.percent() {
                        println!("Downloaded: {:.1}%", percent);
                    }
                    println!("Downloaded: {} / {}",
                        progress.downloaded_size_formatted(),
                        progress.total_size_formatted().unwrap_or("unknown".to_string())
                    );
                    println!("Speed: {}", progress.speed_formatted());

                    if let Some(eta) = progress.estimated_seconds {
                        println!("ETA: {:.0} seconds", eta);
                    }
                }
                DownloadStage::Complete => {
                    println!("✅ Download complete!");
                }
                DownloadStage::Failed(err) => {
                    eprintln!("❌ Download failed: {}", err);
                }
                DownloadStage::Cancelled => {
                    println!("⚠️  Download cancelled");
                }
            }
        })
        .download()
        .await?;

    Ok(bytes)
}
```

### Download Progress Fields

```rust
pub struct DownloadProgress {
    /// URL being downloaded
    pub url: String,
    /// Filename for the downloaded file
    pub filename: Option<String>,
    /// Total bytes to download (if known)
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Download speed in bytes per second
    pub speed_bps: f64,
    /// Estimated time remaining in seconds
    pub estimated_seconds: Option<f64>,
    /// Current stage of download
    pub stage: DownloadStage,
    /// Content type of the download
    pub content_type: Option<String>,
}

impl DownloadProgress {
    /// Get download percentage (0.0 to 100.0), or None if total size is unknown
    pub fn percent(&self) -> Option<f64>;

    /// Get human-readable file size
    pub fn downloaded_size_formatted(&self) -> String;

    /// Get human-readable total size
    pub fn total_size_formatted(&self) -> Option<String>;

    /// Get human-readable speed
    pub fn speed_formatted(&self) -> String;
}
```

## Batch Transfer Management

### Upload Manager

```rust
use ferric_http::{UploadManager, UploadProgress};

let upload_mgr = UploadManager::new();

// Track multiple uploads
for file in files {
    let progress = UploadProgress::new(&file.name, file.size);
    upload_mgr.track(progress);
}

// Get active uploads
let active = upload_mgr.active_uploads();
println!("Active uploads: {}", active.len());

// Get total progress across all files
let total = upload_mgr.total_progress();
println!("Overall progress: {:.1}%", total);

// Update a specific upload
upload_mgr.update("filename.pdf", 5000);

// Clear completed uploads
upload_mgr.clear_completed();
```

### Download Manager

```rust
use ferric_http::{DownloadManager, DownloadProgress};

let download_mgr = DownloadManager::new();

// Track multiple downloads
for url in urls {
    let progress = DownloadProgress::new(url);
    download_mgr.track(progress);
}

// Get statistics
let stats = download_mgr.stats();
println!("Active downloads: {}", stats.active_count);
println!("Completed downloads: {}", stats.completed_count);
println!("Failed downloads: {}", stats.failed_count);

// Get total progress
if let Some(percent) = stats.overall_percent() {
    println!("Overall progress: {:.1}%", percent);
}

// Combined download speed
println!("Total speed: {}", stats.speed_formatted());

// Clear completed downloads
download_mgr.clear_completed();
```

### Download Statistics

```rust
pub struct DownloadStats {
    /// Total number of downloads
    pub total_count: usize,
    /// Number of active downloads
    pub active_count: usize,
    /// Number of completed downloads
    pub completed_count: usize,
    /// Number of failed downloads
    pub failed_count: usize,
    /// Total bytes across all downloads
    pub total_bytes: u64,
    /// Downloaded bytes across all downloads
    pub downloaded_bytes: u64,
    /// Combined download speed
    pub total_speed_bps: f64,
}
```

## Platform Differences

### WASM (Browser)

```rust
#[cfg(target_arch = "wasm32")]
{
    // Upload from browser file input
    let upload = FileUpload::new(url, filename)
        .file(web_sys_file)
        .send()
        .await?;

    // Trigger browser download
    FileDownload::new(url)
        .filename("file.pdf")
        .download_in_browser()
        .await?;
}
```

### Native (Server/Desktop)

```rust
#[cfg(not(target_arch = "wasm32"))]
{
    // Upload from file path
    let upload = FileUpload::new(url, filename)
        .file_path("/path/to/file")
        .send()
        .await?;

    // Download and save to disk
    let path = FileDownload::new(url)
        .save_to("/path/to/save")
        .download_and_save()
        .await?;
}
```

## Advanced Usage

### Custom Headers

```rust
let response = FileUpload::new(url, filename)
    .header("Authorization", "Bearer token")
    .header("X-Custom-Header", "value")
    .file(file)
    .send()
    .await?;
```

### Custom Form Field Names

```rust
let response = FileUpload::new(url, filename)
    .field_name("document") // Default is "file"
    .file(file)
    .send()
    .await?;
```

### Error Handling

```rust
use ferric_http::{FileUpload, Error};

match FileUpload::new(url, filename).file(file).send().await {
    Ok(response) => {
        println!("Upload successful: {}", response.status());
    }
    Err(Error::Request(msg)) => {
        eprintln!("Request error: {}", msg);
    }
    Err(Error::Network(msg)) => {
        eprintln!("Network error: {}", msg);
    }
    Err(Error::Timeout) => {
        eprintln!("Upload timed out");
    }
    Err(e) => {
        eprintln!("Upload failed: {:?}", e);
    }
}
```

## UI Integration Example

### React-like Component

```rust
use ferric_core::prelude::*;
use ferric_http::{FileUpload, UploadProgress};

#[component(selector = "file-uploader")]
struct FileUploaderComponent {
    upload_progress: Signal<Option<UploadProgress>>,
    is_uploading: Signal<bool>,
}

impl FileUploaderComponent {
    pub async fn upload_file(&self, file: web_sys::File) {
        self.is_uploading.set(true);

        let progress_signal = self.upload_progress.clone();

        match FileUpload::new("https://api.example.com/upload", &file.name())
            .on_progress(move |progress| {
                progress_signal.set(Some(progress.clone()));
            })
            .file(file)
            .send()
            .await
        {
            Ok(response) => {
                println!("Upload complete: {}", response.status());
            }
            Err(e) => {
                eprintln!("Upload failed: {:?}", e);
            }
        }

        self.is_uploading.set(false);
    }

    pub fn render_progress(&self) -> String {
        if let Some(progress) = self.upload_progress.get().as_ref() {
            format!(
                r#"<div class="progress">
                    <div class="progress-bar" style="width: {}%"></div>
                    <span>{:.1}% - {}</span>
                </div>"#,
                progress.percent(),
                progress.percent(),
                progress.speed_formatted()
            )
        } else {
            String::new()
        }
    }
}
```

## Performance Tips

1. **Throttle Updates**: Progress callbacks are fired frequently. Consider throttling UI updates.
2. **Batch Operations**: Use managers for multiple concurrent transfers.
3. **Cancel on Unmount**: Clean up transfers when components unmount.
4. **Stream Large Files**: For very large files, consider streaming APIs.

## Security Considerations

1. **Validate File Types**: Always validate file types on the server
2. **Size Limits**: Enforce file size limits
3. **Authentication**: Use proper authentication headers
4. **HTTPS**: Always use HTTPS for file uploads
5. **Scan Uploads**: Scan uploaded files for malware

## License

MIT

