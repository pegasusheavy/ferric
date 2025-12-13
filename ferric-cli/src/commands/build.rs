//! `ferric build` command - Build the project

use crate::config::FerricConfig;
use crate::scss;
use crate::tailwind;
use anyhow::{Context, Result};
use console::style;
use std::process::Command;

/// Run the `build` command
pub async fn run(release: bool, target: &str, watch: bool) -> Result<()> {
    let config = FerricConfig::load()?;

    println!(
        "{} Building {} in {} mode...",
        style("→").cyan().bold(),
        style(&config.project.name).green(),
        if release { "release" } else { "debug" }
    );

    // Compile styles based on preprocessor
    println!("  {} Compiling styles...", style("→").cyan());

    if config.uses_tailwind() || tailwind::is_tailwind_available() {
        // Use TailwindCSS via PostCSS
        tailwind::build_project_styles(&config, release).await?;
    } else if config.styles.preprocessor == "scss" || config.styles.preprocessor == "sass" {
        // Use SCSS/SASS compilation
        scss::build_project_styles(&config, release).await?;
    } else if config.styles.preprocessor == "css" {
        // Plain CSS - just copy to output
        let src = std::path::Path::new(&config.styles.root);
        if src.exists() {
            let out_dir = std::path::Path::new(&config.build.out_dir);
            std::fs::create_dir_all(out_dir)?;
            let dest = out_dir.join(&config.styles.output);
            std::fs::copy(src, &dest)?;
            println!(
                "  {} Copied {} → {}",
                style("✓").green(),
                src.display(),
                dest.display()
            );
        }
    }

    // Build based on target
    match target {
        "browser" => build_browser(&config, release, watch).await?,
        "server" => build_server(&config, release).await?,
        "universal" => {
            build_browser(&config, release, false).await?;
            build_server(&config, release).await?;
        }
        _ => anyhow::bail!(
            "Unknown target: {}. Use 'browser', 'server', or 'universal'",
            target
        ),
    }

    println!("{} Build complete!", style("✓").green().bold());

    Ok(())
}

async fn build_browser(config: &FerricConfig, release: bool, watch: bool) -> Result<()> {
    println!("  {} Building browser target...", style("→").cyan());

    let mut args = vec!["build", "--target", "web"];

    if release {
        args.push("--release");
    }

    let out_dir = format!("--out-dir={}/pkg", config.build.out_dir);
    args.push(&out_dir);

    if watch {
        // Use cargo watch for development
        let status = Command::new("cargo")
            .args([
                "watch",
                "-x",
                &format!("run -p wasm-pack -- {}", args.join(" ")),
            ])
            .status()
            .context("Failed to run cargo watch")?;

        if !status.success() {
            anyhow::bail!("Watch build failed");
        }
    } else {
        let status = Command::new("wasm-pack")
            .args(&args)
            .status()
            .context("Failed to run wasm-pack. Is it installed? Run: cargo install wasm-pack")?;

        if !status.success() {
            anyhow::bail!("wasm-pack build failed");
        }
    }

    // Copy index.html to output
    if std::path::Path::new("index.html").exists() {
        std::fs::copy(
            "index.html",
            format!("{}/index.html", config.build.out_dir),
        )?;
    }

    println!("    {} Browser build complete", style("✓").green());
    Ok(())
}

async fn build_server(config: &FerricConfig, release: bool) -> Result<()> {
    if !config.ssr.enabled {
        println!(
            "    {} SSR not enabled in ferric.toml, skipping server build",
            style("!").yellow()
        );
        return Ok(());
    }

    println!("  {} Building server target...", style("→").cyan());

    let mut args = vec!["build", "--bin", "server"];

    if release {
        args.push("--release");
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to build server")?;

    if !status.success() {
        anyhow::bail!("Server build failed");
    }

    println!("    {} Server build complete", style("✓").green());
    Ok(())
}
