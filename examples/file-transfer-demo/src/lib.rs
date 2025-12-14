//! File upload and download tracking demo for Ferric HTTP.

use ferric_http::{
    UploadManager, DownloadManager,
    UploadProgress, DownloadProgress,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"File Transfer Demo Starting...".into());

    // Run examples
    upload_example();
    download_example();
    batch_example();
}

/// Example of uploading a file with progress tracking.
fn upload_example() {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&"Upload Example:".into());
        web_sys::console::log_1(&"Use FileUpload::new(url, filename).file(file).on_progress(callback).send().await".into());

        // Example code (not executable in this context):
        // let upload = FileUpload::new("https://example.com/upload", "document.pdf")
        //     .field_name("document")
        //     .header("Authorization", "Bearer token")
        //     .on_progress(|progress| {
        //         println!("Uploading: {:.1}%", progress.percent());
        //         println!("Speed: {}/s", progress.speed_formatted());
        //         if let Some(eta) = progress.estimated_seconds {
        //             println!("ETA: {:.0} seconds", eta);
        //         }
        //     })
        //     .file(js_file) // web_sys::File from input
        //     .send()
        //     .await?;
    }
}

/// Example of downloading a file with progress tracking.
fn download_example() {
    web_sys::console::log_1(&"Download Example:".into());
    web_sys::console::log_1(&"Use FileDownload::new(url).filename(name).on_progress(callback).download().await".into());

    // Example code (not executable in this context):
    // let download = FileDownload::new("https://example.com/large-file.zip")
    //     .filename("archive.zip")
    //     .on_progress(|progress| {
    //         if let Some(percent) = progress.percent() {
    //             println!("Downloaded: {:.1}%", percent);
    //         }
    //         println!("Downloaded: {} / {}",
    //             progress.downloaded_size_formatted(),
    //             progress.total_size_formatted().unwrap_or("unknown".to_string())
    //         );
    //         println!("Speed: {}", progress.speed_formatted());
    //     })
    //     .download()
    //     .await?;
}

/// Example of managing multiple uploads/downloads.
fn batch_example() {
    web_sys::console::log_1(&"Batch Transfer Example:".into());

    // Upload manager
    let upload_mgr = UploadManager::new();
    web_sys::console::log_1(&format!(
        "Upload Manager created. Active uploads: {}",
        upload_mgr.active_uploads().len()
    ).into());

    // Download manager
    let download_mgr = DownloadManager::new();
    let stats = download_mgr.stats();
    web_sys::console::log_1(&format!(
        "Download Manager created. Active: {}, Total: {}",
        stats.active_count,
        stats.total_count
    ).into());

    // Example: Track multiple files
    // for file in files {
    //     let progress = UploadProgress::new(&file.name, file.size);
    //     upload_mgr.track(progress);
    // }
    //
    // // Get total progress
    // let total = upload_mgr.total_progress();
    // println!("Overall progress: {:.1}%", total);
}

/// Demonstrates upload progress callbacks.
#[wasm_bindgen]
pub fn demo_upload_callback() {
    web_sys::console::log_1(&"Upload Callback Demo:".into());

    // Example callback that logs progress
    let _callback = |progress: &UploadProgress| {
        match &progress.stage {
            ferric_http::UploadStage::Starting => {
                web_sys::console::log_1(&format!("Starting upload: {}", progress.filename).into());
            }
            ferric_http::UploadStage::Reading => {
                web_sys::console::log_1(&"Reading file...".into());
            }
            ferric_http::UploadStage::Uploading => {
                let percent = progress.percent();
                web_sys::console::log_1(&format!(
                    "Uploading: {:.1}% ({} / {} bytes) @ {}",
                    percent,
                    progress.uploaded_bytes,
                    progress.total_bytes,
                    progress.speed_formatted()
                ).into());
            }
            ferric_http::UploadStage::Complete => {
                web_sys::console::log_1(&format!("Upload complete: {}", progress.filename).into());
            }
            ferric_http::UploadStage::Failed(err) => {
                web_sys::console::log_1(&format!("Upload failed: {}", err).into());
            }
            ferric_http::UploadStage::Cancelled => {
                web_sys::console::log_1(&"Upload cancelled".into());
            }
        }
    };
}

/// Demonstrates download progress callbacks.
#[wasm_bindgen]
pub fn demo_download_callback() {
    web_sys::console::log_1(&"Download Callback Demo:".into());

    // Example callback that logs progress
    let _callback = |progress: &DownloadProgress| {
        match &progress.stage {
            ferric_http::DownloadStage::Starting => {
                web_sys::console::log_1(&format!("Starting download: {}", progress.url).into());
            }
            ferric_http::DownloadStage::Headers => {
                web_sys::console::log_1(&format!(
                    "Headers received. Content-Type: {} Size: {}",
                    progress.content_type.as_ref().unwrap_or(&"unknown".to_string()),
                    progress.total_size_formatted().unwrap_or("unknown".to_string())
                ).into());
            }
            ferric_http::DownloadStage::Downloading => {
                if let Some(percent) = progress.percent() {
                    web_sys::console::log_1(&format!(
                        "Downloading: {:.1}% ({} / {}) @ {}",
                        percent,
                        progress.downloaded_size_formatted(),
                        progress.total_size_formatted().unwrap_or("unknown".to_string()),
                        progress.speed_formatted()
                    ).into());
                }
            }
            ferric_http::DownloadStage::Complete => {
                web_sys::console::log_1(&"Download complete".into());
            }
            ferric_http::DownloadStage::Failed(err) => {
                web_sys::console::log_1(&format!("Download failed: {}", err).into());
            }
            ferric_http::DownloadStage::Cancelled => {
                web_sys::console::log_1(&"Download cancelled".into());
            }
        }
    };
}

