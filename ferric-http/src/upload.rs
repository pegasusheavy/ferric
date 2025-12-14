//! File upload tracking and management.

use crate::{Error, Headers, Result};
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{File, FormData};

/// Upload progress information.
#[derive(Debug, Clone)]
pub struct UploadProgress {
    /// File name being uploaded.
    pub filename: String,
    /// Total bytes to upload.
    pub total_bytes: u64,
    /// Bytes uploaded so far.
    pub uploaded_bytes: u64,
    /// Upload speed in bytes per second.
    pub speed_bps: f64,
    /// Estimated time remaining in seconds.
    pub estimated_seconds: Option<f64>,
    /// Current stage of upload.
    pub stage: UploadStage,
}

impl UploadProgress {
    /// Create a new upload progress tracker.
    pub fn new(filename: impl Into<String>, total_bytes: u64) -> Self {
        Self {
            filename: filename.into(),
            total_bytes,
            uploaded_bytes: 0,
            speed_bps: 0.0,
            estimated_seconds: None,
            stage: UploadStage::Starting,
        }
    }

    /// Get the upload percentage (0.0 to 100.0).
    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.uploaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Check if upload is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self.stage, UploadStage::Complete)
    }

    /// Check if upload failed.
    pub fn is_failed(&self) -> bool {
        matches!(self.stage, UploadStage::Failed(_))
    }

    /// Update uploaded bytes and calculate speed.
    pub fn update(&mut self, uploaded_bytes: u64, elapsed_ms: u64) {
        self.uploaded_bytes = uploaded_bytes;

        if elapsed_ms > 0 {
            self.speed_bps = (uploaded_bytes as f64 / elapsed_ms as f64) * 1000.0;

            let remaining_bytes = self.total_bytes.saturating_sub(uploaded_bytes);
            if self.speed_bps > 0.0 {
                self.estimated_seconds = Some(remaining_bytes as f64 / self.speed_bps);
            }
        }
    }

    /// Mark as complete.
    pub fn complete(&mut self) {
        self.uploaded_bytes = self.total_bytes;
        self.stage = UploadStage::Complete;
    }

    /// Mark as failed.
    pub fn fail(&mut self, error: String) {
        self.stage = UploadStage::Failed(error);
    }

    /// Get human-readable speed.
    pub fn speed_formatted(&self) -> String {
        format!("{}/s", format_bytes(self.speed_bps as u64))
    }
}

/// Format bytes into human-readable form.
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Stage of file upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadStage {
    /// Upload is starting.
    Starting,
    /// File is being read.
    Reading,
    /// Data is being uploaded.
    Uploading,
    /// Upload is complete.
    Complete,
    /// Upload failed.
    Failed(String),
    /// Upload was cancelled.
    Cancelled,
}

/// Callback for upload progress updates.
pub type UploadProgressCallback = Rc<RefCell<dyn FnMut(&UploadProgress)>>;

/// File upload builder.
pub struct FileUpload {
    url: String,
    filename: String,
    #[cfg(target_arch = "wasm32")]
    file: Option<File>,
    #[cfg(not(target_arch = "wasm32"))]
    file_path: Option<String>,
    field_name: String,
    headers: Headers,
    progress_callback: Option<UploadProgressCallback>,
}

impl FileUpload {
    /// Create a new file upload.
    pub fn new(url: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            filename: filename.into(),
            #[cfg(target_arch = "wasm32")]
            file: None,
            #[cfg(not(target_arch = "wasm32"))]
            file_path: None,
            field_name: "file".to_string(),
            headers: Headers::new(),
            progress_callback: None,
        }
    }

    /// Set the file to upload (WASM only).
    #[cfg(target_arch = "wasm32")]
    pub fn file(mut self, file: File) -> Self {
        self.file = Some(file);
        self
    }

    /// Set the file path (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Set the form field name.
    pub fn field_name(mut self, name: impl Into<String>) -> Self {
        self.field_name = name.into();
        self
    }

    /// Add a header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set progress callback.
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&UploadProgress) + 'static,
    {
        self.progress_callback = Some(Rc::new(RefCell::new(callback)));
        self
    }

    /// Execute the upload.
    #[cfg(target_arch = "wasm32")]
    pub async fn send(self) -> Result<crate::Response> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode};

        let file = self.file.ok_or_else(|| Error::Request("No file provided".to_string()))?;
        let file_size = file.size() as u64;

        // Create form data
        let form_data = FormData::new()
            .map_err(|e| Error::Request(format!("Failed to create FormData: {:?}", e)))?;

        form_data
            .append_with_blob(&self.field_name, &file)
            .map_err(|e| Error::Request(format!("Failed to append file: {:?}", e)))?;

        // Create request
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(&form_data));

        let request = Request::new_with_str_and_init(&self.url, &opts)
            .map_err(|e| Error::Request(format!("Failed to create request: {:?}", e)))?;

        // Add headers
        let request_headers = request.headers();
        for (key, value) in self.headers.iter() {
            request_headers
                .set(key, value)
                .map_err(|e| Error::Request(format!("Failed to set header: {:?}", e)))?;
        }

        // Track progress
        if let Some(callback) = &self.progress_callback {
            let mut progress = UploadProgress::new(&self.filename, file_size);
            progress.stage = UploadStage::Uploading;
            callback.borrow_mut()(&progress);
        }

        // Send request
        let window = web_sys::window().ok_or_else(|| Error::Request("No window".to_string()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| Error::Request(format!("Fetch failed: {:?}", e)))?;

        let response: web_sys::Response = resp_value.dyn_into()
            .map_err(|_| Error::Request("Invalid response".to_string()))?;

        // Complete progress
        if let Some(callback) = &self.progress_callback {
            let mut progress = UploadProgress::new(&self.filename, file_size);
            progress.complete();
            callback.borrow_mut()(&progress);
        }

        // Convert to our Response type
        let status = response.status();
        let status_text = response.status_text();
        let response_headers = extract_web_sys_headers(&response.headers())?;
        let body = extract_web_sys_body(&response).await?;
        let final_url = response.url();

        Ok(crate::Response::new(
            status,
            status_text,
            response_headers,
            body,
            final_url,
        ))
    }

    /// Execute the upload (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn send(self) -> Result<crate::Response> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let file_path = self.file_path.ok_or_else(|| Error::Request("No file path provided".to_string()))?;

        // Read file
        let mut file = File::open(&file_path)
            .await
            .map_err(|e| Error::Request(format!("Failed to open file: {}", e)))?;

        let metadata = file.metadata()
            .await
            .map_err(|e| Error::Request(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();

        // Track progress - reading
        if let Some(callback) = &self.progress_callback {
            let mut progress = UploadProgress::new(&self.filename, file_size);
            progress.stage = UploadStage::Reading;
            callback.borrow_mut()(&progress);
        }

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| Error::Request(format!("Failed to read file: {}", e)))?;

        // Create multipart form
        let part = reqwest::multipart::Part::bytes(buffer)
            .file_name(self.filename.clone());

        let form = reqwest::multipart::Form::new()
            .part(self.field_name, part);

        // Track progress - uploading
        if let Some(callback) = &self.progress_callback {
            let mut progress = UploadProgress::new(&self.filename, file_size);
            progress.stage = UploadStage::Uploading;
            callback.borrow_mut()(&progress);
        }

        // Send request
        let client = reqwest::Client::new();
        let mut request = client.post(&self.url).multipart(form);

        for (key, value) in self.headers.iter() {
            request = request.header(key, value);
        }

        let response = request.send()
            .await
            .map_err(|e| Error::Request(format!("Upload failed: {}", e)))?;

        // Complete progress
        if let Some(callback) = &self.progress_callback {
            let mut progress = UploadProgress::new(&self.filename, file_size);
            progress.complete();
            callback.borrow_mut()(&progress);
        }

        // Convert response
        let status = response.status().as_u16();
        let status_text = response.status().to_string();
        let response_headers = extract_reqwest_headers(response.headers());
        let final_url = response.url().to_string();
        let body = response.bytes()
            .await
            .map_err(|e| Error::Response(format!("Failed to read response body: {}", e)))?
            .to_vec();

        Ok(crate::Response::new(
            status,
            status_text,
            response_headers,
            body,
            final_url,
        ))
    }
}

/// Extract headers from web_sys::Headers (WASM).
#[cfg(target_arch = "wasm32")]
fn extract_web_sys_headers(web_headers: &web_sys::Headers) -> Result<Headers> {
    let mut headers = Headers::new();

    let entries_iter = web_headers.entries();

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

/// Extract body from web_sys::Response (WASM).
#[cfg(target_arch = "wasm32")]
async fn extract_web_sys_body(response: &web_sys::Response) -> Result<Vec<u8>> {
    use wasm_bindgen_futures::JsFuture;

    let array_buffer_promise = response
        .array_buffer()
        .map_err(|e| Error::Response(format!("{:?}", e)))?;

    let array_buffer = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| Error::Response(format!("{:?}", e)))?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    Ok(uint8_array.to_vec())
}

/// Extract headers from reqwest::HeaderMap (native).
#[cfg(not(target_arch = "wasm32"))]
fn extract_reqwest_headers(header_map: &reqwest::header::HeaderMap) -> Headers {
    let mut headers = Headers::new();
    for (key, value) in header_map.iter() {
        if let Ok(val_str) = value.to_str() {
            headers.insert(key.as_str().to_string(), val_str.to_string());
        }
    }
    headers
}

/// Upload manager for tracking multiple uploads.
pub struct UploadManager {
    uploads: Rc<RefCell<Vec<UploadProgress>>>,
}

impl UploadManager {
    /// Create a new upload manager.
    pub fn new() -> Self {
        Self {
            uploads: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get all active uploads.
    pub fn active_uploads(&self) -> Vec<UploadProgress> {
        self.uploads
            .borrow()
            .iter()
            .filter(|u| !u.is_complete() && !u.is_failed())
            .cloned()
            .collect()
    }

    /// Get total upload progress across all files.
    pub fn total_progress(&self) -> f64 {
        let uploads = self.uploads.borrow();
        if uploads.is_empty() {
            return 0.0;
        }

        let total_bytes: u64 = uploads.iter().map(|u| u.total_bytes).sum();
        let uploaded_bytes: u64 = uploads.iter().map(|u| u.uploaded_bytes).sum();

        if total_bytes == 0 {
            0.0
        } else {
            (uploaded_bytes as f64 / total_bytes as f64) * 100.0
        }
    }

    /// Track a new upload.
    pub fn track(&self, progress: UploadProgress) {
        self.uploads.borrow_mut().push(progress);
    }

    /// Update an upload's progress.
    pub fn update(&self, filename: &str, uploaded_bytes: u64) {
        if let Some(upload) = self.uploads
            .borrow_mut()
            .iter_mut()
            .find(|u| u.filename == filename)
        {
            upload.uploaded_bytes = uploaded_bytes;
        }
    }

    /// Clear completed uploads.
    pub fn clear_completed(&self) {
        self.uploads.borrow_mut().retain(|u| !u.is_complete());
    }
}

impl Default for UploadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_progress() {
        let mut progress = UploadProgress::new("test.txt", 1000);
        assert_eq!(progress.percent(), 0.0);

        progress.update(500, 1000);
        assert_eq!(progress.percent(), 50.0);
        assert!(progress.speed_bps > 0.0);

        progress.complete();
        assert!(progress.is_complete());
        assert_eq!(progress.percent(), 100.0);
    }

    #[test]
    fn test_upload_manager() {
        let manager = UploadManager::new();

        let progress1 = UploadProgress::new("file1.txt", 1000);
        let mut progress2 = UploadProgress::new("file2.txt", 2000);
        progress2.uploaded_bytes = 1000;

        manager.track(progress1);
        manager.track(progress2);

        let total = manager.total_progress();
        assert!((total - 33.33).abs() < 0.1);
    }
}

