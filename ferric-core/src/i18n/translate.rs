//! Translation storage and retrieval.

use super::locale::Locale;
use super::plural::{get_plural_category, PluralCategory};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A translation key, which can be a simple string or a dot-separated path.
pub type TranslationKey = String;

/// A translation value, which can be a string or nested object.
#[derive(Debug, Clone)]
pub enum TranslationValue {
    /// Simple string value.
    String(String),
    /// Plural forms.
    Plural(HashMap<PluralCategory, String>),
    /// Nested translations.
    Nested(HashMap<String, TranslationValue>),
}

impl TranslationValue {
    /// Get as a string if this is a string value.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            TranslationValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as plural forms if this is a plural value.
    pub fn as_plural(&self) -> Option<&HashMap<PluralCategory, String>> {
        match self {
            TranslationValue::Plural(p) => Some(p),
            _ => None,
        }
    }

    /// Get as nested if this is a nested value.
    pub fn as_nested(&self) -> Option<&HashMap<String, TranslationValue>> {
        match self {
            TranslationValue::Nested(n) => Some(n),
            _ => None,
        }
    }
}

/// Store for translations organized by locale.
#[derive(Debug, Clone, Default)]
pub struct TranslationStore {
    translations: HashMap<String, HashMap<TranslationKey, TranslationValue>>,
}

impl TranslationStore {
    /// Create a new empty translation store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load translations from a JSON value.
    pub fn load_json(&mut self, locale: &str, json: serde_json::Value) {
        let translations = self.translations.entry(locale.to_string()).or_default();
        Self::flatten_json("", &json, translations);
    }

    /// Load translations from a HashMap.
    pub fn load(&mut self, locale: &str, data: HashMap<TranslationKey, TranslationValue>) {
        let translations = self.translations.entry(locale.to_string()).or_default();
        translations.extend(data);
    }

    /// Add a single translation.
    pub fn add(&mut self, locale: &str, key: &str, value: TranslationValue) {
        let translations = self.translations.entry(locale.to_string()).or_default();
        translations.insert(key.to_string(), value);
    }

    /// Add a simple string translation.
    pub fn add_string(&mut self, locale: &str, key: &str, value: impl Into<String>) {
        self.add(locale, key, TranslationValue::String(value.into()));
    }

    /// Add plural translations.
    pub fn add_plural(&mut self, locale: &str, key: &str, forms: HashMap<PluralCategory, String>) {
        self.add(locale, key, TranslationValue::Plural(forms));
    }

    /// Get a translation value for a key in the given locale.
    pub fn get(&self, locale: &str, key: &str) -> Option<&TranslationValue> {
        self.translations.get(locale)?.get(key)
    }

    /// Get a translation with fallback chain.
    pub fn get_with_fallback(&self, locale: &Locale, key: &str) -> Option<&TranslationValue> {
        for fallback in locale.fallback_chain() {
            if let Some(value) = self.get(&fallback, key) {
                return Some(value);
            }
        }
        None
    }

    /// Check if a translation exists.
    pub fn has(&self, locale: &str, key: &str) -> bool {
        self.get(locale, key).is_some()
    }

    /// Get all keys for a locale.
    pub fn keys(&self, locale: &str) -> Vec<&TranslationKey> {
        self.translations
            .get(locale)
            .map(|t| t.keys().collect())
            .unwrap_or_default()
    }

    /// Get all available locales.
    pub fn locales(&self) -> Vec<&str> {
        self.translations.keys().map(|s| s.as_str()).collect()
    }

    /// Flatten JSON into dot-notation keys.
    fn flatten_json(prefix: &str, value: &serde_json::Value, out: &mut HashMap<TranslationKey, TranslationValue>) {
        match value {
            serde_json::Value::String(s) => {
                out.insert(prefix.to_string(), TranslationValue::String(s.clone()));
            }
            serde_json::Value::Object(map) => {
                // Check if this is a plural object (has keys like "one", "other")
                let plural_keys = ["zero", "one", "two", "few", "many", "other"];
                let is_plural = map.keys().all(|k| plural_keys.contains(&k.as_str()));

                if is_plural && !map.is_empty() {
                    let mut forms = HashMap::new();
                    for (k, v) in map {
                        if let serde_json::Value::String(s) = v {
                            let category = match k.as_str() {
                                "zero" => PluralCategory::Zero,
                                "one" => PluralCategory::One,
                                "two" => PluralCategory::Two,
                                "few" => PluralCategory::Few,
                                "many" => PluralCategory::Many,
                                _ => PluralCategory::Other,
                            };
                            forms.insert(category, s.clone());
                        }
                    }
                    out.insert(prefix.to_string(), TranslationValue::Plural(forms));
                } else {
                    // Nested object
                    for (k, v) in map {
                        let new_prefix = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{}.{}", prefix, k)
                        };
                        Self::flatten_json(&new_prefix, v, out);
                    }
                }
            }
            _ => {
                // Convert other types to string
                out.insert(prefix.to_string(), TranslationValue::String(value.to_string()));
            }
        }
    }
}

/// Thread-local translation store for global access.
thread_local! {
    static GLOBAL_STORE: RefCell<TranslationStore> = RefCell::new(TranslationStore::new());
    static CURRENT_LOCALE: RefCell<Locale> = RefCell::new(Locale::default());
}

/// Set the current locale.
pub fn set_locale(locale: Locale) {
    CURRENT_LOCALE.with(|l| *l.borrow_mut() = locale);
}

/// Get the current locale.
pub fn current_locale() -> Locale {
    CURRENT_LOCALE.with(|l| l.borrow().clone())
}

/// Load translations into the global store.
pub fn load_translations(locale: &str, json: serde_json::Value) {
    GLOBAL_STORE.with(|store| store.borrow_mut().load_json(locale, json));
}

/// Translate a key with optional interpolation parameters.
///
/// Parameters are provided as key-value pairs and are substituted
/// for `{key}` placeholders in the translation string.
pub fn translate(key: &str, params: &[(&str, &str)]) -> String {
    t(key, params)
}

/// Short alias for `translate`.
pub fn t(key: &str, params: &[(&str, &str)]) -> String {
    CURRENT_LOCALE.with(|locale| {
        GLOBAL_STORE.with(|store| {
            let store = store.borrow();
            let locale = locale.borrow();

            if let Some(value) = store.get_with_fallback(&locale, key)
                && let Some(s) = value.as_string() {
                    return interpolate(s, params);
                }

            // Return key if not found
            format!("{{{{ {} }}}}", key)
        })
    })
}

/// Translate with pluralization.
///
/// The count determines which plural form to use based on the current locale's rules.
pub fn translate_plural(key: &str, count: f64, params: &[(&str, &str)]) -> String {
    tp(key, count, params)
}

/// Short alias for `translate_plural`.
pub fn tp(key: &str, count: f64, params: &[(&str, &str)]) -> String {
    CURRENT_LOCALE.with(|locale| {
        GLOBAL_STORE.with(|store| {
            let store = store.borrow();
            let locale = locale.borrow();

            if let Some(value) = store.get_with_fallback(&locale, key)
                && let Some(plurals) = value.as_plural() {
                    let category = get_plural_category(&locale.language, count);

                    // Try the specific category, then fall back to "other"
                    let template = plurals
                        .get(&category)
                        .or_else(|| plurals.get(&PluralCategory::Other))
                        .map(|s| s.as_str())
                        .unwrap_or(key);

                    return interpolate(template, params);
                }

            // Return key if not found
            format!("{{{{ {} }}}}", key)
        })
    })
}

/// Interpolate parameters into a template string.
fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut result = template.to_string();

    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}

/// Create a reactive translation signal.
///
/// The signal automatically updates when the locale changes.
pub fn reactive_t(key: &str, params: Vec<(String, String)>) -> Rc<dyn Fn() -> String> {
    let key = key.to_string();
    Rc::new(move || {
        let param_refs: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        t(&key, &param_refs)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_json() {
        let mut store = TranslationStore::new();
        store.load_json(
            "en",
            serde_json::json!({
                "greeting": "Hello, {name}!",
                "common": {
                    "save": "Save",
                    "cancel": "Cancel"
                }
            }),
        );

        assert!(store.has("en", "greeting"));
        assert!(store.has("en", "common.save"));
        assert!(store.has("en", "common.cancel"));
    }

    #[test]
    fn test_interpolation() {
        let result = interpolate("Hello, {name}! You have {count} messages.", &[
            ("name", "World"),
            ("count", "5"),
        ]);
        assert_eq!(result, "Hello, World! You have 5 messages.");
    }

    #[test]
    fn test_plural_loading() {
        let mut store = TranslationStore::new();
        store.load_json(
            "en",
            serde_json::json!({
                "items": {
                    "one": "{count} item",
                    "other": "{count} items"
                }
            }),
        );

        let value = store.get("en", "items").unwrap();
        assert!(value.as_plural().is_some());
    }

    #[test]
    fn test_fallback_chain() {
        let mut store = TranslationStore::new();
        store.add_string("en", "greeting", "Hello");
        store.add_string("en-US", "goodbye", "Goodbye");

        let locale = super::super::locale::parse_locale("en-US");

        // Should find "goodbye" in en-US
        assert!(store.get_with_fallback(&locale, "goodbye").is_some());

        // Should find "greeting" by falling back to "en"
        assert!(store.get_with_fallback(&locale, "greeting").is_some());
    }
}

