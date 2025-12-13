//! `ferric new` command - Create a new Ferric project

use crate::config::FerricConfig;
use crate::templates::project;
use crate::utils::to_snake_case;
use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Run the `new` command
pub async fn run(name: &str, ssr: bool, skip_git: bool, _template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!(
        "{} Creating new Ferric project: {}",
        style("→").cyan().bold(),
        style(name).green()
    );

    // Create directory structure
    create_directories(project_dir, ssr)?;

    // Generate project files
    generate_project_files(project_dir, name, ssr)?;

    // Initialize git repository
    if !skip_git {
        init_git(project_dir)?;
    }

    println!();
    println!(
        "{} Project '{}' created successfully!",
        style("✓").green().bold(),
        name
    );
    println!();
    println!("  Next steps:");
    println!("    cd {}", name);
    println!("    ferric serve");
    println!();

    if ssr {
        println!(
            "  {} SSR is enabled. Use 'ferric serve --ssr' to run with server-side rendering.",
            style("Note:").yellow()
        );
    }

    Ok(())
}

fn create_directories(project_dir: &Path, ssr: bool) -> Result<()> {
    // Angular-style directory structure
    let dirs = vec![
        "src",
        "src/app",           // Main app component (app.rs, app.component.html, app.component.scss)
        "src/components",    // Reusable components
        "src/services",      // Injectable services
        "src/styles",        // Global SCSS partials (_variables.scss, _mixins.scss, etc.)
        "assets",            // Static assets
    ];

    for dir in &dirs {
        fs::create_dir_all(project_dir.join(dir))?;
    }

    if ssr {
        fs::create_dir_all(project_dir.join("src/server"))?;
    }

    Ok(())
}

fn generate_project_files(project_dir: &Path, name: &str, ssr: bool) -> Result<()> {
    let _snake_name = to_snake_case(name);

    // Cargo.toml
    let cargo_toml = if ssr {
        project::cargo_toml_with_ssr(name)
    } else {
        project::cargo_toml(name)
    };
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // ferric.toml
    let ferric_toml = FerricConfig::default_toml(name, ssr);
    fs::write(project_dir.join("ferric.toml"), ferric_toml)?;

    // src/lib.rs
    let lib_rs = if ssr {
        project::lib_rs_with_ssr(name)
    } else {
        project::lib_rs(name)
    };
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // src/app/mod.rs
    fs::write(project_dir.join("src/app/mod.rs"), project::app_mod(name))?;

    // src/app/app.rs
    fs::write(project_dir.join("src/app/app.rs"), project::app_component())?;

    // src/app/app.component.html (Angular-style template file)
    fs::write(project_dir.join("src/app/app.component.html"), project::app_html(name))?;

    // src/app/app.component.scss (Angular-style component styles)
    fs::write(project_dir.join("src/app/app.component.scss"), project::app_component_scss())?;

    // src/styles.scss (Root stylesheet - Angular-style entry point)
    fs::write(project_dir.join("src/styles.scss"), project::root_styles_scss())?;

    // src/styles/_variables.scss (SCSS variables partial)
    fs::write(project_dir.join("src/styles/_variables.scss"), project::variables_scss())?;

    // index.html
    fs::write(project_dir.join("index.html"), project::index_html(name))?;

    // .gitignore
    fs::write(project_dir.join(".gitignore"), project::gitignore())?;

    // README.md
    fs::write(project_dir.join("README.md"), project::readme(name))?;

    // SSR-specific files
    if ssr {
        // src/server/mod.rs
        fs::write(
            project_dir.join("src/server/mod.rs"),
            project::server_mod(),
        )?;

        // src/server/main.rs
        fs::write(
            project_dir.join("src/server/main.rs"),
            project::server_main(name),
        )?;

        // src/server/routes.rs
        fs::write(
            project_dir.join("src/server/routes.rs"),
            project::server_routes(name),
        )?;
    }

    println!("  {} Generated project files", style("✓").green());

    Ok(())
}

fn init_git(project_dir: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["init"])
        .current_dir(project_dir)
        .output()
        .context("Failed to run git init")?;

    if status.status.success() {
        println!("  {} Initialized git repository", style("✓").green());
    }

    Ok(())
}
