//! Pluralization rules for different languages.
//!
//! Based on CLDR plural rules: https://cldr.unicode.org/index/cldr-spec/plural-rules

/// Plural categories as defined by CLDR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    /// Used for zero items (in some languages).
    Zero,
    /// Used for one item.
    One,
    /// Used for two items (in some languages like Arabic).
    Two,
    /// Used for "few" items (in some Slavic languages).
    Few,
    /// Used for "many" items (in some Slavic languages).
    Many,
    /// Default category for all other cases.
    Other,
}

impl PluralCategory {
    /// Get the string key for this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

impl std::fmt::Display for PluralCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Plural rules for a specific language.
pub trait PluralRules {
    /// Get the plural category for a given count.
    fn select(&self, n: f64) -> PluralCategory;

    /// Get all categories used by this language.
    fn categories(&self) -> Vec<PluralCategory>;
}

/// Get the plural category for a given count in a language.
pub fn get_plural_category(language: &str, n: f64) -> PluralCategory {
    // Extract just the language code (not region)
    let lang = language.split('-').next().unwrap_or(language);

    match lang.to_lowercase().as_str() {
        // Languages with only "one" and "other"
        "en" | "de" | "nl" | "sv" | "da" | "no" | "nb" | "nn" | "fi" | "et" |
        "hu" | "tr" | "es" | "it" | "pt" | "el" | "bg" | "ca" | "eu" | "gl" |
        "af" | "sq" | "ast" | "az" | "bn" | "chr" | "eo" | "fo" | "fur" |
        "fy" | "gu" | "ha" | "haw" | "hi" | "ia" | "id" | "jv" | "kk" |
        "kn" | "ku" | "ky" | "lb" | "lg" | "ml" | "mn" | "mr" | "ms" |
        "ne" | "om" | "or" | "pa" | "pap" | "ps" | "rm" | "rof" | "saq" |
        "seh" | "so" | "sw" | "syr" | "ta" | "te" | "th" | "ti" | "tk" |
        "ur" | "vo" | "wae" | "xog" | "zu" => {
            if n.abs() == 1.0 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }

        // French, Brazilian Portuguese: 0 and 1 are singular
        "fr" => {
            if n >= 0.0 && n < 2.0 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }

        // Latvian: special rules for 0, 1, 11-19
        "lv" => {
            let n_mod10 = (n as i64) % 10;
            let n_mod100 = (n as i64) % 100;

            if n_mod10 == 0 || (n_mod100 >= 11 && n_mod100 <= 19) {
                PluralCategory::Zero
            } else if n_mod10 == 1 && n_mod100 != 11 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }

        // Russian, Ukrainian, Belarusian, Serbian, Croatian, Bosnian
        "ru" | "uk" | "be" | "sr" | "hr" | "bs" => {
            let n_i = n as i64;
            let n_mod10 = n_i % 10;
            let n_mod100 = n_i % 100;

            if n_mod10 == 1 && n_mod100 != 11 {
                PluralCategory::One
            } else if n_mod10 >= 2 && n_mod10 <= 4 && !(n_mod100 >= 12 && n_mod100 <= 14) {
                PluralCategory::Few
            } else if n_mod10 == 0 || (n_mod10 >= 5 && n_mod10 <= 9) || (n_mod100 >= 11 && n_mod100 <= 14) {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }

        // Polish
        "pl" => {
            let n_i = n as i64;
            let n_mod10 = n_i % 10;
            let n_mod100 = n_i % 100;

            if n_i == 1 {
                PluralCategory::One
            } else if n_mod10 >= 2 && n_mod10 <= 4 && !(n_mod100 >= 12 && n_mod100 <= 14) {
                PluralCategory::Few
            } else if n_i != 1 && (n_mod10 == 0 || n_mod10 == 1) || (n_mod10 >= 5 && n_mod10 <= 9) || (n_mod100 >= 12 && n_mod100 <= 14) {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }

        // Czech, Slovak
        "cs" | "sk" => {
            let n_i = n as i64;
            if n_i == 1 {
                PluralCategory::One
            } else if n_i >= 2 && n_i <= 4 {
                PluralCategory::Few
            } else {
                PluralCategory::Other
            }
        }

        // Arabic
        "ar" => {
            let n_i = n as i64;
            let n_mod100 = n_i % 100;

            if n_i == 0 {
                PluralCategory::Zero
            } else if n_i == 1 {
                PluralCategory::One
            } else if n_i == 2 {
                PluralCategory::Two
            } else if n_mod100 >= 3 && n_mod100 <= 10 {
                PluralCategory::Few
            } else if n_mod100 >= 11 && n_mod100 <= 99 {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }

        // Welsh
        "cy" => {
            let n_i = n as i64;
            if n_i == 0 {
                PluralCategory::Zero
            } else if n_i == 1 {
                PluralCategory::One
            } else if n_i == 2 {
                PluralCategory::Two
            } else if n_i == 3 {
                PluralCategory::Few
            } else if n_i == 6 {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }

        // Irish
        "ga" => {
            let n_i = n as i64;
            if n_i == 1 {
                PluralCategory::One
            } else if n_i == 2 {
                PluralCategory::Two
            } else if n_i >= 3 && n_i <= 6 {
                PluralCategory::Few
            } else if n_i >= 7 && n_i <= 10 {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }

        // Lithuanian
        "lt" => {
            let n_i = n as i64;
            let n_mod10 = n_i % 10;
            let n_mod100 = n_i % 100;

            if n_mod10 == 1 && !(n_mod100 >= 11 && n_mod100 <= 19) {
                PluralCategory::One
            } else if n_mod10 >= 2 && n_mod10 <= 9 && !(n_mod100 >= 11 && n_mod100 <= 19) {
                PluralCategory::Few
            } else {
                PluralCategory::Other
            }
        }

        // Slovenian
        "sl" => {
            let n_i = n as i64;
            let n_mod100 = n_i % 100;

            if n_mod100 == 1 {
                PluralCategory::One
            } else if n_mod100 == 2 {
                PluralCategory::Two
            } else if n_mod100 >= 3 && n_mod100 <= 4 {
                PluralCategory::Few
            } else {
                PluralCategory::Other
            }
        }

        // Maltese
        "mt" => {
            let n_i = n as i64;
            let n_mod100 = n_i % 100;

            if n_i == 1 {
                PluralCategory::One
            } else if n_i == 0 || (n_mod100 >= 2 && n_mod100 <= 10) {
                PluralCategory::Few
            } else if n_mod100 >= 11 && n_mod100 <= 19 {
                PluralCategory::Many
            } else {
                PluralCategory::Other
            }
        }

        // Hebrew
        "he" | "iw" => {
            let n_i = n as i64;
            if n_i == 1 {
                PluralCategory::One
            } else if n_i == 2 {
                PluralCategory::Two
            } else {
                PluralCategory::Other
            }
        }

        // Chinese, Japanese, Korean, Vietnamese, Lao, Burmese - no plurals
        // Note: Thai (th), Indonesian (id), Malay (ms) are handled above in the one/other case
        "zh" | "ja" | "ko" | "vi" | "lo" | "my" => {
            PluralCategory::Other
        }

        // Default: simple one/other
        _ => {
            if n.abs() == 1.0 {
                PluralCategory::One
            } else {
                PluralCategory::Other
            }
        }
    }
}

/// Get ordinal category (1st, 2nd, 3rd, etc.) for a language.
pub fn get_ordinal_category(language: &str, n: f64) -> PluralCategory {
    let lang = language.split('-').next().unwrap_or(language);

    match lang.to_lowercase().as_str() {
        "en" => {
            let n_i = n as i64;
            let n_mod10 = n_i % 10;
            let n_mod100 = n_i % 100;

            if n_mod10 == 1 && n_mod100 != 11 {
                PluralCategory::One // 1st, 21st, 31st...
            } else if n_mod10 == 2 && n_mod100 != 12 {
                PluralCategory::Two // 2nd, 22nd, 32nd...
            } else if n_mod10 == 3 && n_mod100 != 13 {
                PluralCategory::Few // 3rd, 23rd, 33rd...
            } else {
                PluralCategory::Other // 4th, 11th, 12th, 13th...
            }
        }

        // Most languages don't have ordinal variations
        _ => PluralCategory::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_plurals() {
        assert_eq!(get_plural_category("en", 0.0), PluralCategory::Other);
        assert_eq!(get_plural_category("en", 1.0), PluralCategory::One);
        assert_eq!(get_plural_category("en", 2.0), PluralCategory::Other);
        assert_eq!(get_plural_category("en", 5.0), PluralCategory::Other);
    }

    #[test]
    fn test_russian_plurals() {
        assert_eq!(get_plural_category("ru", 1.0), PluralCategory::One);
        assert_eq!(get_plural_category("ru", 2.0), PluralCategory::Few);
        assert_eq!(get_plural_category("ru", 5.0), PluralCategory::Many);
        assert_eq!(get_plural_category("ru", 11.0), PluralCategory::Many);
        assert_eq!(get_plural_category("ru", 21.0), PluralCategory::One);
        assert_eq!(get_plural_category("ru", 22.0), PluralCategory::Few);
    }

    #[test]
    fn test_arabic_plurals() {
        assert_eq!(get_plural_category("ar", 0.0), PluralCategory::Zero);
        assert_eq!(get_plural_category("ar", 1.0), PluralCategory::One);
        assert_eq!(get_plural_category("ar", 2.0), PluralCategory::Two);
        assert_eq!(get_plural_category("ar", 5.0), PluralCategory::Few);
        assert_eq!(get_plural_category("ar", 15.0), PluralCategory::Many);
    }

    #[test]
    fn test_english_ordinals() {
        assert_eq!(get_ordinal_category("en", 1.0), PluralCategory::One);
        assert_eq!(get_ordinal_category("en", 2.0), PluralCategory::Two);
        assert_eq!(get_ordinal_category("en", 3.0), PluralCategory::Few);
        assert_eq!(get_ordinal_category("en", 4.0), PluralCategory::Other);
        assert_eq!(get_ordinal_category("en", 11.0), PluralCategory::Other);
        assert_eq!(get_ordinal_category("en", 21.0), PluralCategory::One);
    }
}

