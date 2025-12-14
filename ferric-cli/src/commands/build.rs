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

    // Apply cache busting if enabled
    if config.build.cache_busting.enabled && release {
        println!("  {} Applying cache busting...", style("→").cyan());
        apply_cache_busting(config)?;
    }

    println!("    {} Browser build complete", style("✓").green());
    Ok(())
}

/// Apply cache busting to build artifacts
fn apply_cache_busting(config: &FerricConfig) -> Result<()> {
    use crate::cache_busting::{process_directory, Strategy, update_html_references};
    use std::path::Path;

    let cb_config = &config.build.cache_busting;
    let strategy = Strategy::from_str(&cb_config.strategy);
    let out_dir = Path::new(&config.build.out_dir);

    let mut all_mappings = std::collections::HashMap::new();

    // Process pkg directory (wasm + js files)
    let pkg_dir = out_dir.join("pkg");
    if pkg_dir.exists() {
        println!("    {} Processing pkg/ directory...", style("→").cyan());
        let pkg_mappings = process_directory(
            &pkg_dir,
            strategy,
            cb_config.hash_length,
            cb_config.css,
            cb_config.js,
            cb_config.assets,
        )?;
        println!(
            "      {} Hashed {} files in pkg/",
            style("✓").green(),
            pkg_mappings.len()
        );
        all_mappings.extend(pkg_mappings);
    }

    // Process styles in root output directory
    let css_files = ["styles.css", "main.css", "app.css"];
    for css_file in &css_files {
        let css_path = out_dir.join(css_file);
        if css_path.exists() && cb_config.css {
            match crate::cache_busting::apply_cache_busting(&css_path, strategy, cb_config.hash_length) {
                Ok((new_path, original_name)) => {
                    let new_name = new_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    all_mappings.insert(original_name, new_name);
                    println!(
                        "      {} Hashed {}",
                        style("✓").green(),
                        css_file
                    );
                }
                Err(e) => {
                    println!(
                        "      {} Failed to hash {}: {}",
                        style("!").yellow(),
                        css_file,
                        e
                    );
                }
            }
        }
    }

    // Process styles directory if it exists
    let styles_dir = out_dir.join("styles");
    if styles_dir.exists() {
        println!("    {} Processing styles/ directory...", style("→").cyan());
        let style_mappings = process_directory(
            &styles_dir,
            strategy,
            cb_config.hash_length,
            cb_config.css,
            false, // Don't reprocess js in styles dir
            cb_config.assets,
        )?;
        println!(
            "      {} Hashed {} files in styles/",
            style("✓").green(),
            style_mappings.len()
        );
        all_mappings.extend(style_mappings);
    }

    // Process assets if enabled
    if cb_config.assets {
        let assets_dir = out_dir.join("assets");
        if assets_dir.exists() {
            println!("    {} Processing assets/ directory...", style("→").cyan());
            let asset_mappings = process_directory(
                &assets_dir,
                strategy,
                cb_config.hash_length,
                false,
                false,
                true,
            )?;
            println!(
                "      {} Hashed {} assets",
                style("✓").green(),
                asset_mappings.len()
            );
            all_mappings.extend(asset_mappings);
        }
    }

    // Update HTML file references
    let html_path = out_dir.join("index.html");
    if html_path.exists() && !all_mappings.is_empty() {
        println!("    {} Updating index.html references...", style("→").cyan());
        update_html_references(&html_path, &all_mappings)?;
        println!(
            "    {} Updated {} asset references in index.html",
            style("✓").green(),
            all_mappings.len()
        );

        // Print sample mappings for verification
        if all_mappings.len() <= 5 {
            for (original, hashed) in &all_mappings {
                println!(
                    "      {} {} → {}",
                    style("→").dim(),
                    original,
                    hashed
                );
            }
        } else {
            let mut sample: Vec<_> = all_mappings.iter().take(3).collect();
            sample.sort_by_key(|(k, _)| *k);
            for (original, hashed) in sample {
                println!(
                    "      {} {} → {}",
                    style("→").dim(),
                    original,
                    hashed
                );
            }
            println!(
                "      {} ... and {} more",
                style("→").dim(),
                all_mappings.len() - 3
            );
        }
    } else if all_mappings.is_empty() {
        println!(
            "    {} No files to hash (check build configuration)",
            style("!").yellow()
        );
    } else if !html_path.exists() {
        println!(
            "    {} index.html not found, skipping reference updates",
            style("!").yellow()
        );
    }

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
