//! Configuration for markdown rendering.

use serde::{Deserialize, Serialize};

/// Configuration for markdown rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownConfig {
    /// Enable HTML sanitization (recommended).
    pub sanitize_html: bool,

    /// Allow raw HTML in markdown.
    pub allow_dangerous_html: bool,

    /// Enable GitHub Flavored Markdown features.
    pub enable_gfm: bool,

    /// Enable table support.
    pub enable_tables: bool,

    /// Enable strikethrough support.
    pub enable_strikethrough: bool,

    /// Enable task lists.
    pub enable_tasklists: bool,

    /// Enable footnotes.
    pub enable_footnotes: bool,

    /// Enable syntax highlighting for code blocks.
    pub enable_syntax_highlighting: bool,

    /// Syntax highlighting theme.
    pub syntax_theme: SyntaxTheme,

    /// Add target="_blank" to external links.
    pub external_links_new_tab: bool,

    /// Add rel="noopener noreferrer" to external links.
    pub external_links_noopener: bool,

    /// CSS class prefix for generated elements.
    pub css_class_prefix: String,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            sanitize_html: true,
            allow_dangerous_html: false,
            enable_gfm: true,
            enable_tables: true,
            enable_strikethrough: true,
            enable_tasklists: true,
            enable_footnotes: true,
            enable_syntax_highlighting: false,
            syntax_theme: SyntaxTheme::default(),
            external_links_new_tab: true,
            external_links_noopener: true,
            css_class_prefix: "md-".to_string(),
        }
    }
}

impl MarkdownConfig {
    /// Create a configuration for GitHub Flavored Markdown.
    pub fn github() -> Self {
        Self {
            enable_gfm: true,
            enable_tables: true,
            enable_strikethrough: true,
            enable_tasklists: true,
            enable_footnotes: false,
            ..Default::default()
        }
    }

    /// Create a minimal configuration (basic markdown only).
    pub fn minimal() -> Self {
        Self {
            enable_gfm: false,
            enable_tables: false,
            enable_strikethrough: false,
            enable_tasklists: false,
            enable_footnotes: false,
            enable_syntax_highlighting: false,
            ..Default::default()
        }
    }

    /// Create an unsafe configuration (no sanitization).
    ///
    /// ⚠️ **Warning**: This allows arbitrary HTML and JavaScript.
    /// Only use with trusted content!
    pub fn unsafe_mode() -> Self {
        Self {
            sanitize_html: false,
            allow_dangerous_html: true,
            ..Default::default()
        }
    }
}

/// Options for rendering markdown.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub sanitize_html: bool,
    pub allow_dangerous_html: bool,
    pub enable_syntax_highlighting: bool,
    pub syntax_theme: SyntaxTheme,
    pub css_class_prefix: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            sanitize_html: true,
            allow_dangerous_html: false,
            enable_syntax_highlighting: false,
            syntax_theme: SyntaxTheme::default(),
            css_class_prefix: "md-".to_string(),
        }
    }
}

impl From<&MarkdownConfig> for RenderOptions {
    fn from(config: &MarkdownConfig) -> Self {
        Self {
            sanitize_html: config.sanitize_html,
            allow_dangerous_html: config.allow_dangerous_html,
            enable_syntax_highlighting: config.enable_syntax_highlighting,
            syntax_theme: config.syntax_theme.clone(),
            css_class_prefix: config.css_class_prefix.clone(),
        }
    }
}

/// Syntax highlighting theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum SyntaxTheme {
    /// InspiredGitHub theme.
    #[default]
    GitHub,
    /// Solarized (dark).
    SolarizedDark,
    /// Solarized (light).
    SolarizedLight,
    /// Monokai theme.
    Monokai,
    /// Base16 Ocean Dark.
    Base16OceanDark,
    /// Base16 Ocean Light.
    Base16OceanLight,
}


impl SyntaxTheme {
    pub fn as_str(&self) -> &str {
        match self {
            Self::GitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
            Self::Monokai => "Monokai",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
        }
    }
}

