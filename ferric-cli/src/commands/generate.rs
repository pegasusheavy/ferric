//! `ferric generate` command - Generate components, services, etc.

use crate::templates::{component, directive, guard, pipe, service};
use crate::utils::{to_kebab_case, to_pascal_case, to_snake_case};
use anyhow::Result;
use console::style;
use std::fs;
use std::path::Path;

/// Generate a new component
pub async fn component(name: &str, inline_template: bool, inline_style: bool) -> Result<()> {
    let kebab_name = to_kebab_case(name);
    let pascal_name = to_pascal_case(name);
    let snake_name = to_snake_case(name);

    let component_dir = Path::new("src/components").join(&kebab_name);
    fs::create_dir_all(&component_dir)?;

    // Generate component file
    let component_rs = component::component_rs(&pascal_name, &kebab_name, inline_template, inline_style);
    fs::write(component_dir.join(format!("{}.rs", snake_name)), component_rs)?;

    // Generate mod.rs
    let mod_rs = component::mod_rs(&snake_name, &pascal_name);
    fs::write(component_dir.join("mod.rs"), mod_rs)?;

    // Generate template if not inline
    if !inline_template {
        let template_dir = Path::new("templates/components");
        fs::create_dir_all(template_dir)?;
        let template = component::template(&pascal_name);
        fs::write(template_dir.join(format!("{}.html", kebab_name)), template)?;
    }

    // Generate styles if not inline
    if !inline_style {
        let style_dir = Path::new("src/styles/components");
        fs::create_dir_all(style_dir)?;
        let styles = component::styles(&kebab_name);
        fs::write(style_dir.join(format!("{}.scss", kebab_name)), styles)?;
    }

    println!(
        "{} Generated component: {}",
        style("✓").green().bold(),
        pascal_name
    );
    println!("  → src/components/{}/", kebab_name);

    Ok(())
}

/// Generate a new service
pub async fn service(name: &str) -> Result<()> {
    let pascal_name = to_pascal_case(name);
    let snake_name = to_snake_case(name);
    let kebab_name = to_kebab_case(name);

    let services_dir = Path::new("src/services");
    fs::create_dir_all(services_dir)?;

    let service_rs = service::service_rs(&pascal_name);
    fs::write(services_dir.join(format!("{}.rs", snake_name)), service_rs)?;

    // Update mod.rs if it exists
    update_mod_rs(services_dir, &snake_name, &pascal_name)?;

    println!(
        "{} Generated service: {}",
        style("✓").green().bold(),
        pascal_name
    );
    println!("  → src/services/{}.rs", snake_name);

    Ok(())
}

/// Generate a new directive
pub async fn directive(name: &str) -> Result<()> {
    let pascal_name = to_pascal_case(name);
    let snake_name = to_snake_case(name);

    let directives_dir = Path::new("src/directives");
    fs::create_dir_all(directives_dir)?;

    let directive_rs = directive::directive_rs(&pascal_name);
    fs::write(directives_dir.join(format!("{}.rs", snake_name)), directive_rs)?;

    update_mod_rs(directives_dir, &snake_name, &pascal_name)?;

    println!(
        "{} Generated directive: {}",
        style("✓").green().bold(),
        pascal_name
    );
    println!("  → src/directives/{}.rs", snake_name);

    Ok(())
}

/// Generate a new guard
pub async fn guard(name: &str) -> Result<()> {
    let pascal_name = to_pascal_case(name);
    let snake_name = to_snake_case(name);

    let guards_dir = Path::new("src/guards");
    fs::create_dir_all(guards_dir)?;

    let guard_rs = guard::guard_rs(&pascal_name);
    fs::write(guards_dir.join(format!("{}.rs", snake_name)), guard_rs)?;

    update_mod_rs(guards_dir, &snake_name, &pascal_name)?;

    println!(
        "{} Generated guard: {}",
        style("✓").green().bold(),
        pascal_name
    );
    println!("  → src/guards/{}.rs", snake_name);

    Ok(())
}

/// Generate a new pipe
pub async fn pipe(name: &str) -> Result<()> {
    let pascal_name = to_pascal_case(name);
    let snake_name = to_snake_case(name);

    let pipes_dir = Path::new("src/pipes");
    fs::create_dir_all(pipes_dir)?;

    let pipe_rs = pipe::pipe_rs(&pascal_name);
    fs::write(pipes_dir.join(format!("{}.rs", snake_name)), pipe_rs)?;

    update_mod_rs(pipes_dir, &snake_name, &pascal_name)?;

    println!(
        "{} Generated pipe: {}",
        style("✓").green().bold(),
        pascal_name
    );
    println!("  → src/pipes/{}.rs", snake_name);

    Ok(())
}

fn update_mod_rs(dir: &Path, module_name: &str, type_name: &str) -> Result<()> {
    let mod_path = dir.join("mod.rs");

    let new_content = if mod_path.exists() {
        let existing = fs::read_to_string(&mod_path)?;
        format!(
            "{}mod {};\npub use {}::{};\n",
            existing, module_name, module_name, type_name
        )
    } else {
        format!(
            "mod {};\npub use {}::{};\n",
            module_name, module_name, type_name
        )
    };

    fs::write(mod_path, new_content)?;
    Ok(())
}
