//! `ferric clean` command - Clean build artifacts

use crate::config::FerricConfig;
use anyhow::Result;
use console::style;
use std::fs;
use std::path::Path;

/// Run the `clean` command
pub async fn run(all: bool) -> Result<()> {
    let config = FerricConfig::load().unwrap_or_default();

    println!(
        "{} Cleaning build artifacts...",
        style("→").cyan().bold()
    );

    // Clean dist directory
    let out_dir = Path::new(&config.build.out_dir);
    if out_dir.exists() {
        fs::remove_dir_all(out_dir)?;
        println!("  {} Removed {}/", style("✓").green(), config.build.out_dir);
    }

    // Clean target directory
    let target_dir = Path::new("target");
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
        println!("  {} Removed target/", style("✓").green());
    }

    // Clean pkg directory (wasm-pack output)
    let pkg_dir = Path::new("pkg");
    if pkg_dir.exists() {
        fs::remove_dir_all(pkg_dir)?;
        println!("  {} Removed pkg/", style("✓").green());
    }

    if all {
        // Clean node_modules if requested
        let node_modules = Path::new("node_modules");
        if node_modules.exists() {
            fs::remove_dir_all(node_modules)?;
            println!("  {} Removed node_modules/", style("✓").green());
        }
    }

    println!(
        "{} Clean complete!",
        style("✓").green().bold()
    );

    Ok(())
}
