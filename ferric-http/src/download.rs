//! File download tracking and management.

use crate::{Client, Error, Result};
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use tokio::io::AsyncWriteExt;

/// Download progress information.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// URL being downloaded.
    pub url: String,
    /// Filename for the downloaded file.
    pub filename: Option<String>,
    /// Total bytes to download (if known).
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,
    /// Download speed in bytes per second.
    pub speed_bps: f64,
    /// Estimated time remaining in seconds.
    pub estimated_seconds: Option<f64>,
    /// Current stage of download.
    pub stage: DownloadStage,
    /// Content type of the download.
    pub content_type: Option<String>,
}

impl DownloadProgress {
    /// Create a new download progress tracker.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            filename: None,
            total_bytes: None,
            downloaded_bytes: 0,
            speed_bps: 0.0,
            estimated_seconds: None,
            stage: DownloadStage::Starting,
            content_type: None,
        }
    }

    /// Get the download percentage (0.0 to 100.0).
    ///
    /// Returns None if total size is unknown.
    pub fn percent(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total == 0 {
                0.0
            } else {
                (self.downloaded_bytes as f64 / total as f64) * 100.0
            }
        })
    }

    /// Check if download is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self.stage, DownloadStage::Complete)
    }

    /// Check if download failed.
    pub fn is_failed(&self) -> bool {
        matches!(self.stage, DownloadStage::Failed(_))
    }

    /// Update downloaded bytes and calculate speed.
    pub fn update(&mut self, downloaded_bytes: u64, elapsed_ms: u64) {
        self.downloaded_bytes = downloaded_bytes;

        if elapsed_ms > 0 {
            self.speed_bps = (downloaded_bytes as f64 / elapsed_ms as f64) * 1000.0;

            if let Some(total) = self.total_bytes {
                let remaining_bytes = total.saturating_sub(downloaded_bytes);
                if self.speed_bps > 0.0 {
                    self.estimated_seconds = Some(remaining_bytes as f64 / self.speed_bps);
                }
            }
        }
    }

    /// Mark as complete.
    pub fn complete(&mut self) {
        if let Some(total) = self.total_bytes {
            self.downloaded_bytes = total;
        }
        self.stage = DownloadStage::Complete;
    }

    /// Mark as failed.
    pub fn fail(&mut self, error: String) {
        self.stage = DownloadStage::Failed(error);
    }

    /// Get human-readable file size.
    pub fn downloaded_size_formatted(&self) -> String {
        format_bytes(self.downloaded_bytes)
    }

    /// Get human-readable total size.
    pub fn total_size_formatted(&self) -> Option<String> {
        self.total_bytes.map(format_bytes)
    }

    /// Get human-readable speed.
    pub fn speed_formatted(&self) -> String {
        format!("{}/s", format_bytes(self.speed_bps as u64))
    }
}

/// Format bytes into human-readable form.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Stage of file download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStage {
    /// Download is starting.
    Starting,
    /// Headers received, preparing to download.
    Headers,
    /// Data is being downloaded.
    Downloading,
    /// Download is complete.
    Complete,
    /// Download failed.
    Failed(String),
    /// Download was cancelled.
    Cancelled,
}

/// Callback for download progress updates.
pub type DownloadProgressCallback = Rc<RefCell<dyn FnMut(&DownloadProgress)>>;

/// File download builder.
pub struct FileDownload {
    url: String,
    filename: Option<String>,
    progress_callback: Option<DownloadProgressCallback>,
    #[cfg(not(target_arch = "wasm32"))]
    save_path: Option<String>,
}

impl FileDownload {
    /// Create a new file download.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            filename: None,
            progress_callback: None,
            #[cfg(not(target_arch = "wasm32"))]
            save_path: None,
        }
    }

    /// Set the filename for the download.
    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set progress callback.
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&DownloadProgress) + 'static,
    {
        self.progress_callback = Some(Rc::new(RefCell::new(callback)));
        self
    }

    /// Set save path (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to(mut self, path: impl Into<String>) -> Self {
        self.save_path = Some(path.into());
        self
    }

    /// Execute the download and return the bytes.
    pub async fn download(self) -> Result<Vec<u8>> {
        let mut progress = DownloadProgress::new(&self.url);
        progress.filename = self.filename.clone();

        // Notify starting
        if let Some(ref callback) = self.progress_callback {
            callback.borrow_mut()(&progress);
        }

        // Make request
        let response = Client::new().get(&self.url).send().await?;

        // Extract headers
        progress.stage = DownloadStage::Headers;
        progress.content_type = response.header("content-type").cloned();

        if let Some(length_str) = response.header("content-length")
            && let Ok(length) = length_str.parse::<u64>() {
                progress.total_bytes = Some(length);
            }

        if let Some(ref callback) = self.progress_callback {
            callback.borrow_mut()(&progress);
        }

        // Download body
        progress.stage = DownloadStage::Downloading;
        let start = std::time::Instant::now();

        let bytes = response.bytes().to_vec();

        // Update final progress
        progress.downloaded_bytes = bytes.len() as u64;
        progress.update(bytes.len() as u64, start.elapsed().as_millis() as u64);
        progress.complete();

        if let Some(ref callback) = self.progress_callback {
            callback.borrow_mut()(&progress);
        }

        Ok(bytes)
    }

    /// Execute the download and save to file (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn download_and_save(self) -> Result<String> {
        use tokio::fs::File;

        let save_path = self.save_path.clone().ok_or_else(|| {
            Error::Request("No save path specified".to_string())
        })?;

        let bytes = self.download().await?;

        // Save to file
        let mut file = File::create(&save_path)
            .await
            .map_err(|e| Error::Request(format!("Failed to create file: {}", e)))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| Error::Request(format!("Failed to write file: {}", e)))?;

        Ok(save_path)
    }

    /// Trigger browser download (WASM only).
    #[cfg(target_arch = "wasm32")]
    pub async fn download_in_browser(self) -> Result<()> {
        use web_sys::{Blob, Url, HtmlAnchorElement};
        use js_sys::Array;

        let bytes = self.download().await?;

        let window = web_sys::window().ok_or_else(|| Error::Request("No window".to_string()))?;
        let document = window.document().ok_or_else(|| Error::Request("No document".to_string()))?;

        // Create blob
        let array = Array::new();
        array.push(&js_sys::Uint8Array::from(bytes.as_slice()));

        let blob = Blob::new_with_u8_array_sequence(&array)
            .map_err(|e| Error::Request(format!("Failed to create blob: {:?}", e)))?;

        // Create download link
        let url = Url::create_object_url_with_blob(&blob)
            .map_err(|e| Error::Request(format!("Failed to create URL: {:?}", e)))?;

        let anchor = document.create_element("a")
            .map_err(|e| Error::Request(format!("Failed to create anchor: {:?}", e)))?
            .dyn_into::<HtmlAnchorElement>()
            .map_err(|_| Error::Request("Failed to cast to anchor".to_string()))?;

        anchor.set_href(&url);
        if let Some(filename) = &self.filename {
            anchor.set_download(filename);
        }

        // Trigger download
        anchor.click();

        // Clean up
        Url::revoke_object_url(&url)
            .map_err(|e| Error::Request(format!("Failed to revoke URL: {:?}", e)))?;

        Ok(())
    }
}

/// Download manager for tracking multiple downloads.
pub struct DownloadManager {
    downloads: Rc<RefCell<Vec<DownloadProgress>>>,
}

impl DownloadManager {
    /// Create a new download manager.
    pub fn new() -> Self {
        Self {
            downloads: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get all active downloads.
    pub fn active_downloads(&self) -> Vec<DownloadProgress> {
        self.downloads
            .borrow()
            .iter()
            .filter(|d| !d.is_complete() && !d.is_failed())
            .cloned()
            .collect()
    }

    /// Get total download progress across all files.
    pub fn total_progress(&self) -> Option<f64> {
        let downloads = self.downloads.borrow();
        if downloads.is_empty() {
            return None;
        }

        let total_bytes: u64 = downloads.iter().filter_map(|d| d.total_bytes).sum();
        let downloaded_bytes: u64 = downloads.iter().map(|d| d.downloaded_bytes).sum();

        if total_bytes == 0 {
            None
        } else {
            Some((downloaded_bytes as f64 / total_bytes as f64) * 100.0)
        }
    }

    /// Track a new download.
    pub fn track(&self, progress: DownloadProgress) {
        self.downloads.borrow_mut().push(progress);
    }

    /// Update a download's progress.
    pub fn update(&self, url: &str, downloaded_bytes: u64) {
        if let Some(download) = self.downloads
            .borrow_mut()
            .iter_mut()
            .find(|d| d.url == url)
        {
            download.downloaded_bytes = downloaded_bytes;
        }
    }

    /// Clear completed downloads.
    pub fn clear_completed(&self) {
        self.downloads.borrow_mut().retain(|d| !d.is_complete());
    }

    /// Get download statistics.
    pub fn stats(&self) -> DownloadStats {
        let downloads = self.downloads.borrow();

        let total_count = downloads.len();
        let active_count = downloads.iter().filter(|d| !d.is_complete() && !d.is_failed()).count();
        let completed_count = downloads.iter().filter(|d| d.is_complete()).count();
        let failed_count = downloads.iter().filter(|d| d.is_failed()).count();

        let total_bytes = downloads.iter().filter_map(|d| d.total_bytes).sum();
        let downloaded_bytes = downloads.iter().map(|d| d.downloaded_bytes).sum();
        let total_speed = downloads.iter()
            .filter(|d| matches!(d.stage, DownloadStage::Downloading))
            .map(|d| d.speed_bps)
            .sum();

        DownloadStats {
            total_count,
            active_count,
            completed_count,
            failed_count,
            total_bytes,
            downloaded_bytes,
            total_speed_bps: total_speed,
        }
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Download statistics.
#[derive(Debug, Clone)]
pub struct DownloadStats {
    /// Total number of downloads.
    pub total_count: usize,
    /// Number of active downloads.
    pub active_count: usize,
    /// Number of completed downloads.
    pub completed_count: usize,
    /// Number of failed downloads.
    pub failed_count: usize,
    /// Total bytes across all downloads.
    pub total_bytes: u64,
    /// Downloaded bytes across all downloads.
    pub downloaded_bytes: u64,
    /// Combined download speed.
    pub total_speed_bps: f64,
}

impl DownloadStats {
    /// Get overall progress percentage.
    pub fn overall_percent(&self) -> Option<f64> {
        if self.total_bytes == 0 {
            None
        } else {
            Some((self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0)
        }
    }

    /// Get formatted total speed.
    pub fn speed_formatted(&self) -> String {
        format!("{}/s", format_bytes(self.total_speed_bps as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress() {
        let mut progress = DownloadProgress::new("https://example.com/file.zip");
        assert_eq!(progress.percent(), None);

        progress.total_bytes = Some(1000);
        progress.update(500, 1000);
        assert_eq!(progress.percent(), Some(50.0));
        assert!(progress.speed_bps > 0.0);

        progress.complete();
        assert!(progress.is_complete());
        assert_eq!(progress.percent(), Some(100.0));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_download_manager() {
        let manager = DownloadManager::new();

        let mut progress1 = DownloadProgress::new("https://example.com/file1.zip");
        progress1.total_bytes = Some(1000);
        progress1.downloaded_bytes = 500;

        let mut progress2 = DownloadProgress::new("https://example.com/file2.zip");
        progress2.total_bytes = Some(2000);
        progress2.downloaded_bytes = 1000;

        manager.track(progress1);
        manager.track(progress2);

        let stats = manager.stats();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.active_count, 2);
        assert_eq!(stats.total_bytes, 3000);
        assert_eq!(stats.downloaded_bytes, 1500);
        assert_eq!(stats.overall_percent(), Some(50.0));
    }
}

