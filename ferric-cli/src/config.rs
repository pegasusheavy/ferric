//! Project configuration (ferric.toml)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main project configuration loaded from ferric.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct FerricConfig {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectConfig,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,

    /// SSR configuration
    #[serde(default)]
    pub ssr: SsrConfig,

    /// Style configuration
    #[serde(default)]
    pub styles: StyleConfig,

    /// Development server configuration
    #[serde(default)]
    pub serve: ServeConfig,

    /// Asset configuration
    #[serde(default)]
    pub assets: AssetConfig,
}


impl FerricConfig {
    /// Load configuration from ferric.toml in the current directory
    pub fn load() -> anyhow::Result<Self> {
        Self::load_from(Path::new("ferric.toml"))
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: FerricConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to ferric.toml
    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(Path::new("ferric.toml"))
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check if project uses TailwindCSS
    pub fn uses_tailwind(&self) -> bool {
        self.styles.preprocessor == "tailwind" || self.styles.tailwind.enabled
    }

    /// Generate default ferric.toml content
    pub fn default_toml(name: &str, ssr_enabled: bool) -> String {
        format!(
            r##"# Ferric Project Configuration
# Documentation: https://github.com/user/ferric

[project]
name = "{name}"
version = "0.1.0"
description = "A Ferric web application"

[build]
# Output directory for build artifacts
out_dir = "dist"
# Source directory
src_dir = "src"
# Target optimization level: "debug" or "release"
optimization = "debug"
# Enable source maps
source_maps = true
# Wasm optimization level for release builds (0-4, "s", "z")
wasm_opt = "s"

[build.targets]
# Build targets to generate
browser = true
server = {ssr_enabled}

[ssr]
# Enable server-side rendering
enabled = {ssr_enabled}
# SSR server entry point
entry = "src/server.rs"
# Port for SSR server
port = 3001
# Enable hydration
hydration = true

[styles]
# Style preprocessor: "scss", "sass", "css", "tailwind"
preprocessor = "scss"
# Root stylesheet (main entry point, like Angular's styles.scss)
root = "src/styles.scss"
# Output filename for compiled CSS
output = "styles.css"
# Output style: "expanded", "compressed"
output_style = "expanded"
# Bundle component styles automatically
bundle_components = true
# Autoprefixer targets
autoprefixer = ["last 2 versions", "> 1%"]

[styles.include_paths]
# Additional paths for SCSS @import/@use resolution
paths = ["src", "src/styles"]

[styles.tailwind]
# Enable TailwindCSS (set preprocessor = "tailwind" or enabled = true)
enabled = false
# Content paths to scan for class usage
content = ["./src/**/*.rs", "./src/**/*.html", "./**/*.html"]
# Enable autoprefixer in PostCSS pipeline
autoprefixer = true
# Enable cssnano minification
cssnano = false

[serve]
# Development server port
port = 3000
# Host to bind to
host = "127.0.0.1"
# Enable live reload
live_reload = true
# Open browser on start
open = false
# Proxy API requests
# proxy = {{ "/api" = "http://localhost:8080" }}

[assets]
# Static assets directory
dir = "assets"
# Assets to copy to output
copy = ["favicon.ico", "robots.txt"]
# Enable asset hashing for cache busting
hash = true
# Image optimization
optimize_images = false

[assets.public]
# Public path prefix for assets
prefix = "/"
"##,
            name = name,
            ssr_enabled = ssr_enabled
        )
    }

    /// Generate ferric.toml content for TailwindCSS project
    pub fn tailwind_toml(name: &str) -> String {
        format!(
            r##"# Ferric Project Configuration (TailwindCSS)
# Documentation: https://github.com/user/ferric

[project]
name = "{name}"
version = "0.1.0"
description = "A Ferric web application with TailwindCSS"

[build]
out_dir = "dist"
src_dir = "src"
optimization = "debug"
source_maps = true
wasm_opt = "s"

[build.targets]
browser = true
server = false

[ssr]
enabled = false

[styles]
# Use TailwindCSS via PostCSS
preprocessor = "tailwind"
# Root stylesheet with @import "tailwindcss"
root = "src/styles.css"
# Output filename for compiled CSS
output = "styles.css"
# Output style: "expanded", "compressed"
output_style = "expanded"

[styles.tailwind]
enabled = true
content = ["./src/**/*.rs", "./src/**/*.html", "./**/*.html"]
autoprefixer = true
cssnano = false

[serve]
port = 3000
host = "127.0.0.1"
live_reload = true
open = false

[assets]
dir = "assets"
copy = ["favicon.ico", "robots.txt"]
hash = false
"##,
            name = name
        )
    }
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    #[serde(default = "default_name")]
    pub name: String,

    /// Project version
    #[serde(default = "default_version")]
    pub version: String,

    /// Project description
    #[serde(default)]
    pub description: String,

    /// Project authors
    #[serde(default)]
    pub authors: Vec<String>,
}

fn default_name() -> String {
    "ferric-app".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            version: default_version(),
            description: String::new(),
            authors: Vec::new(),
        }
    }
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Output directory
    #[serde(default = "default_out_dir")]
    pub out_dir: String,

    /// Source directory
    #[serde(default = "default_src_dir")]
    pub src_dir: String,

    /// Optimization level
    #[serde(default = "default_optimization")]
    pub optimization: String,

    /// Enable source maps
    #[serde(default = "default_true")]
    pub source_maps: bool,

    /// Wasm optimization level
    #[serde(default = "default_wasm_opt")]
    pub wasm_opt: String,

    /// Build targets
    #[serde(default)]
    pub targets: BuildTargets,

    /// Cache busting configuration
    #[serde(default)]
    pub cache_busting: CacheBustingConfig,
}

/// Cache busting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBustingConfig {
    /// Enable cache busting
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Hash strategy: "md5", "sha256", or "timestamp"
    #[serde(default = "default_hash_strategy")]
    pub strategy: String,

    /// Hash length for content-based strategies
    #[serde(default = "default_hash_length")]
    pub hash_length: usize,

    /// Apply to CSS files
    #[serde(default = "default_true")]
    pub css: bool,

    /// Apply to JS/Wasm files
    #[serde(default = "default_true")]
    pub js: bool,

    /// Apply to assets
    #[serde(default)]
    pub assets: bool,
}

fn default_hash_strategy() -> String {
    "md5".to_string()
}

fn default_hash_length() -> usize {
    8
}

fn default_out_dir() -> String {
    "dist".to_string()
}

fn default_src_dir() -> String {
    "src".to_string()
}

fn default_optimization() -> String {
    "debug".to_string()
}

fn default_wasm_opt() -> String {
    "s".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            out_dir: default_out_dir(),
            src_dir: default_src_dir(),
            optimization: default_optimization(),
            source_maps: true,
            wasm_opt: default_wasm_opt(),
            targets: BuildTargets::default(),
            cache_busting: CacheBustingConfig::default(),
        }
    }
}

impl Default for CacheBustingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: default_hash_strategy(),
            hash_length: default_hash_length(),
            css: true,
            js: true,
            assets: false,
        }
    }
}

/// Build targets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTargets {
    /// Build for browser
    #[serde(default = "default_true")]
    pub browser: bool,

    /// Build for server (SSR)
    #[serde(default)]
    pub server: bool,
}

impl Default for BuildTargets {
    fn default() -> Self {
        Self {
            browser: true,
            server: false,
        }
    }
}

/// SSR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsrConfig {
    /// Enable SSR
    #[serde(default)]
    pub enabled: bool,

    /// SSR entry point
    #[serde(default = "default_ssr_entry")]
    pub entry: String,

    /// SSR server port
    #[serde(default = "default_ssr_port")]
    pub port: u16,

    /// Enable hydration
    #[serde(default = "default_true")]
    pub hydration: bool,
}

fn default_ssr_entry() -> String {
    "src/server.rs".to_string()
}

fn default_ssr_port() -> u16 {
    3001
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            entry: default_ssr_entry(),
            port: default_ssr_port(),
            hydration: true,
        }
    }
}

/// Style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Style preprocessor: "scss", "sass", "css", "tailwind"
    #[serde(default = "default_preprocessor")]
    pub preprocessor: String,

    /// Root stylesheet (main entry point, like Angular's styles.scss)
    /// This file imports all other styles and is the single output
    #[serde(default = "default_root_stylesheet")]
    pub root: String,

    /// Additional global style files to bundle (legacy, prefer using root)
    #[serde(default)]
    pub global: Vec<String>,

    /// Output filename for compiled CSS
    #[serde(default = "default_css_output")]
    pub output: String,

    /// Output style (expanded/compressed)
    #[serde(default = "default_output_style")]
    pub output_style: String,

    /// Enable CSS modules
    #[serde(default)]
    pub css_modules: bool,

    /// Autoprefixer targets
    #[serde(default)]
    pub autoprefixer: Vec<String>,

    /// Include paths for SCSS @import/@use resolution
    #[serde(default)]
    pub include_paths: IncludePaths,

    /// Inline component styles into the root stylesheet
    /// When true, component .scss files are automatically imported
    #[serde(default = "default_true")]
    pub bundle_components: bool,

    /// TailwindCSS-specific configuration
    #[serde(default)]
    pub tailwind: TailwindConfig,
}

fn default_preprocessor() -> String {
    "scss".to_string()
}

fn default_root_stylesheet() -> String {
    "src/styles.scss".to_string()
}

fn default_css_output() -> String {
    "styles.css".to_string()
}

fn default_output_style() -> String {
    "expanded".to_string()
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            preprocessor: default_preprocessor(),
            root: default_root_stylesheet(),
            global: Vec::new(),
            output: default_css_output(),
            output_style: default_output_style(),
            css_modules: false,
            autoprefixer: vec!["last 2 versions".to_string(), "> 1%".to_string()],
            include_paths: IncludePaths::default(),
            bundle_components: true,
            tailwind: TailwindConfig::default(),
        }
    }
}

/// TailwindCSS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailwindConfig {
    /// Enable TailwindCSS processing via PostCSS
    #[serde(default)]
    pub enabled: bool,

    /// Content paths to scan for class usage
    #[serde(default = "default_tailwind_content")]
    pub content: Vec<String>,

    /// Enable autoprefixer in PostCSS pipeline
    #[serde(default = "default_true")]
    pub autoprefixer: bool,

    /// Enable cssnano minification (in addition to output_style)
    #[serde(default)]
    pub cssnano: bool,

    /// Additional PostCSS plugins to use
    #[serde(default)]
    pub postcss_plugins: Vec<String>,
}

fn default_tailwind_content() -> Vec<String> {
    vec![
        "./src/**/*.rs".to_string(),
        "./src/**/*.html".to_string(),
        "./**/*.html".to_string(),
    ]
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            content: default_tailwind_content(),
            autoprefixer: true,
            cssnano: false,
            postcss_plugins: Vec::new(),
        }
    }
}

/// SCSS include paths
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncludePaths {
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Development server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeConfig {
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Enable live reload
    #[serde(default = "default_true")]
    pub live_reload: bool,

    /// Open browser on start
    #[serde(default)]
    pub open: bool,

    /// Proxy configuration
    #[serde(default)]
    pub proxy: HashMap<String, String>,
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            live_reload: true,
            open: false,
            proxy: HashMap::new(),
        }
    }
}

/// Asset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    /// Assets directory
    #[serde(default = "default_assets_dir")]
    pub dir: String,

    /// Files to copy
    #[serde(default)]
    pub copy: Vec<String>,

    /// Enable asset hashing
    #[serde(default)]
    pub hash: bool,

    /// Optimize images
    #[serde(default)]
    pub optimize_images: bool,

    /// Public path configuration
    #[serde(default)]
    pub public: PublicConfig,
}

fn default_assets_dir() -> String {
    "assets".to_string()
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            dir: default_assets_dir(),
            copy: vec!["favicon.ico".to_string()],
            hash: false,
            optimize_images: false,
            public: PublicConfig::default(),
        }
    }
}

/// Public path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicConfig {
    /// Public path prefix
    #[serde(default = "default_prefix")]
    pub prefix: String,
}

fn default_prefix() -> String {
    "/".to_string()
}

impl Default for PublicConfig {
    fn default() -> Self {
        Self {
            prefix: default_prefix(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FerricConfig::default();
        assert_eq!(config.build.out_dir, "dist");
        assert!(!config.ssr.enabled);
        assert_eq!(config.styles.preprocessor, "scss");
        assert!(!config.uses_tailwind());
    }

    #[test]
    fn test_tailwind_config() {
        let mut config = FerricConfig::default();
        config.styles.preprocessor = "tailwind".to_string();
        assert!(config.uses_tailwind());
    }

    #[test]
    fn test_config_serialization() {
        let config = FerricConfig::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("[project]"));
        assert!(toml.contains("[build]"));
    }
}
