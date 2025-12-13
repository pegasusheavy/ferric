//! # Internationalization (i18n) Module
//!
//! This module provides comprehensive internationalization support for Ferric applications,
//! including translation management, pluralization, number/date formatting, and locale handling.
//!
//! ## Features
//!
//! - **Translation Management**: Load, cache, and retrieve translations
//! - **Pluralization**: Language-aware plural form selection
//! - **Formatting**: Locale-aware number and date formatting
//! - **Reactive Integration**: Signals that update when locale changes
//! - **DI Integration**: Injectable I18nService for component use
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_core::i18n::*;
//!
//! // Initialize the i18n service
//! let i18n = I18nService::new("en");
//!
//! // Load translations
//! i18n.load_translations("en", json!({
//!     "greeting": "Hello, {name}!",
//!     "items": {
//!         "one": "{count} item",
//!         "other": "{count} items"
//!     }
//! }));
//!
//! // Translate with interpolation
//! let msg = i18n.t("greeting", &[("name", "World")]);
//! // => "Hello, World!"
//!
//! // Pluralization
//! let items = i18n.tp("items", 5, &[("count", "5")]);
//! // => "5 items"
//! ```
//!
//! ## Translation File Format
//!
//! Translations can be loaded from JSON:
//!
//! ```json
//! {
//!   "common": {
//!     "save": "Save",
//!     "cancel": "Cancel",
//!     "delete": "Delete"
//!   },
//!   "errors": {
//!     "required": "This field is required",
//!     "invalid_email": "Please enter a valid email address"
//!   },
//!   "items": {
//!     "zero": "No items",
//!     "one": "{count} item",
//!     "other": "{count} items"
//!   }
//! }
//! ```
//!
//! ## Locale-Aware Formatting
//!
//! ```ignore
//! let i18n = I18nService::new("de-DE");
//!
//! // Number formatting
//! i18n.format_number(1234567.89); // => "1.234.567,89"
//!
//! // Currency formatting
//! i18n.format_currency(99.99, "EUR"); // => "99,99 €"
//!
//! // Date formatting
//! i18n.format_date(date, DateFormat::Long); // => "15. Januar 2024"
//! ```

mod format;
pub mod icu;
mod locale;
mod plural;
mod service;
mod translate;

pub use format::{DateFormat, NumberFormat, format_number, format_currency, format_date};
pub use icu::{
    // Core types
    IcuMessage, IcuValue, IcuError, IcuResult,
    // Argument types
    ArgType, Argument, MessagePart,
    // Number/Date styles
    NumberStyle, DateStyle,
    // Plural types
    PluralArg, PluralCase, PluralSelector,
    // Select types
    SelectArg, SelectCase,
    // Functions
    compile as icu_compile, format as icu_format, format_map as icu_format_map,
};
pub use locale::{Locale, LocaleInfo, parse_locale, get_browser_locale};
pub use plural::{PluralCategory, PluralRules, get_plural_category, get_ordinal_category};
pub use service::I18nService;
pub use translate::{
    TranslationStore, TranslationKey, TranslationValue,
    t, tp, translate, translate_plural,
};

