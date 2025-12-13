//! Progress tracking for HTTP requests.
//!
//! Provides callbacks for tracking upload and download progress.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, Progress, ProgressCallback};
//!
//! let progress = Progress::new()
//!     .on_upload(|progress| {
//!         println!("Upload: {:.1}%", progress.percent());
//!     })
//!     .on_download(|progress| {
//!         println!("Download: {} / {} bytes", progress.loaded, progress.total.unwrap_or(0));
//!     });
//!
//! let response = client
//!     .get("https://example.com/large-file")
//!     .with_progress(progress)
//!     .send()
//!     .await?;
//! ```

use std::fmt;
use std::sync::Arc;

/// Progress information for an upload or download operation.
#[derive(Debug, Clone, Copy)]
pub struct ProgressInfo {
    /// Number of bytes transferred so far.
    pub loaded: u64,
    /// Total number of bytes (if known).
    pub total: Option<u64>,
    /// Whether the operation is complete.
    pub complete: bool,
}

impl ProgressInfo {
    /// Create a new progress info.
    pub fn new(loaded: u64, total: Option<u64>) -> Self {
        Self {
            loaded,
            total,
            complete: false,
        }
    }

    /// Create completed progress info.
    pub fn completed(loaded: u64) -> Self {
        Self {
            loaded,
            total: Some(loaded),
            complete: true,
        }
    }

    /// Get progress as a percentage (0.0 to 100.0).
    ///
    /// Returns 0.0 if total is unknown.
    pub fn percent(&self) -> f64 {
        match self.total {
            Some(total) if total > 0 => (self.loaded as f64 / total as f64) * 100.0,
            _ => 0.0,
        }
    }

    /// Get progress as a fraction (0.0 to 1.0).
    ///
    /// Returns None if total is unknown.
    pub fn fraction(&self) -> Option<f64> {
        self.total.map(|total| {
            if total > 0 {
                self.loaded as f64 / total as f64
            } else {
                0.0
            }
        })
    }

    /// Get remaining bytes.
    ///
    /// Returns None if total is unknown.
    pub fn remaining(&self) -> Option<u64> {
        self.total.map(|total| total.saturating_sub(self.loaded))
    }

    /// Check if the total size is known.
    pub fn is_length_known(&self) -> bool {
        self.total.is_some()
    }

    /// Check if the operation is complete.
    pub fn is_complete(&self) -> bool {
        self.complete || self.total.map(|t| self.loaded >= t).unwrap_or(false)
    }
}

impl Default for ProgressInfo {
    fn default() -> Self {
        Self::new(0, None)
    }
}

/// Type alias for a progress callback function.
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// Progress tracking configuration.
#[derive(Clone)]
pub struct Progress {
    /// Callback for upload progress.
    pub(crate) upload_callback: Option<ProgressCallback>,
    /// Callback for download progress.
    pub(crate) download_callback: Option<ProgressCallback>,
    /// Minimum interval between callbacks (in milliseconds).
    pub(crate) throttle_ms: u32,
}

impl Progress {
    /// Create a new progress tracker.
    pub fn new() -> Self {
        Self {
            upload_callback: None,
            download_callback: None,
            throttle_ms: 100, // Default 100ms throttle
        }
    }

    /// Set the upload progress callback.
    pub fn on_upload<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressInfo) + Send + Sync + 'static,
    {
        self.upload_callback = Some(Arc::new(callback));
        self
    }

    /// Set the download progress callback.
    pub fn on_download<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressInfo) + Send + Sync + 'static,
    {
        self.download_callback = Some(Arc::new(callback));
        self
    }

    /// Set both upload and download callbacks to the same function.
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressInfo) + Send + Sync + Clone + 'static,
    {
        let cb1 = Arc::new(callback.clone());
        let cb2 = Arc::new(callback);
        self.upload_callback = Some(cb1);
        self.download_callback = Some(cb2);
        self
    }

    /// Set the throttle interval in milliseconds.
    ///
    /// Progress callbacks will be called at most once per interval.
    pub fn throttle_ms(mut self, ms: u32) -> Self {
        self.throttle_ms = ms;
        self
    }

    /// Notify upload progress.
    pub fn notify_upload(&self, info: ProgressInfo) {
        if let Some(ref cb) = self.upload_callback {
            cb(info);
        }
    }

    /// Notify download progress.
    pub fn notify_download(&self, info: ProgressInfo) {
        if let Some(ref cb) = self.download_callback {
            cb(info);
        }
    }

    /// Check if any callbacks are registered.
    pub fn has_callbacks(&self) -> bool {
        self.upload_callback.is_some() || self.download_callback.is_some()
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Progress")
            .field("has_upload_callback", &self.upload_callback.is_some())
            .field("has_download_callback", &self.download_callback.is_some())
            .field("throttle_ms", &self.throttle_ms)
            .finish()
    }
}

/// Tracks progress state for throttling callbacks.
pub struct ProgressTracker {
    progress: Progress,
    last_upload_time: std::time::Instant,
    last_download_time: std::time::Instant,
    last_upload_loaded: u64,
    last_download_loaded: u64,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new(progress: Progress) -> Self {
        Self {
            progress,
            last_upload_time: std::time::Instant::now(),
            last_download_time: std::time::Instant::now(),
            last_upload_loaded: 0,
            last_download_loaded: 0,
        }
    }

    /// Report upload progress (with throttling).
    pub fn report_upload(&mut self, loaded: u64, total: Option<u64>) {
        let info = ProgressInfo::new(loaded, total);

        // Always report completion
        if info.is_complete() {
            self.progress.notify_upload(ProgressInfo::completed(loaded));
            return;
        }

        // Check throttle
        let elapsed = self.last_upload_time.elapsed();
        if elapsed.as_millis() >= self.progress.throttle_ms as u128
            || loaded != self.last_upload_loaded
        {
            self.progress.notify_upload(info);
            self.last_upload_time = std::time::Instant::now();
            self.last_upload_loaded = loaded;
        }
    }

    /// Report download progress (with throttling).
    pub fn report_download(&mut self, loaded: u64, total: Option<u64>) {
        let info = ProgressInfo::new(loaded, total);

        // Always report completion
        if info.is_complete() {
            self.progress.notify_download(ProgressInfo::completed(loaded));
            return;
        }

        // Check throttle
        let elapsed = self.last_download_time.elapsed();
        if elapsed.as_millis() >= self.progress.throttle_ms as u128
            || loaded != self.last_download_loaded
        {
            self.progress.notify_download(info);
            self.last_download_time = std::time::Instant::now();
            self.last_download_loaded = loaded;
        }
    }

    /// Get the underlying progress config.
    pub fn config(&self) -> &Progress {
        &self.progress
    }
}

/// Event emitted during request/response processing.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Request is about to start.
    Started,
    /// Upload progress update.
    UploadProgress(ProgressInfo),
    /// Upload complete.
    UploadComplete,
    /// Download progress update.
    DownloadProgress(ProgressInfo),
    /// Download complete.
    DownloadComplete,
    /// Request completed successfully.
    Completed,
    /// Request failed with an error.
    Error(String),
}

impl ProgressEvent {
    /// Check if this is a terminal event (Completed or Error).
    pub fn is_terminal(&self) -> bool {
        matches!(self, ProgressEvent::Completed | ProgressEvent::Error(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn test_progress_info() {
        let info = ProgressInfo::new(50, Some(100));
        assert_eq!(info.percent(), 50.0);
        assert_eq!(info.fraction(), Some(0.5));
        assert_eq!(info.remaining(), Some(50));
        assert!(!info.is_complete());
    }

    #[test]
    fn test_progress_complete() {
        let info = ProgressInfo::completed(100);
        assert!(info.is_complete());
        assert_eq!(info.percent(), 100.0);
    }

    #[test]
    fn test_progress_unknown_total() {
        let info = ProgressInfo::new(50, None);
        assert_eq!(info.percent(), 0.0);
        assert!(info.fraction().is_none());
        assert!(info.remaining().is_none());
    }

    #[test]
    fn test_progress_callback() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = counter.clone();

        let progress = Progress::new().on_download(move |info| {
            counter_clone.store(info.loaded, Ordering::SeqCst);
        });

        progress.notify_download(ProgressInfo::new(42, Some(100)));
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }
}


//!
//! Provides callbacks for tracking upload and download progress.
//!
//! ## Usage
//!
//! ```ignore
//! use ferric_http::{Client, Progress, ProgressCallback};
//!
//! let progress = Progress::new()
//!     .on_upload(|progress| {
//!         println!("Upload: {:.1}%", progress.percent());
//!     })
//!     .on_download(|progress| {
//!         println!("Download: {} / {} bytes", progress.loaded, progress.total.unwrap_or(0));
//!     });
//!
//! let response = client
//!     .get("https://example.com/large-file")
//!     .with_progress(progress)
//!     .send()
//!     .await?;
//! ```

use std::fmt;
use std::sync::Arc;

/// Progress information for an upload or download operation.
#[derive(Debug, Clone, Copy)]
pub struct ProgressInfo {
    /// Number of bytes transferred so far.
    pub loaded: u64,
    /// Total number of bytes (if known).
    pub total: Option<u64>,
    /// Whether the operation is complete.
    pub complete: bool,
}

impl ProgressInfo {
    /// Create a new progress info.
    pub fn new(loaded: u64, total: Option<u64>) -> Self {
        Self {
            loaded,
            total,
            complete: false,
        }
    }

    /// Create completed progress info.
    pub fn completed(loaded: u64) -> Self {
        Self {
            loaded,
            total: Some(loaded),
            complete: true,
        }
    }

    /// Get progress as a percentage (0.0 to 100.0).
    ///
    /// Returns 0.0 if total is unknown.
    pub fn percent(&self) -> f64 {
        match self.total {
            Some(total) if total > 0 => (self.loaded as f64 / total as f64) * 100.0,
            _ => 0.0,
        }
    }

    /// Get progress as a fraction (0.0 to 1.0).
    ///
    /// Returns None if total is unknown.
    pub fn fraction(&self) -> Option<f64> {
        self.total.map(|total| {
            if total > 0 {
                self.loaded as f64 / total as f64
            } else {
                0.0
            }
        })
    }

    /// Get remaining bytes.
    ///
    /// Returns None if total is unknown.
    pub fn remaining(&self) -> Option<u64> {
        self.total.map(|total| total.saturating_sub(self.loaded))
    }

    /// Check if the total size is known.
    pub fn is_length_known(&self) -> bool {
        self.total.is_some()
    }

    /// Check if the operation is complete.
    pub fn is_complete(&self) -> bool {
        self.complete || self.total.map(|t| self.loaded >= t).unwrap_or(false)
    }
}

impl Default for ProgressInfo {
    fn default() -> Self {
        Self::new(0, None)
    }
}

/// Type alias for a progress callback function.
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// Progress tracking configuration.
#[derive(Clone)]
pub struct Progress {
    /// Callback for upload progress.
    pub(crate) upload_callback: Option<ProgressCallback>,
    /// Callback for download progress.
    pub(crate) download_callback: Option<ProgressCallback>,
    /// Minimum interval between callbacks (in milliseconds).
    pub(crate) throttle_ms: u32,
}

impl Progress {
    /// Create a new progress tracker.
    pub fn new() -> Self {
        Self {
            upload_callback: None,
            download_callback: None,
            throttle_ms: 100, // Default 100ms throttle
        }
    }

    /// Set the upload progress callback.
    pub fn on_upload<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressInfo) + Send + Sync + 'static,
    {
        self.upload_callback = Some(Arc::new(callback));
        self
    }

    /// Set the download progress callback.
    pub fn on_download<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressInfo) + Send + Sync + 'static,
    {
        self.download_callback = Some(Arc::new(callback));
        self
    }

    /// Set both upload and download callbacks to the same function.
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(ProgressInfo) + Send + Sync + Clone + 'static,
    {
        let cb1 = Arc::new(callback.clone());
        let cb2 = Arc::new(callback);
        self.upload_callback = Some(cb1);
        self.download_callback = Some(cb2);
        self
    }

    /// Set the throttle interval in milliseconds.
    ///
    /// Progress callbacks will be called at most once per interval.
    pub fn throttle_ms(mut self, ms: u32) -> Self {
        self.throttle_ms = ms;
        self
    }

    /// Notify upload progress.
    pub fn notify_upload(&self, info: ProgressInfo) {
        if let Some(ref cb) = self.upload_callback {
            cb(info);
        }
    }

    /// Notify download progress.
    pub fn notify_download(&self, info: ProgressInfo) {
        if let Some(ref cb) = self.download_callback {
            cb(info);
        }
    }

    /// Check if any callbacks are registered.
    pub fn has_callbacks(&self) -> bool {
        self.upload_callback.is_some() || self.download_callback.is_some()
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Progress")
            .field("has_upload_callback", &self.upload_callback.is_some())
            .field("has_download_callback", &self.download_callback.is_some())
            .field("throttle_ms", &self.throttle_ms)
            .finish()
    }
}

/// Tracks progress state for throttling callbacks.
pub struct ProgressTracker {
    progress: Progress,
    last_upload_time: std::time::Instant,
    last_download_time: std::time::Instant,
    last_upload_loaded: u64,
    last_download_loaded: u64,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new(progress: Progress) -> Self {
        Self {
            progress,
            last_upload_time: std::time::Instant::now(),
            last_download_time: std::time::Instant::now(),
            last_upload_loaded: 0,
            last_download_loaded: 0,
        }
    }

    /// Report upload progress (with throttling).
    pub fn report_upload(&mut self, loaded: u64, total: Option<u64>) {
        let info = ProgressInfo::new(loaded, total);

        // Always report completion
        if info.is_complete() {
            self.progress.notify_upload(ProgressInfo::completed(loaded));
            return;
        }

        // Check throttle
        let elapsed = self.last_upload_time.elapsed();
        if elapsed.as_millis() >= self.progress.throttle_ms as u128
            || loaded != self.last_upload_loaded
        {
            self.progress.notify_upload(info);
            self.last_upload_time = std::time::Instant::now();
            self.last_upload_loaded = loaded;
        }
    }

    /// Report download progress (with throttling).
    pub fn report_download(&mut self, loaded: u64, total: Option<u64>) {
        let info = ProgressInfo::new(loaded, total);

        // Always report completion
        if info.is_complete() {
            self.progress.notify_download(ProgressInfo::completed(loaded));
            return;
        }

        // Check throttle
        let elapsed = self.last_download_time.elapsed();
        if elapsed.as_millis() >= self.progress.throttle_ms as u128
            || loaded != self.last_download_loaded
        {
            self.progress.notify_download(info);
            self.last_download_time = std::time::Instant::now();
            self.last_download_loaded = loaded;
        }
    }

    /// Get the underlying progress config.
    pub fn config(&self) -> &Progress {
        &self.progress
    }
}

/// Event emitted during request/response processing.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Request is about to start.
    Started,
    /// Upload progress update.
    UploadProgress(ProgressInfo),
    /// Upload complete.
    UploadComplete,
    /// Download progress update.
    DownloadProgress(ProgressInfo),
    /// Download complete.
    DownloadComplete,
    /// Request completed successfully.
    Completed,
    /// Request failed with an error.
    Error(String),
}

impl ProgressEvent {
    /// Check if this is a terminal event (Completed or Error).
    pub fn is_terminal(&self) -> bool {
        matches!(self, ProgressEvent::Completed | ProgressEvent::Error(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn test_progress_info() {
        let info = ProgressInfo::new(50, Some(100));
        assert_eq!(info.percent(), 50.0);
        assert_eq!(info.fraction(), Some(0.5));
        assert_eq!(info.remaining(), Some(50));
        assert!(!info.is_complete());
    }

    #[test]
    fn test_progress_complete() {
        let info = ProgressInfo::completed(100);
        assert!(info.is_complete());
        assert_eq!(info.percent(), 100.0);
    }

    #[test]
    fn test_progress_unknown_total() {
        let info = ProgressInfo::new(50, None);
        assert_eq!(info.percent(), 0.0);
        assert!(info.fraction().is_none());
        assert!(info.remaining().is_none());
    }

    #[test]
    fn test_progress_callback() {
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = counter.clone();

        let progress = Progress::new().on_download(move |info| {
            counter_clone.store(info.loaded, Ordering::SeqCst);
        });

        progress.notify_download(ProgressInfo::new(42, Some(100)));
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }
}

