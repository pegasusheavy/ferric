//! I18n service for dependency injection.

use super::format::{format_currency, format_date, format_number, DateFormat, NumberFormat, SimpleDate, format_number_with_options};
use super::icu::{IcuMessage, IcuValue, IcuResult};
use super::locale::{get_browser_locale, is_rtl_language, parse_locale, Locale};
use super::plural::get_plural_category;
use super::translate::{TranslationStore, t, tp, set_locale, load_translations};
use crate::reactive::{Signal, signal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The main i18n service for managing translations and localization.
///
/// This service can be registered with the dependency injection system
/// and provides methods for translation, formatting, and locale management.
///
/// # Example
///
/// ```ignore
/// use ferric_core::i18n::I18nService;
///
/// // Create with default locale
/// let i18n = I18nService::new("en");
///
/// // Load translations
/// i18n.load(serde_json::json!({
///     "greeting": "Hello, {name}!",
///     "items": {
///         "one": "{count} item",
///         "other": "{count} items"
///     }
/// }));
///
/// // Translate
/// let msg = i18n.t("greeting", &[("name", "World")]);
/// ```
#[derive(Clone)]
pub struct I18nService {
    inner: Rc<RefCell<I18nServiceInner>>,
}

struct I18nServiceInner {
    locale: Locale,
    store: TranslationStore,
    locale_signal: Signal<String>,
}

impl I18nService {
    /// Create a new i18n service with the given default locale.
    pub fn new(locale: &str) -> Self {
        let parsed_locale = parse_locale(locale);

        // Update the global locale
        set_locale(parsed_locale.clone());

        Self {
            inner: Rc::new(RefCell::new(I18nServiceInner {
                locale: parsed_locale.clone(),
                store: TranslationStore::new(),
                locale_signal: signal(parsed_locale.tag()),
            })),
        }
    }

    /// Create a new i18n service using the browser's preferred locale.
    pub fn from_browser() -> Self {
        let locale = get_browser_locale();
        Self::new(&locale.tag())
    }

    /// Get the current locale.
    pub fn locale(&self) -> Locale {
        self.inner.borrow().locale.clone()
    }

    /// Get the current locale tag (e.g., "en-US").
    pub fn locale_tag(&self) -> String {
        self.inner.borrow().locale.tag()
    }

    /// Get a reactive signal for the current locale.
    ///
    /// This signal updates whenever `set_locale` is called.
    pub fn locale_signal(&self) -> Signal<String> {
        self.inner.borrow().locale_signal.clone()
    }

    /// Set the current locale.
    pub fn set_locale(&self, locale: &str) {
        let mut inner = self.inner.borrow_mut();
        inner.locale = parse_locale(locale);
        inner.locale_signal.set(inner.locale.tag());

        // Update global locale
        set_locale(inner.locale.clone());
    }

    /// Check if the current locale uses RTL text direction.
    pub fn is_rtl(&self) -> bool {
        let inner = self.inner.borrow();
        is_rtl_language(&inner.locale.language)
    }

    /// Load translations for the current locale.
    pub fn load(&self, translations: serde_json::Value) {
        let locale = self.locale_tag();
        self.load_for_locale(&locale, translations);
    }

    /// Load translations for a specific locale.
    pub fn load_for_locale(&self, locale: &str, translations: serde_json::Value) {
        let mut inner = self.inner.borrow_mut();
        inner.store.load_json(locale, translations.clone());

        // Also update global store
        load_translations(locale, translations);
    }

    /// Translate a key with optional interpolation parameters.
    pub fn t(&self, key: &str, params: &[(&str, &str)]) -> String {
        let inner = self.inner.borrow();
        let locale = &inner.locale;

        if let Some(value) = inner.store.get_with_fallback(locale, key)
            && let Some(s) = value.as_string() {
                return interpolate(s, params);
            }

        // Fallback to global store
        drop(inner);
        t(key, params)
    }

    /// Translate with pluralization.
    pub fn tp(&self, key: &str, count: f64, params: &[(&str, &str)]) -> String {
        let inner = self.inner.borrow();
        let locale = &inner.locale;

        if let Some(value) = inner.store.get_with_fallback(locale, key)
            && let Some(plurals) = value.as_plural() {
                let category = get_plural_category(&locale.language, count);

                if let Some(template) = plurals.get(&category).or_else(|| plurals.get(&super::plural::PluralCategory::Other)) {
                    return interpolate(template, params);
                }
            }

        // Fallback to global store
        drop(inner);
        tp(key, count, params)
    }

    /// Format a number using locale conventions.
    pub fn format_number(&self, value: f64) -> String {
        let inner = self.inner.borrow();
        format_number(&inner.locale.tag(), value)
    }

    /// Format a number with custom options.
    pub fn format_number_with_options(&self, value: f64, options: &NumberFormat) -> String {
        let inner = self.inner.borrow();
        format_number_with_options(&inner.locale.tag(), value, options)
    }

    /// Format a currency value.
    pub fn format_currency(&self, value: f64, currency_code: &str) -> String {
        let inner = self.inner.borrow();
        format_currency(&inner.locale.tag(), value, currency_code)
    }

    /// Format a date.
    pub fn format_date(&self, date: &SimpleDate, format: DateFormat) -> String {
        let inner = self.inner.borrow();
        format_date(&inner.locale.tag(), date, format)
    }

    /// Check if a translation key exists.
    pub fn has(&self, key: &str) -> bool {
        let inner = self.inner.borrow();
        inner.store.get_with_fallback(&inner.locale, key).is_some()
    }

    /// Get all available locales that have translations loaded.
    pub fn available_locales(&self) -> Vec<String> {
        let inner = self.inner.borrow();
        inner.store.locales().iter().map(|s| s.to_string()).collect()
    }

    // ========================================================================
    // ICU Message Format Support
    // ========================================================================

    /// Format an ICU message string with arguments.
    ///
    /// ICU Message Format provides powerful features for internationalization:
    /// - Plural forms: `{count, plural, one{# item} other{# items}}`
    /// - Select: `{gender, select, male{He} female{She} other{They}}`
    /// - Number/date formatting: `{amount, number, currency}`
    /// - Nested messages and escaping
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let i18n = I18nService::new("en");
    ///
    /// // Simple substitution
    /// let msg = i18n.icu("Hello, {name}!", &[("name", IcuValue::from("World"))]);
    /// // => "Hello, World!"
    ///
    /// // Plural
    /// let msg = i18n.icu(
    ///     "{count, plural, one{# message} other{# messages}}",
    ///     &[("count", IcuValue::Number(5.0))]
    /// );
    /// // => "5 messages"
    /// ```
    pub fn icu(&self, message: &str, args: &[(&str, IcuValue)]) -> String {
        let inner = self.inner.borrow();
        match IcuMessage::parse(message) {
            Ok(msg) => msg.format(args, &inner.locale.tag()),
            Err(_) => message.to_string(),
        }
    }

    /// Format an ICU message with a HashMap of arguments.
    pub fn icu_map(&self, message: &str, args: &HashMap<String, IcuValue>) -> String {
        let args_vec: Vec<(&str, IcuValue)> = args
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        self.icu(message, &args_vec)
    }

    /// Translate a key and format it as an ICU message.
    ///
    /// This looks up the translation for the key and then formats it using ICU.
    pub fn t_icu(&self, key: &str, args: &[(&str, IcuValue)]) -> String {
        let inner = self.inner.borrow();
        let locale = &inner.locale;

        // Get the message template
        let template = if let Some(value) = inner.store.get_with_fallback(locale, key) {
            if let Some(s) = value.as_string() {
                s.to_string()
            } else {
                return format!("{{{{ {} }}}}", key);
            }
        } else {
            return format!("{{{{ {} }}}}", key);
        };

        // Format using ICU
        let locale_tag = locale.tag();
        drop(inner);

        match IcuMessage::parse(&template) {
            Ok(msg) => msg.format(args, &locale_tag),
            Err(_) => template,
        }
    }

    /// Compile an ICU message for repeated use.
    ///
    /// Returns a compiled message that can be formatted multiple times
    /// without re-parsing.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let i18n = I18nService::new("en");
    /// let msg = i18n.compile_icu("{count, plural, one{# item} other{# items}}").unwrap();
    ///
    /// // Format multiple times with different values
    /// for count in 1..=5 {
    ///     println!("{}", i18n.format_compiled(&msg, &[("count", IcuValue::Number(count as f64))]));
    /// }
    /// ```
    pub fn compile_icu(&self, message: &str) -> IcuResult<IcuMessage> {
        IcuMessage::parse(message)
    }

    /// Format a pre-compiled ICU message.
    pub fn format_compiled(&self, message: &IcuMessage, args: &[(&str, IcuValue)]) -> String {
        let inner = self.inner.borrow();
        message.format(args, &inner.locale.tag())
    }
}

impl Default for I18nService {
    fn default() -> Self {
        Self::from_browser()
    }
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

// Implement Injectable trait for DI integration
impl crate::di::Injectable for I18nService {
    fn create(_injector: &crate::di::Injector) -> Self {
        Self::from_browser()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_service_basic() {
        let i18n = I18nService::new("en");

        i18n.load(serde_json::json!({
            "greeting": "Hello, {name}!"
        }));

        let result = i18n.t("greeting", &[("name", "World")]);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_i18n_service_plural() {
        let i18n = I18nService::new("en");

        i18n.load(serde_json::json!({
            "items": {
                "one": "{count} item",
                "other": "{count} items"
            }
        }));

        assert_eq!(i18n.tp("items", 1.0, &[("count", "1")]), "1 item");
        assert_eq!(i18n.tp("items", 5.0, &[("count", "5")]), "5 items");
    }

    #[test]
    fn test_i18n_service_locale_change() {
        let i18n = I18nService::new("en");

        i18n.load_for_locale("en", serde_json::json!({
            "hello": "Hello"
        }));

        i18n.load_for_locale("de", serde_json::json!({
            "hello": "Hallo"
        }));

        assert_eq!(i18n.t("hello", &[]), "Hello");

        i18n.set_locale("de");
        assert_eq!(i18n.t("hello", &[]), "Hallo");
    }

    #[test]
    fn test_i18n_formatting() {
        let i18n = I18nService::new("en-US");

        assert_eq!(i18n.format_number(1234.56), "1,234.56");
        assert_eq!(i18n.format_currency(99.99, "USD"), "$99.99");
    }

    #[test]
    fn test_icu_simple() {
        let i18n = I18nService::new("en");
        let result = i18n.icu("Hello, {name}!", &[("name", IcuValue::from("World"))]);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_icu_plural() {
        let i18n = I18nService::new("en");

        let msg = "{count, plural, one{# message} other{# messages}}";

        assert_eq!(
            i18n.icu(msg, &[("count", IcuValue::Number(1.0))]),
            "1 message"
        );
        assert_eq!(
            i18n.icu(msg, &[("count", IcuValue::Number(5.0))]),
            "5 messages"
        );
    }

    #[test]
    fn test_icu_select() {
        let i18n = I18nService::new("en");

        let msg = "{gender, select, male{He} female{She} other{They}} liked your post";

        assert_eq!(
            i18n.icu(msg, &[("gender", IcuValue::from("male"))]),
            "He liked your post"
        );
        assert_eq!(
            i18n.icu(msg, &[("gender", IcuValue::from("female"))]),
            "She liked your post"
        );
    }

    #[test]
    fn test_t_icu() {
        let i18n = I18nService::new("en");

        i18n.load(serde_json::json!({
            "messages": "{count, plural, one{You have # message} other{You have # messages}}"
        }));

        assert_eq!(
            i18n.t_icu("messages", &[("count", IcuValue::Number(1.0))]),
            "You have 1 message"
        );
        assert_eq!(
            i18n.t_icu("messages", &[("count", IcuValue::Number(42.0))]),
            "You have 42 messages"
        );
    }

    #[test]
    fn test_compile_icu() {
        let i18n = I18nService::new("en");
        let msg = i18n.compile_icu("{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}").unwrap();

        assert_eq!(i18n.format_compiled(&msg, &[("n", IcuValue::Number(1.0))]), "1st");
        assert_eq!(i18n.format_compiled(&msg, &[("n", IcuValue::Number(2.0))]), "2nd");
        assert_eq!(i18n.format_compiled(&msg, &[("n", IcuValue::Number(3.0))]), "3rd");
        assert_eq!(i18n.format_compiled(&msg, &[("n", IcuValue::Number(4.0))]), "4th");
    }
}

