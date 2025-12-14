//! SCSS compilation module
//!
//! Provides SCSS to CSS compilation similar to Angular's build system.

use crate::config::FerricConfig;
use anyhow::{Context, Result};
use console::style;
use grass::{Options, OutputStyle};
use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use walkdir::WalkDir;

/// Compile SCSS file to CSS
pub fn compile_scss(
    input: &Path,
    output: &Path,
    options: &ScssOptions,
) -> Result<String> {
    // Debug: print include paths
    if std::env::var("FERRIC_DEBUG").is_ok() {
        eprintln!("SCSS include paths:");
        for p in &options.include_paths {
            eprintln!("  - {}", p.display());
        }
    }

    let scss_options = Options::default()
        .style(if options.minify {
            OutputStyle::Compressed
        } else {
            OutputStyle::Expanded
        })
        .load_paths(&options.include_paths);

    let css = grass::from_path(input, &scss_options)
        .map_err(|e| {
            anyhow::anyhow!(
                "SCSS compilation error in {}:\n  {}",
                input.display(),
                e
            )
        })?;

    // Write output if path is provided
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, &css)?;

    Ok(css)
}

/// Compile SCSS string to CSS
pub fn compile_scss_string(scss: &str, options: &ScssOptions) -> Result<String> {
    let scss_options = Options::default()
        .style(if options.minify {
            OutputStyle::Compressed
        } else {
            OutputStyle::Expanded
        })
        .load_paths(&options.include_paths);

    let css = grass::from_string(scss, &scss_options)
        .with_context(|| "Failed to compile SCSS string")?;

    Ok(css)
}

/// SCSS compilation options
#[derive(Debug, Clone, Default)]
pub struct ScssOptions {
    /// Minify output CSS
    pub minify: bool,
    /// Include paths for @import resolution
    pub include_paths: Vec<PathBuf>,
    /// Generate source maps
    pub source_maps: bool,
}

impl ScssOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn minify(mut self, minify: bool) -> Self {
        self.minify = minify;
        self
    }

    pub fn include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    pub fn include_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.include_paths = paths;
        self
    }

    pub fn source_maps(mut self, enabled: bool) -> Self {
        self.source_maps = enabled;
        self
    }

    /// Create options from ferric.toml configuration
    pub fn from_config(config: &FerricConfig) -> Self {
        let minify = config.styles.output_style == "compressed";
        let include_paths: Vec<PathBuf> = config
            .styles
            .include_paths
            .paths
            .iter()
            .map(PathBuf::from)
            .collect();

        Self {
            minify,
            include_paths,
            source_maps: config.build.source_maps,
        }
    }
}

/// Compile all SCSS files in a directory
pub fn compile_directory(
    input_dir: &Path,
    output_dir: &Path,
    options: &ScssOptions,
) -> Result<Vec<CompiledFile>> {
    let mut compiled = Vec::new();

    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "scss" || ext == "sass")
                .unwrap_or(false)
        })
        .filter(|e| {
            // Skip partials (files starting with _)
            !e.file_name()
                .to_str()
                .map(|s| s.starts_with('_'))
                .unwrap_or(false)
        })
    {
        let input_path = entry.path();
        let relative = input_path.strip_prefix(input_dir)?;
        let output_path = output_dir.join(relative).with_extension("css");

        match compile_scss(input_path, &output_path, options) {
            Ok(css) => {
                compiled.push(CompiledFile {
                    input: input_path.to_path_buf(),
                    output: output_path,
                    size: css.len(),
                });
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to compile {}: {}",
                    style("Error:").red().bold(),
                    input_path.display(),
                    e
                );
            }
        }
    }

    Ok(compiled)
}

/// Information about a compiled file
#[derive(Debug)]
pub struct CompiledFile {
    pub input: PathBuf,
    pub output: PathBuf,
    pub size: usize,
}

/// Watch SCSS files for changes and recompile
pub async fn watch_and_compile(
    input: &Path,
    output_dir: &Path,
    options: &ScssOptions,
) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    let watch_dir = if input.is_dir() {
        input
    } else {
        input.parent().unwrap_or(Path::new("."))
    };

    watcher.watch(watch_dir, RecursiveMode::Recursive)?;

    println!(
        "{} Watching for SCSS changes in {}",
        style("Watch:").cyan().bold(),
        watch_dir.display()
    );

    // Initial compilation
    if input.is_dir() {
        compile_directory(input, output_dir, options)?;
    } else {
        let output_path = output_dir.join(input.file_name().unwrap()).with_extension("css");
        compile_scss(input, &output_path, options)?;
    }

    // Watch for changes
    loop {
        match rx.recv() {
            Ok(event) => {
                for path in event.paths {
                    if let Some(ext) = path.extension()
                        && (ext == "scss" || ext == "sass") {
                            println!(
                                "{} {}",
                                style("Changed:").yellow(),
                                path.display()
                            );

                            // Recompile the changed file or directory
                            if input.is_dir() {
                                if let Err(e) = compile_directory(input, output_dir, options) {
                                    eprintln!("{} {}", style("Error:").red().bold(), e);
                                }
                            } else {
                                let output_path = output_dir
                                    .join(path.file_name().unwrap())
                                    .with_extension("css");
                                if let Err(e) = compile_scss(&path, &output_path, options) {
                                    eprintln!("{} {}", style("Error:").red().bold(), e);
                                }
                            }

                            println!(
                                "{} Compilation complete",
                                style("Done:").green().bold()
                            );
                        }
                }
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// CLI command handler for SCSS compilation
pub async fn compile_command(
    input: &str,
    output: Option<&str>,
    watch: bool,
    minify: bool,
) -> Result<()> {
    let input_path = Path::new(input);

    // Load config if available
    let config = FerricConfig::load().unwrap_or_default();
    let mut options = ScssOptions::from_config(&config);

    // Override with CLI options
    if minify {
        options.minify = true;
    }

    // Determine output path
    let output_path = match output {
        Some(o) => PathBuf::from(o),
        None => {
            if input_path.is_dir() {
                PathBuf::from(&config.build.out_dir).join("styles")
            } else {
                input_path.with_extension("css")
            }
        }
    };

    if watch {
        let output_dir = if output_path.is_dir() || input_path.is_dir() {
            output_path
        } else {
            output_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        watch_and_compile(input_path, &output_dir, &options).await
    } else if input_path.is_dir() {
        let compiled = compile_directory(input_path, &output_path, &options)?;
        println!(
            "{} Compiled {} SCSS files",
            style("Success:").green().bold(),
            compiled.len()
        );
        for file in &compiled {
            println!(
                "  {} -> {} ({} bytes)",
                file.input.display(),
                file.output.display(),
                file.size
            );
        }
        Ok(())
    } else {
        compile_scss(input_path, &output_path, &options)?;
        println!(
            "{} {} -> {}",
            style("Compiled:").green().bold(),
            input_path.display(),
            output_path.display()
        );
        Ok(())
    }
}

/// Build all styles for a project according to ferric.toml
///
/// This follows Angular's pattern where:
/// 1. A root stylesheet (src/styles.scss) is the main entry point
/// 2. Component styles can be bundled automatically
/// 3. Output is a single CSS file in the dist directory
pub async fn build_project_styles(config: &FerricConfig, release: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Create SCSS options with minification based on release mode or config
    let minify = release || config.styles.output_style == "compressed";
    let mut options = ScssOptions {
        minify,
        include_paths: Vec::new(),
        source_maps: config.build.source_maps,
    };

    // Start with absolute path to src directory for @use/@import resolution
    let src_dir = cwd.join(&config.build.src_dir);
    options.include_paths.push(src_dir.clone());

    // Add any configured include paths (convert to absolute, skip duplicates)
    for path in &config.styles.include_paths.paths {
        let abs_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            cwd.join(path)
        };
        if !options.include_paths.contains(&abs_path) {
            options.include_paths.push(abs_path);
        }
    }

    let output_dir = cwd.join(&config.build.out_dir);
    std::fs::create_dir_all(&output_dir)?;

    let root_stylesheet = cwd.join(&config.styles.root);

    if root_stylesheet.exists() {
        // Compile root stylesheet to output directory
        let output_path = output_dir.join(&config.styles.output);

        compile_scss(&root_stylesheet, &output_path, &options)?;

        let size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        println!(
            "  {} {} → {} ({})",
            style("✓").green(),
            config.styles.root,
            output_path.strip_prefix(&cwd).unwrap_or(&output_path).display(),
            format_size(size)
        );
    } else {
        // Fallback: compile global styles individually (legacy behavior)
        println!(
            "  {} Root stylesheet not found at {}, using legacy global styles",
            style("!").yellow(),
            config.styles.root
        );

        for global_style in &config.styles.global {
            let input_path = cwd.join(global_style);
            if input_path.exists() {
                let output_path = output_dir.join(
                    Path::new(global_style)
                        .file_name()
                        .unwrap_or_default()
                ).with_extension("css");

                compile_scss(&input_path, &output_path, &options)?;
                println!(
                    "  {} {}",
                    style("✓").green(),
                    global_style
                );
            }
        }
    }

    Ok(())
}

/// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Generate a root stylesheet that imports all component styles
///
/// This scans the src directory for component SCSS files and generates
/// @use statements for each one.
pub fn generate_root_stylesheet(config: &FerricConfig) -> Result<String> {
    let src_dir = Path::new(&config.build.src_dir);
    let mut imports = Vec::new();

    // Add styles directory partials first
    let styles_dir = src_dir.join("styles");
    if styles_dir.exists() {
        imports.push("// Core styles".to_string());
        imports.push("@use 'styles/variables' as *;".to_string());
        imports.push("@use 'styles/mixins' as *;".to_string());
        imports.push("".to_string());
    }

    // Find all component .scss files (non-partials)
    if src_dir.exists() {
        let mut component_files: Vec<PathBuf> = WalkDir::new(src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "scss" || ext == "sass")
                    .unwrap_or(false)
            })
            .filter(|e| {
                let filename = e.file_name().to_str().unwrap_or("");
                // Include component files, exclude partials and root
                !filename.starts_with('_') &&
                filename != "styles.scss" &&
                filename.ends_with(".component.scss")
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        component_files.sort();

        if !component_files.is_empty() {
            imports.push("// Component styles".to_string());

            for file in component_files {
                if let Ok(relative) = file.strip_prefix(src_dir) {
                    // Convert path to SCSS @use format
                    let module_path = relative
                        .with_extension("")
                        .to_string_lossy()
                        .replace('\\', "/");
                    imports.push(format!("@use '{}';", module_path));
                }
            }
        }
    }

    Ok(imports.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_scss_string() {
        let scss = ".foo { .bar { color: red; } }";
        let options = ScssOptions::new();
        let css = compile_scss_string(scss, &options).unwrap();
        assert!(css.contains(".foo .bar"));
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_compile_scss_string_minified() {
        let scss = ".foo { color: red; }";
        let options = ScssOptions::new().minify(true);
        let css = compile_scss_string(scss, &options).unwrap();
        // Compressed output should not have extra whitespace
        assert!(!css.contains("  "));
    }

    #[test]
    fn test_scss_options_from_config() {
        let config = FerricConfig::default();
        let options = ScssOptions::from_config(&config);
        assert!(!options.minify); // default is expanded
    }
}
