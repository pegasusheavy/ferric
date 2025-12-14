//! Dependency Injection integration for markdown.

use crate::{MarkdownService, MarkdownConfig, MarkdownPipe};
use ferric_core::di::{InjectionToken, Injector, Injectable};

/// Injection tokens for markdown configuration.
pub mod tokens {
    use super::*;

    /// Enable HTML sanitization.
    pub const MARKDOWN_SANITIZE_HTML: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_SANITIZE_HTML", 6001);

    /// Allow dangerous HTML (disables sanitization).
    pub const MARKDOWN_ALLOW_DANGEROUS_HTML: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ALLOW_DANGEROUS_HTML", 6002);

    /// Enable GitHub Flavored Markdown.
    pub const MARKDOWN_ENABLE_GFM: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ENABLE_GFM", 6003);

    /// Enable tables.
    pub const MARKDOWN_ENABLE_TABLES: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ENABLE_TABLES", 6004);

    /// Enable strikethrough.
    pub const MARKDOWN_ENABLE_STRIKETHROUGH: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ENABLE_STRIKETHROUGH", 6005);

    /// Enable task lists.
    pub const MARKDOWN_ENABLE_TASKLISTS: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ENABLE_TASKLISTS", 6006);

    /// Enable footnotes.
    pub const MARKDOWN_ENABLE_FOOTNOTES: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ENABLE_FOOTNOTES", 6007);

    /// Enable syntax highlighting.
    pub const MARKDOWN_ENABLE_SYNTAX_HIGHLIGHTING: InjectionToken<bool> =
        InjectionToken::with_id("MARKDOWN_ENABLE_SYNTAX_HIGHLIGHTING", 6008);

    /// Syntax highlighting theme.
    pub const MARKDOWN_SYNTAX_THEME: InjectionToken<String> =
        InjectionToken::with_id("MARKDOWN_SYNTAX_THEME", 6009);

    /// CSS class prefix.
    pub const MARKDOWN_CSS_CLASS_PREFIX: InjectionToken<String> =
        InjectionToken::with_id("MARKDOWN_CSS_CLASS_PREFIX", 6010);

    /// Complete configuration object.
    pub const MARKDOWN_CONFIG: InjectionToken<MarkdownConfig> =
        InjectionToken::with_id("MARKDOWN_CONFIG", 6011);
}

impl Injectable for MarkdownConfig {
    fn create(injector: &Injector) -> Self {
        // Check for complete config first
        if let Some(config) = injector.resolve_token(&tokens::MARKDOWN_CONFIG) {
            return (*config).clone();
        }

        // Build config from individual tokens
        let mut config = Self::default();

        if let Some(sanitize) = injector.resolve_token(&tokens::MARKDOWN_SANITIZE_HTML) {
            config.sanitize_html = *sanitize;
        }

        if let Some(dangerous) = injector.resolve_token(&tokens::MARKDOWN_ALLOW_DANGEROUS_HTML) {
            config.allow_dangerous_html = *dangerous;
        }

        if let Some(gfm) = injector.resolve_token(&tokens::MARKDOWN_ENABLE_GFM) {
            config.enable_gfm = *gfm;
        }

        if let Some(tables) = injector.resolve_token(&tokens::MARKDOWN_ENABLE_TABLES) {
            config.enable_tables = *tables;
        }

        if let Some(strike) = injector.resolve_token(&tokens::MARKDOWN_ENABLE_STRIKETHROUGH) {
            config.enable_strikethrough = *strike;
        }

        if let Some(tasks) = injector.resolve_token(&tokens::MARKDOWN_ENABLE_TASKLISTS) {
            config.enable_tasklists = *tasks;
        }

        if let Some(footnotes) = injector.resolve_token(&tokens::MARKDOWN_ENABLE_FOOTNOTES) {
            config.enable_footnotes = *footnotes;
        }

        if let Some(syntax) = injector.resolve_token(&tokens::MARKDOWN_ENABLE_SYNTAX_HIGHLIGHTING) {
            config.enable_syntax_highlighting = *syntax;
        }

        if let Some(prefix) = injector.resolve_token(&tokens::MARKDOWN_CSS_CLASS_PREFIX) {
            config.css_class_prefix = (*prefix).clone();
        }

        config
    }
}

/// Markdown module for DI setup.
pub struct MarkdownModule;

impl MarkdownModule {
    /// Provide markdown service with default configuration.
    pub fn provide(injector: &Injector) {
        injector.register_singleton::<MarkdownConfig>();
        injector.register_singleton::<MarkdownService>();
        injector.register_singleton::<MarkdownPipe>();
    }

    /// Provide markdown with GitHub Flavored Markdown configuration.
    pub fn provide_github(injector: &Injector) {
        injector.register_token(&tokens::MARKDOWN_CONFIG, MarkdownConfig::github());
        injector.register_singleton::<MarkdownService>();
        injector.register_singleton::<MarkdownPipe>();
    }

    /// Provide markdown with minimal configuration.
    pub fn provide_minimal(injector: &Injector) {
        injector.register_token(&tokens::MARKDOWN_CONFIG, MarkdownConfig::minimal());
        injector.register_singleton::<MarkdownService>();
        injector.register_singleton::<MarkdownPipe>();
    }

    /// Provide markdown with custom configuration.
    pub fn provide_with_config<F>(injector: &Injector, configure: F)
    where
        F: Fn(&Injector),
    {
        configure(injector);
        injector.register_singleton::<MarkdownConfig>();
        injector.register_singleton::<MarkdownService>();
        injector.register_singleton::<MarkdownPipe>();
    }

    /// Provide markdown with unsafe mode (no sanitization).
    ///
    /// ⚠️ **Warning**: Only use with trusted content!
    pub fn provide_unsafe(injector: &Injector) {
        injector.register_token(&tokens::MARKDOWN_CONFIG, MarkdownConfig::unsafe_mode());
        injector.register_singleton::<MarkdownService>();
        injector.register_singleton::<MarkdownPipe>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_provide() {
        let injector = Injector::root();
        MarkdownModule::provide(&injector);

        assert!(injector.has::<MarkdownService>());
        assert!(injector.has::<MarkdownPipe>());
    }

    #[test]
    fn test_module_github() {
        let injector = Injector::root();
        MarkdownModule::provide_github(&injector);

        let service = injector.resolve::<MarkdownService>().unwrap();
        assert!(service.config().enable_gfm);
    }

    #[test]
    fn test_config_from_tokens() {
        let injector = Injector::root();

        // Set individual tokens
        injector.register_token(&tokens::MARKDOWN_ENABLE_GFM, true);
        injector.register_token(&tokens::MARKDOWN_CSS_CLASS_PREFIX, "custom-".to_string());

        // Create config
        injector.register_singleton::<MarkdownConfig>();
        let config = injector.resolve::<MarkdownConfig>().unwrap();

        assert!(config.enable_gfm);
        assert_eq!(config.css_class_prefix, "custom-");
    }
}

