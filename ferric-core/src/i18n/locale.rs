//! Locale parsing and management.

use std::fmt;

/// Represents a parsed locale identifier.
///
/// A locale consists of a language code, optional script, region, and variants.
/// Examples: "en", "en-US", "zh-Hans-CN", "sr-Latn-RS"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    /// ISO 639-1 or 639-2 language code (e.g., "en", "zh")
    pub language: String,
    /// ISO 15924 script code (e.g., "Latn", "Hans")
    pub script: Option<String>,
    /// ISO 3166-1 alpha-2 region code (e.g., "US", "CN")
    pub region: Option<String>,
    /// Additional variant subtags
    pub variants: Vec<String>,
}

impl Locale {
    /// Create a new locale with just a language code.
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            script: None,
            region: None,
            variants: Vec::new(),
        }
    }

    /// Create a locale with language and region.
    pub fn with_region(language: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            script: None,
            region: Some(region.into().to_uppercase()),
            variants: Vec::new(),
        }
    }

    /// Set the script.
    pub fn script(mut self, script: impl Into<String>) -> Self {
        let s = script.into();
        // Title case for scripts
        self.script = Some(format!(
            "{}{}",
            s.chars().next().unwrap_or_default().to_uppercase(),
            s.chars().skip(1).collect::<String>().to_lowercase()
        ));
        self
    }

    /// Add a variant.
    pub fn variant(mut self, variant: impl Into<String>) -> Self {
        self.variants.push(variant.into().to_lowercase());
        self
    }

    /// Get the full locale tag (e.g., "en-US", "zh-Hans-CN").
    pub fn tag(&self) -> String {
        let mut parts = vec![self.language.clone()];

        if let Some(ref script) = self.script {
            parts.push(script.clone());
        }

        if let Some(ref region) = self.region {
            parts.push(region.clone());
        }

        for variant in &self.variants {
            parts.push(variant.clone());
        }

        parts.join("-")
    }

    /// Get a list of fallback locales for translation lookups.
    ///
    /// For "zh-Hans-CN", returns: ["zh-Hans-CN", "zh-Hans", "zh", "en"]
    pub fn fallback_chain(&self) -> Vec<String> {
        let mut chain = Vec::new();

        // Full tag
        chain.push(self.tag());

        // Without variants
        if !self.variants.is_empty() {
            let mut locale = self.clone();
            locale.variants.clear();
            chain.push(locale.tag());
        }

        // Without region
        if self.region.is_some() {
            let mut locale = self.clone();
            locale.region = None;
            locale.variants.clear();
            chain.push(locale.tag());
        }

        // Without script
        if self.script.is_some() {
            chain.push(self.language.clone());
        }

        // Default fallback to English
        if self.language != "en" {
            chain.push("en".to_string());
        }

        chain
    }

    /// Check if this locale matches another (considering fallbacks).
    pub fn matches(&self, other: &Locale) -> bool {
        if self.language != other.language {
            return false;
        }

        // If other specifies a script, it must match
        if let Some(ref other_script) = other.script {
            if self.script.as_ref() != Some(other_script) {
                return false;
            }
        }

        // If other specifies a region, it must match
        if let Some(ref other_region) = other.region {
            if self.region.as_ref() != Some(other_region) {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag())
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}

/// Additional information about a locale.
#[derive(Debug, Clone)]
pub struct LocaleInfo {
    /// The locale.
    pub locale: Locale,
    /// Native name (e.g., "Deutsch" for German).
    pub native_name: String,
    /// English name (e.g., "German").
    pub english_name: String,
    /// Text direction.
    pub direction: TextDirection,
}

/// Text direction for a locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    /// Left-to-right (most languages).
    Ltr,
    /// Right-to-left (Arabic, Hebrew, etc.).
    Rtl,
}

impl Default for TextDirection {
    fn default() -> Self {
        TextDirection::Ltr
    }
}

/// Parse a locale string into a `Locale` struct.
///
/// Supports various formats:
/// - "en"
/// - "en-US"
/// - "en_US" (underscore variant)
/// - "zh-Hans-CN"
pub fn parse_locale(locale_str: &str) -> Locale {
    // Normalize: replace underscores with hyphens
    let normalized = locale_str.replace('_', "-");
    let parts: Vec<&str> = normalized.split('-').collect();

    let mut locale = Locale::new(parts.first().copied().unwrap_or("en"));

    for (i, part) in parts.iter().enumerate().skip(1) {
        let len = part.len();

        if len == 4 && i == 1 {
            // Script (4 letters, e.g., "Hans", "Latn")
            locale.script = Some(format!(
                "{}{}",
                part.chars().next().unwrap_or_default().to_uppercase(),
                part.chars().skip(1).collect::<String>().to_lowercase()
            ));
        } else if len == 2 && part.chars().all(|c| c.is_ascii_alphabetic()) {
            // Region (2 letters, e.g., "US", "CN")
            locale.region = Some(part.to_uppercase());
        } else if len >= 5 || (len >= 4 && part.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)) {
            // Variant (5+ chars, or 4+ starting with digit)
            locale.variants.push(part.to_lowercase());
        }
    }

    locale
}

/// Get the browser's preferred locale.
#[cfg(target_arch = "wasm32")]
pub fn get_browser_locale() -> Locale {
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return Locale::default(),
    };

    let navigator = window.navigator();

    // Try to get language from navigator
    if let Some(lang) = navigator.language() {
        return parse_locale(&lang);
    }

    Locale::default()
}

/// Get the browser's preferred locale (non-WASM fallback).
#[cfg(not(target_arch = "wasm32"))]
pub fn get_browser_locale() -> Locale {
    // On non-WASM platforms, try to get from environment
    if let Ok(lang) = std::env::var("LANG") {
        // LANG format is usually like "en_US.UTF-8"
        let locale_part = lang.split('.').next().unwrap_or("en");
        return parse_locale(locale_part);
    }

    Locale::default()
}

/// Check if a language uses RTL text direction.
pub fn is_rtl_language(language: &str) -> bool {
    matches!(
        language.to_lowercase().as_str(),
        "ar" | "arc" | "arz" | "az" | "dv" | "fa" | "he" | "khw" | "ks" |
        "ku" | "mzn" | "nqo" | "pnb" | "ps" | "sd" | "ug" | "ur" | "yi"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_locale() {
        let locale = parse_locale("en");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_parse_locale_with_region() {
        let locale = parse_locale("en-US");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_parse_locale_with_underscore() {
        let locale = parse_locale("en_US");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_parse_locale_with_script() {
        let locale = parse_locale("zh-Hans-CN");
        assert_eq!(locale.language, "zh");
        assert_eq!(locale.script, Some("Hans".to_string()));
        assert_eq!(locale.region, Some("CN".to_string()));
    }

    #[test]
    fn test_fallback_chain() {
        let locale = parse_locale("zh-Hans-CN");
        let chain = locale.fallback_chain();
        assert_eq!(chain, vec!["zh-Hans-CN", "zh-Hans", "zh", "en"]);
    }

    #[test]
    fn test_rtl_detection() {
        assert!(is_rtl_language("ar"));
        assert!(is_rtl_language("he"));
        assert!(!is_rtl_language("en"));
        assert!(!is_rtl_language("de"));
    }
}

