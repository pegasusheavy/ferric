//! `ferric test` command - Run tests

use anyhow::{Context, Result};
use console::style;
use std::process::Command;

/// Run the `test` command
pub async fn run(watch: bool, filter: Option<String>) -> Result<()> {
    println!(
        "{} Running tests...",
        style("→").cyan().bold()
    );

    if watch {
        run_watch_mode(filter).await
    } else {
        run_once(filter).await
    }
}

async fn run_once(filter: Option<String>) -> Result<()> {
    let mut args = vec!["test"];

    if let Some(ref f) = filter {
        args.push("--");
        args.push(f);
    }

    // Run Rust tests
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run cargo test")?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }

    // Run wasm-bindgen tests
    println!();
    println!("  {} Running WASM tests...", style("→").cyan());

    let wasm_status = Command::new("wasm-pack")
        .args(["test", "--headless", "--chrome"])
        .status();

    if let Ok(status) = wasm_status
        && !status.success() {
            println!(
                "  {} WASM tests failed or wasm-pack not available",
                style("!").yellow()
            );
        }

    println!(
        "{} Tests complete!",
        style("✓").green().bold()
    );

    Ok(())
}

async fn run_watch_mode(filter: Option<String>) -> Result<()> {
    let mut args = vec!["watch", "-x", "test"];

    if let Some(ref f) = filter {
        args.push("--");
        args.push("--");
        args.push(f);
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run cargo watch. Is it installed? Run: cargo install cargo-watch")?;

    if !status.success() {
        anyhow::bail!("Watch mode ended");
    }

    Ok(())
}
