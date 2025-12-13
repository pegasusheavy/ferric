//! `ferric info` command - Show project information

use crate::config::FerricConfig;
use anyhow::Result;
use console::style;
use std::path::Path;
use std::process::Command;

/// Run the `info` command
pub async fn run() -> Result<()> {
    println!(
        "{} Project Information",
        style("Ferric").cyan().bold()
    );
    println!();

    // Load project config
    let config = FerricConfig::load();

    if let Ok(config) = config {
        println!("  {} {}", style("Name:").bold(), config.project.name);
        println!("  {} {}", style("Version:").bold(), config.project.version);

        if !config.project.description.is_empty() {
            println!("  {} {}", style("Description:").bold(), config.project.description);
        }

        println!();
        println!("  {}", style("Build Configuration:").bold().underlined());
        println!("    Output directory: {}", config.build.out_dir);
        println!("    Source directory: {}", config.build.src_dir);
        println!(
            "    SSR enabled: {}",
            if config.ssr.enabled {
                style("Yes").green()
            } else {
                style("No").dim()
            }
        );
        println!(
            "    Style preprocessor: {}",
            config.styles.preprocessor
        );
    } else {
        println!(
            "  {} No ferric.toml found",
            style("!").yellow()
        );
    }

    println!();
    println!("  {}", style("Environment:").bold().underlined());

    // Rust version
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("    Rust: {}", version.trim());
    }

    // wasm-pack version
    if let Ok(output) = Command::new("wasm-pack").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("    wasm-pack: {}", version.trim());
    } else {
        println!(
            "    wasm-pack: {}",
            style("not installed").red()
        );
    }

    // Check for Cargo.toml
    if Path::new("Cargo.toml").exists() {
        println!("    Cargo.toml: {}", style("✓").green());
    }

    // Check for ferric.toml
    if Path::new("ferric.toml").exists() {
        println!("    ferric.toml: {}", style("✓").green());
    }

    println!();

    Ok(())
}
