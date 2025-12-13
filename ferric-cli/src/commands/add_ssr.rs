//! `ferric add-ssr` command - Add SSR support to existing project

use crate::config::FerricConfig;
use crate::templates::project;
use anyhow::Result;
use console::style;
use std::fs;
use std::path::Path;

/// Run the `add-ssr` command
pub async fn run() -> Result<()> {
    // Load existing config
    let mut config = FerricConfig::load()?;

    if config.ssr.enabled {
        println!(
            "{} SSR is already enabled in this project",
            style("!").yellow().bold()
        );
        return Ok(());
    }

    println!(
        "{} Adding SSR support to project...",
        style("→").cyan().bold()
    );

    // Create server directory
    let server_dir = Path::new("src/server");
    fs::create_dir_all(server_dir)?;

    // Generate SSR files
    fs::write(server_dir.join("mod.rs"), project::server_mod())?;
    fs::write(
        server_dir.join("main.rs"),
        project::server_main(&config.project.name),
    )?;
    fs::write(
        server_dir.join("routes.rs"),
        project::server_routes(&config.project.name),
    )?;

    println!("  {} Generated server files", style("✓").green());

    // Update ferric.toml
    config.ssr.enabled = true;
    config.build.targets.server = true;
    config.save()?;

    println!("  {} Updated ferric.toml", style("✓").green());

    // Update Cargo.toml to add SSR dependencies
    update_cargo_toml()?;

    println!("  {} Updated Cargo.toml", style("✓").green());

    println!();
    println!(
        "{} SSR support added successfully!",
        style("✓").green().bold()
    );
    println!();
    println!("  Next steps:");
    println!("    1. Run 'cargo build' to compile the server");
    println!("    2. Use 'ferric serve --ssr' to run with SSR");
    println!("    3. Or build with 'ferric build --target=universal'");
    println!();

    Ok(())
}

fn update_cargo_toml() -> Result<()> {
    let cargo_path = Path::new("Cargo.toml");

    if !cargo_path.exists() {
        anyhow::bail!("Cargo.toml not found. Are you in a Ferric project?");
    }

    let content = fs::read_to_string(cargo_path)?;

    // Check if ferric-ssr is already a dependency
    if content.contains("ferric-ssr") {
        return Ok(());
    }

    // Add SSR dependencies
    let ssr_deps = r#"
# SSR dependencies
ferric-ssr = { git = "https://github.com/pegasusheavy/ferric.git" }
tokio = { version = "1.35", features = ["full"] }
hyper = { version = "1.5", features = ["full"] }

[[bin]]
name = "server"
path = "src/server/main.rs"
"#;

    let updated = if content.contains("[dev-dependencies]") {
        content.replace("[dev-dependencies]", &format!("{}\n[dev-dependencies]", ssr_deps))
    } else {
        format!("{}\n{}", content, ssr_deps)
    };

    fs::write(cargo_path, updated)?;

    Ok(())
}
