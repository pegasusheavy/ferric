//! `ferric serve` command - Start development server
//!
//! Uses the Armature HTTP framework for serving static files
//! with hot reload support.

use crate::config::FerricConfig;
use crate::scss;
use crate::server::{self, DevServerConfig};
use crate::tailwind;
use anyhow::Result;
use console::style;
use notify::{RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;

/// Run the `serve` command
pub async fn run(port: u16, host: &str, open: bool, ssr: bool) -> Result<()> {
    let config = FerricConfig::load()?;

    let port = if port != 3000 { port } else { config.serve.port };
    let host = if host != "127.0.0.1" {
        host.to_string()
    } else {
        config.serve.host.clone()
    };

    println!(
        "{} Preparing development server...",
        style("→").cyan().bold()
    );

    // Determine style compilation mode
    let use_tailwind = tailwind::is_tailwind_available();

    // Build initial assets
    println!("  {} Compiling styles...", style("→").cyan());
    if use_tailwind {
        tailwind::build_project_styles(&config, false).await?;
    } else if config.styles.preprocessor == "scss" || config.styles.preprocessor == "sass" {
        scss::build_project_styles(&config, false).await?;
    }

    // Start watcher for style files in a separate thread
    let style_config = config.clone();
    let use_tailwind_watcher = use_tailwind;
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })
        .expect("Failed to create style watcher");

        // Watch src directory for style changes
        let _ = watcher.watch(Path::new("src"), RecursiveMode::Recursive);
        // Also watch index.html for Tailwind class changes
        let _ = watcher.watch(Path::new("index.html"), RecursiveMode::NonRecursive);

        // Debounce timer
        let mut last_compile = std::time::Instant::now();
        let debounce_duration = std::time::Duration::from_millis(150);

        loop {
            if let Ok(event) = rx.recv() {
                for path in event.paths {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_str().unwrap_or("");

                        // Check if we should recompile
                        let should_compile = if use_tailwind_watcher {
                            // For TailwindCSS, watch .css, .html, and .rs files
                            matches!(ext_str, "css" | "html" | "rs")
                        } else {
                            // For SCSS, watch .scss and .sass files
                            matches!(ext_str, "scss" | "sass")
                        };

                        // Debounce rapid changes
                        if should_compile && last_compile.elapsed() > debounce_duration {
                            last_compile = std::time::Instant::now();

                            println!(
                                "  {} Style source changed: {}",
                                style("↻").yellow(),
                                path.display()
                            );

                            let rt = tokio::runtime::Runtime::new().unwrap();
                            if use_tailwind_watcher {
                                let _ =
                                    rt.block_on(tailwind::build_project_styles(&style_config, false));
                            } else {
                                let _ =
                                    rt.block_on(scss::build_project_styles(&style_config, false));
                            }
                        }
                    }
                }
            }
        }
    });

    // Create Armature-based dev server configuration
    let mut dev_config = DevServerConfig::from_ferric_config(&config);
    dev_config.port = port;
    dev_config.host = host;
    dev_config.open_browser = open || config.serve.open;
    dev_config.ssr = ssr || config.ssr.enabled;

    if ssr {
        println!(
            "  {} SSR mode enabled - server will handle rendering",
            style("⚡").magenta()
        );
    }

    // Start the Armature-based development server
    server::start_dev_server(dev_config).await
}
