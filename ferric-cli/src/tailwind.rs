//! TailwindCSS integration module
//!
//! Provides TailwindCSS compilation support for Ferric projects using PostCSS.
//! This module invokes Tailwind via PostCSS programmatically, avoiding the need
//! for @tailwindcss/cli.

use crate::config::FerricConfig;
use anyhow::{Context, Result};
use console::style;
use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

/// TailwindCSS compilation options
#[derive(Debug, Clone)]
pub struct TailwindOptions {
    /// Input CSS file (with @tailwind directives or @import "tailwindcss")
    pub input: PathBuf,
    /// Output CSS file
    pub output: PathBuf,
    /// Minify output using cssnano or lightningcss
    pub minify: bool,
    /// Watch for changes
    pub watch: bool,
    /// Content paths to scan for class usage
    pub content: Vec<String>,
    /// Working directory
    pub cwd: PathBuf,
    /// Enable autoprefixer
    pub autoprefixer: bool,
    /// PostCSS plugins to include
    pub postcss_plugins: Vec<String>,
}

impl Default for TailwindOptions {
    fn default() -> Self {
        Self {
            input: PathBuf::from("src/styles.css"),
            output: PathBuf::from("dist/styles.css"),
            minify: false,
            watch: false,
            content: vec![
                "./src/**/*.rs".to_string(),
                "./src/**/*.html".to_string(),
                "./**/*.html".to_string(),
            ],
            cwd: std::env::current_dir().unwrap_or_default(),
            autoprefixer: true,
            postcss_plugins: Vec::new(),
        }
    }
}

impl TailwindOptions {
    /// Create options from ferric.toml configuration
    pub fn from_config(config: &FerricConfig) -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();

        Self {
            input: cwd.join(&config.styles.root),
            output: cwd.join(&config.build.out_dir).join(&config.styles.output),
            minify: config.styles.output_style == "compressed",
            watch: false,
            content: vec![
                format!("./{src}/**/*.rs", src = config.build.src_dir),
                format!("./{src}/**/*.html", src = config.build.src_dir),
                "./**/*.html".to_string(),
            ],
            cwd,
            autoprefixer: !config.styles.autoprefixer.is_empty(),
            postcss_plugins: Vec::new(),
        }
    }

    pub fn minify(mut self, minify: bool) -> Self {
        self.minify = minify;
        self
    }

    pub fn watch(mut self, watch: bool) -> Self {
        self.watch = watch;
        self
    }

    pub fn autoprefixer(mut self, enabled: bool) -> Self {
        self.autoprefixer = enabled;
        self
    }
}

/// Check if TailwindCSS is available in the project
pub fn is_tailwind_available() -> bool {
    // Check if package.json exists and has tailwindcss
    if let Ok(content) = std::fs::read_to_string("package.json") {
        if content.contains("tailwindcss") {
            return true;
        }
    }

    // Check for tailwind.config.js/ts/mjs
    Path::new("tailwind.config.js").exists()
        || Path::new("tailwind.config.ts").exists()
        || Path::new("tailwind.config.mjs").exists()
}

/// Detect the package manager to use (pnpm, npm, yarn, bun)
pub fn detect_package_manager() -> &'static str {
    if Path::new("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if Path::new("yarn.lock").exists() {
        "yarn"
    } else if Path::new("bun.lockb").exists() {
        "bun"
    } else {
        "npm"
    }
}

/// Generate the PostCSS build script that uses Tailwind directly
fn generate_postcss_script(options: &TailwindOptions) -> String {
    let input_path = options.input.to_string_lossy().replace('\\', "/");
    let output_path = options.output.to_string_lossy().replace('\\', "/");

    let minify_plugin = if options.minify {
        r#"
    // Minification with cssnano
    (await import('cssnano')).default({ preset: 'default' }),"#
    } else {
        ""
    };

    let autoprefixer_plugin = if options.autoprefixer {
        r#"
    (await import('autoprefixer')).default,"#
    } else {
        ""
    };

    format!(
        r#"
import fs from 'fs';
import path from 'path';
import postcss from 'postcss';
import tailwindcss from '@tailwindcss/postcss';

async function build() {{
  const inputPath = '{input_path}';
  const outputPath = '{output_path}';

  // Ensure output directory exists
  const outputDir = path.dirname(outputPath);
  if (!fs.existsSync(outputDir)) {{
    fs.mkdirSync(outputDir, {{ recursive: true }});
  }}

  // Read input CSS
  const css = fs.readFileSync(inputPath, 'utf8');

  // Configure PostCSS plugins
  const plugins = [
    tailwindcss,{autoprefixer_plugin}{minify_plugin}
  ];

  try {{
    // Process with PostCSS
    const result = await postcss(plugins).process(css, {{
      from: inputPath,
      to: outputPath,
    }});

    // Write output
    fs.writeFileSync(outputPath, result.css);

    // Write source map if generated
    if (result.map) {{
      fs.writeFileSync(outputPath + '.map', result.map.toString());
    }}

    console.log(JSON.stringify({{
      success: true,
      input: inputPath,
      output: outputPath,
      size: result.css.length
    }}));
  }} catch (error) {{
    console.error(JSON.stringify({{
      success: false,
      error: error.message
    }}));
    process.exit(1);
  }}
}}

build();
"#,
        input_path = input_path,
        output_path = output_path,
        autoprefixer_plugin = autoprefixer_plugin,
        minify_plugin = minify_plugin,
    )
}

/// Generate a watch script for PostCSS + Tailwind
fn generate_postcss_watch_script(options: &TailwindOptions) -> String {
    let input_path = options.input.to_string_lossy().replace('\\', "/");
    let output_path = options.output.to_string_lossy().replace('\\', "/");
    let content_globs = options
        .content
        .iter()
        .map(|c| format!("'{}'", c.replace('\\', "/")))
        .collect::<Vec<_>>()
        .join(", ");

    let minify_plugin = if options.minify {
        r#"
    (await import('cssnano')).default({ preset: 'default' }),"#
    } else {
        ""
    };

    let autoprefixer_plugin = if options.autoprefixer {
        r#"
    (await import('autoprefixer')).default,"#
    } else {
        ""
    };

    format!(
        r#"
import fs from 'fs';
import path from 'path';
import postcss from 'postcss';
import tailwindcss from '@tailwindcss/postcss';
import chokidar from 'chokidar';

const inputPath = '{input_path}';
const outputPath = '{output_path}';
const contentGlobs = [{content_globs}];

// Ensure output directory exists
const outputDir = path.dirname(outputPath);
if (!fs.existsSync(outputDir)) {{
  fs.mkdirSync(outputDir, {{ recursive: true }});
}}

async function build() {{
  const css = fs.readFileSync(inputPath, 'utf8');

  const plugins = [
    tailwindcss,{autoprefixer_plugin}{minify_plugin}
  ];

  try {{
    const result = await postcss(plugins).process(css, {{
      from: inputPath,
      to: outputPath,
    }});

    fs.writeFileSync(outputPath, result.css);
    console.log(JSON.stringify({{
      type: 'build',
      success: true,
      size: result.css.length,
      timestamp: new Date().toISOString()
    }}));
  }} catch (error) {{
    console.error(JSON.stringify({{
      type: 'error',
      message: error.message
    }}));
  }}
}}

// Initial build
await build();

// Watch for changes
const watcher = chokidar.watch([inputPath, ...contentGlobs], {{
  ignoreInitial: true,
  ignored: [outputPath, '**/node_modules/**', '**/.git/**']
}});

watcher.on('all', async (event, filePath) => {{
  console.log(JSON.stringify({{
    type: 'change',
    event,
    file: filePath
  }}));
  await build();
}});

console.log(JSON.stringify({{ type: 'watching', paths: [inputPath, ...contentGlobs] }}));
"#,
        input_path = input_path,
        output_path = output_path,
        content_globs = content_globs,
        autoprefixer_plugin = autoprefixer_plugin,
        minify_plugin = minify_plugin,
    )
}

/// Compile TailwindCSS using PostCSS
pub fn compile_tailwind(options: &TailwindOptions) -> Result<()> {
    // Ensure output directory exists
    if let Some(parent) = options.output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let script = generate_postcss_script(options);

    // Write script to temp file
    let script_path = options.cwd.join(".ferric-postcss-build.mjs");
    std::fs::write(&script_path, &script)?;

    // Run the script with Node.js
    let output = Command::new("node")
        .current_dir(&options.cwd)
        .arg(&script_path)
        .output()
        .context("Failed to run Node.js. Is it installed?")?;

    // Clean up temp script
    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try to parse error from JSON output
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                anyhow::bail!("PostCSS/Tailwind error: {}", error);
            }
        }

        anyhow::bail!(
            "TailwindCSS compilation failed:\n{}\n{}",
            stderr,
            stdout
        );
    }

    Ok(())
}

/// Compile TailwindCSS with detailed output
pub fn compile_tailwind_verbose(options: &TailwindOptions) -> Result<CompileResult> {
    let start = std::time::Instant::now();

    // Ensure output directory exists
    if let Some(parent) = options.output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let script = generate_postcss_script(options);

    // Write script to temp file
    let script_path = options.cwd.join(".ferric-postcss-build.mjs");
    std::fs::write(&script_path, &script)?;

    // Run the script with Node.js
    let output = Command::new("node")
        .current_dir(&options.cwd)
        .arg(&script_path)
        .output()
        .context("Failed to run Node.js. Is it installed?")?;

    // Clean up temp script
    let _ = std::fs::remove_file(&script_path);

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Try to parse error from JSON output
        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                anyhow::bail!("PostCSS/Tailwind error: {}", error);
            }
        }

        anyhow::bail!(
            "TailwindCSS compilation failed:\n{}\n{}",
            stderr,
            stdout
        );
    }

    // Parse the JSON result
    let size = if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
        result.get("size").and_then(|s| s.as_u64()).unwrap_or(0)
    } else {
        std::fs::metadata(&options.output)
            .map(|m| m.len())
            .unwrap_or(0)
    };

    Ok(CompileResult {
        input: options.input.clone(),
        output: options.output.clone(),
        size,
        duration: start.elapsed(),
    })
}

/// Result of a TailwindCSS compilation
#[derive(Debug)]
pub struct CompileResult {
    pub input: PathBuf,
    pub output: PathBuf,
    pub size: u64,
    pub duration: std::time::Duration,
}

/// Start TailwindCSS watch mode using PostCSS
pub fn start_tailwind_watch(options: &TailwindOptions) -> Result<Child> {
    let script = generate_postcss_watch_script(options);

    // Write script to a permanent location for watch mode
    let script_path = options.cwd.join(".ferric-postcss-watch.mjs");
    std::fs::write(&script_path, &script)?;

    let child = Command::new("node")
        .current_dir(&options.cwd)
        .arg(&script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start TailwindCSS watch process")?;

    Ok(child)
}

/// Watch for CSS changes and recompile TailwindCSS (Rust-based watcher)
pub async fn watch_and_compile(options: TailwindOptions) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let running = Arc::new(AtomicBool::new(true));

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    // Watch input file directory
    if let Some(parent) = options.input.parent() {
        watcher.watch(parent, RecursiveMode::NonRecursive)?;
    }

    // Watch src directory for content changes
    let src_dir = options.cwd.join("src");
    if src_dir.exists() {
        watcher.watch(&src_dir, RecursiveMode::Recursive)?;
    }

    println!(
        "{} Watching for TailwindCSS changes...",
        style("Watch:").cyan().bold()
    );

    // Initial compilation
    match compile_tailwind_verbose(&options) {
        Ok(result) => {
            println!(
                "  {} {} → {} ({}) in {:?}",
                style("✓").green(),
                result.input.display(),
                result.output.display(),
                format_size(result.size),
                result.duration
            );
        }
        Err(e) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
        }
    }

    // Debounce timer
    let mut last_compile = std::time::Instant::now();
    let debounce_duration = std::time::Duration::from_millis(100);

    // Watch for changes
    while running.load(Ordering::Relaxed) {
        match rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(event) => {
                let should_compile = event.paths.iter().any(|path| {
                    let ext = path.extension().and_then(|e| e.to_str());
                    matches!(ext, Some("css") | Some("html") | Some("rs"))
                });

                if should_compile && last_compile.elapsed() > debounce_duration {
                    last_compile = std::time::Instant::now();

                    println!(
                        "  {} Changes detected, recompiling...",
                        style("↻").yellow()
                    );

                    match compile_tailwind_verbose(&options) {
                        Ok(result) => {
                            println!(
                                "  {} Compiled in {:?} ({})",
                                style("✓").green(),
                                result.duration,
                                format_size(result.size)
                            );
                        }
                        Err(e) => {
                            eprintln!("  {} {}", style("✗").red(), e);
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}

/// Build TailwindCSS styles for a project
pub async fn build_project_styles(config: &FerricConfig, release: bool) -> Result<()> {
    if !is_tailwind_available() {
        println!(
            "  {} TailwindCSS not detected, skipping",
            style("!").yellow()
        );
        return Ok(());
    }

    let mut options = TailwindOptions::from_config(config);
    options.minify = release || config.styles.output_style == "compressed";

    match compile_tailwind_verbose(&options) {
        Ok(result) => {
            let cwd = std::env::current_dir().unwrap_or_default();
            println!(
                "  {} {} → {} ({}) in {:?}",
                style("✓").green(),
                result
                    .input
                    .strip_prefix(&cwd)
                    .unwrap_or(&result.input)
                    .display(),
                result
                    .output
                    .strip_prefix(&cwd)
                    .unwrap_or(&result.output)
                    .display(),
                format_size(result.size),
                result.duration
            );
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

/// Install TailwindCSS dependencies in a project
pub async fn install_tailwind(cwd: &Path, include_extras: bool) -> Result<()> {
    let pm = detect_package_manager();

    println!(
        "  {} Installing TailwindCSS using {}...",
        style("→").cyan(),
        pm
    );

    let mut packages = vec!["tailwindcss", "@tailwindcss/postcss", "postcss"];

    if include_extras {
        packages.extend(["autoprefixer", "cssnano"]);
    }

    let install_args: Vec<&str> = match pm {
        "pnpm" => {
            let mut args = vec!["add", "-D"];
            args.extend(packages.iter());
            args
        }
        "yarn" => {
            let mut args = vec!["add", "-D"];
            args.extend(packages.iter());
            args
        }
        "bun" => {
            let mut args = vec!["add", "-D"];
            args.extend(packages.iter());
            args
        }
        _ => {
            let mut args = vec!["install", "-D"];
            args.extend(packages.iter());
            args
        }
    };

    let status = Command::new(pm)
        .current_dir(cwd)
        .args(&install_args)
        .status()
        .context(format!("Failed to run {}", pm))?;

    if !status.success() {
        anyhow::bail!("Failed to install TailwindCSS dependencies");
    }

    println!(
        "  {} TailwindCSS installed successfully",
        style("✓").green()
    );
    Ok(())
}

/// Generate a basic TailwindCSS input CSS file
pub fn generate_tailwind_css() -> &'static str {
    r#"@import "tailwindcss";
"#
}

/// Generate a TailwindCSS 4 CSS file with theme customization
pub fn generate_tailwind_css_with_theme(project_name: &str) -> String {
    format!(
        r#"@import "tailwindcss";

/* Custom theme for {project_name} */
@theme {{
  /* Font families */
  --font-sans: "Inter var", "Inter", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", "Fira Code", ui-monospace, monospace;

  /* Brand colors */
  --color-ferric-50: #fff7ed;
  --color-ferric-100: #ffedd5;
  --color-ferric-200: #fed7aa;
  --color-ferric-300: #fdba74;
  --color-ferric-400: #fb923c;
  --color-ferric-500: #f97316;
  --color-ferric-600: #ea580c;
  --color-ferric-700: #c2410c;
  --color-ferric-800: #9a3412;
  --color-ferric-900: #7c2d12;
  --color-ferric-950: #431407;
}}

/* Base layer customizations */
@layer base {{
  html {{
    scroll-behavior: smooth;
  }}

  body {{
    font-family: var(--font-sans);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }}

  /* Custom scrollbar */
  ::-webkit-scrollbar {{
    width: 8px;
    height: 8px;
  }}

  ::-webkit-scrollbar-track {{
    background: transparent;
  }}

  ::-webkit-scrollbar-thumb {{
    background-color: var(--color-slate-300);
    border-radius: 0.25rem;
  }}

  @media (prefers-color-scheme: dark) {{
    ::-webkit-scrollbar-thumb {{
      background-color: var(--color-slate-600);
    }}
  }}

  ::-webkit-scrollbar-thumb:hover {{
    background-color: var(--color-slate-400);
  }}

  @media (prefers-color-scheme: dark) {{
    ::-webkit-scrollbar-thumb:hover {{
      background-color: var(--color-slate-500);
    }}
  }}

  /* Selection color */
  ::selection {{
    background-color: color-mix(in oklab, var(--color-ferric-500) 30%, transparent);
  }}
}}

/* Component layer for reusable styles */
@layer components {{
  /* Primary button */
  .btn {{
    @apply inline-flex items-center justify-center gap-2 rounded-xl px-6 py-3 text-sm font-semibold transition-all duration-200;
  }}

  .btn:focus {{
    @apply outline-none ring-2 ring-offset-2;
  }}

  .btn-primary {{
    @apply bg-gradient-to-r from-ferric-500 to-red-600 text-white shadow-lg shadow-ferric-500/25;
  }}

  .btn-primary:hover {{
    @apply -translate-y-0.5 shadow-xl shadow-ferric-500/40;
  }}

  .btn-secondary {{
    @apply bg-slate-900 text-white dark:bg-white dark:text-slate-900;
  }}

  .btn-secondary:hover {{
    @apply -translate-y-0.5;
  }}

  .btn-ghost {{
    @apply text-slate-600 dark:text-slate-400;
  }}

  .btn-ghost:hover {{
    @apply bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-white;
  }}

  /* Code block */
  .code-block {{
    @apply overflow-hidden rounded-xl bg-slate-900 shadow-xl dark:bg-slate-950;
  }}

  .code-block-header {{
    @apply flex items-center gap-2 border-b border-slate-700 bg-slate-800 px-4 py-3 dark:bg-black/30;
  }}

  .code-block-content {{
    @apply overflow-x-auto p-5 text-sm leading-relaxed;
  }}

  .code-block-content code {{
    @apply font-mono text-slate-300;
  }}

  /* Inline code */
  .inline-code {{
    @apply rounded bg-slate-200 px-1.5 py-0.5 font-mono text-sm text-slate-800 dark:bg-slate-700 dark:text-slate-200;
  }}

  /* Feature card */
  .feature-card {{
    @apply rounded-2xl border border-slate-200 bg-white p-6 transition-all duration-200 dark:border-slate-700 dark:bg-slate-800;
  }}

  .feature-card:hover {{
    @apply -translate-y-1 border-ferric-500 shadow-lg shadow-ferric-500/10;
  }}
}}

/* Utility layer for custom utilities */
@layer utilities {{
  /* Text gradient */
  .text-gradient {{
    @apply bg-gradient-to-r from-ferric-500 to-red-600 bg-clip-text text-transparent;
  }}

  /* Background grid pattern */
  .bg-grid {{
    background-image: linear-gradient(90deg, rgba(148, 163, 184, 0.1) 1px, transparent 1px),
      linear-gradient(rgba(148, 163, 184, 0.1) 1px, transparent 1px);
    background-size: 24px 24px;
  }}
}}
"#,
        project_name = project_name
    )
}

/// Generate package.json for a TailwindCSS project (PostCSS-based)
pub fn generate_package_json(project_name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "A Ferric web application",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev:css": "ferric tailwind --watch",
    "build:css": "ferric tailwind --minify"
  }},
  "devDependencies": {{
    "@tailwindcss/postcss": "^4.1.8",
    "autoprefixer": "^10.4.21",
    "cssnano": "^7.0.6",
    "postcss": "^8.5.4",
    "tailwindcss": "^4.1.8"
  }}
}}
"#,
        name = project_name
    )
}

/// Generate postcss.config.js for the project
pub fn generate_postcss_config() -> &'static str {
    r#"export default {
  plugins: {
    "@tailwindcss/postcss": {},
    autoprefixer: {},
  },
};
"#
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

/// CLI command handler for TailwindCSS operations
pub async fn compile_command(
    input: Option<&str>,
    output: Option<&str>,
    watch: bool,
    minify: bool,
) -> Result<()> {
    let config = FerricConfig::load().unwrap_or_default();
    let cwd = std::env::current_dir()?;

    let input_path = input
        .map(|i| cwd.join(i))
        .unwrap_or_else(|| cwd.join(&config.styles.root));

    let output_path = output
        .map(|o| cwd.join(o))
        .unwrap_or_else(|| cwd.join(&config.build.out_dir).join(&config.styles.output));

    let options = TailwindOptions {
        input: input_path,
        output: output_path,
        minify,
        watch,
        content: vec![
            format!("./{src}/**/*.rs", src = config.build.src_dir),
            format!("./{src}/**/*.html", src = config.build.src_dir),
            "./**/*.html".to_string(),
        ],
        cwd,
        autoprefixer: true,
        postcss_plugins: Vec::new(),
    };

    if watch {
        watch_and_compile(options).await
    } else {
        let result = compile_tailwind_verbose(&options)?;
        println!(
            "{} {} → {} ({}) in {:?}",
            style("Compiled:").green().bold(),
            result.input.display(),
            result.output.display(),
            format_size(result.size),
            result.duration
        );
        Ok(())
    }
}

/// Initialize TailwindCSS in a project
pub async fn init_command(cwd: &Path) -> Result<()> {
    println!(
        "{} Initializing TailwindCSS...",
        style("→").cyan().bold()
    );

    // Check if package.json exists
    let package_json_path = cwd.join("package.json");
    if !package_json_path.exists() {
        // Create package.json
        let project_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ferric-app");
        std::fs::write(&package_json_path, generate_package_json(project_name))?;
        println!("  {} Created package.json", style("✓").green());
    }

    // Install dependencies
    install_tailwind(cwd, true).await?;

    // Create postcss.config.js if it doesn't exist
    let postcss_config_path = cwd.join("postcss.config.mjs");
    if !postcss_config_path.exists() {
        std::fs::write(&postcss_config_path, generate_postcss_config())?;
        println!("  {} Created postcss.config.mjs", style("✓").green());
    }

    // Create src/styles.css if it doesn't exist
    let styles_css_path = cwd.join("src/styles.css");
    if !styles_css_path.exists() {
        std::fs::create_dir_all(styles_css_path.parent().unwrap())?;
        let project_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ferric-app");
        std::fs::write(&styles_css_path, generate_tailwind_css_with_theme(project_name))?;
        println!("  {} Created src/styles.css", style("✓").green());
    }

    println!(
        "\n{} TailwindCSS initialized successfully!",
        style("✓").green().bold()
    );
    println!("\nRun {} to start development", style("ferric serve").cyan());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_package_manager() {
        let pm = detect_package_manager();
        assert!(["pnpm", "npm", "yarn", "bun"].contains(&pm));
    }

    #[test]
    fn test_tailwind_options_default() {
        let options = TailwindOptions::default();
        assert!(!options.minify);
        assert!(!options.watch);
        assert!(options.autoprefixer);
    }

    #[test]
    fn test_generate_package_json() {
        let json = generate_package_json("test-app");
        assert!(json.contains("test-app"));
        assert!(json.contains("tailwindcss"));
        assert!(json.contains("postcss"));
        assert!(json.contains("autoprefixer"));
        assert!(json.contains("cssnano"));
    }

    #[test]
    fn test_generate_postcss_script() {
        let options = TailwindOptions::default();
        let script = generate_postcss_script(&options);
        assert!(script.contains("postcss"));
        assert!(script.contains("tailwindcss"));
    }
}
