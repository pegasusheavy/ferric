//! Locale-aware formatting for numbers, dates, and currencies.

/// Number formatting options.
#[derive(Debug, Clone)]
pub struct NumberFormat {
    /// Minimum integer digits.
    pub min_integer_digits: usize,
    /// Minimum fraction digits.
    pub min_fraction_digits: usize,
    /// Maximum fraction digits.
    pub max_fraction_digits: usize,
    /// Use grouping separators (e.g., "1,000").
    pub use_grouping: bool,
}

impl Default for NumberFormat {
    fn default() -> Self {
        Self {
            min_integer_digits: 1,
            min_fraction_digits: 0,
            max_fraction_digits: 3,
            use_grouping: true,
        }
    }
}

impl NumberFormat {
    /// Create a new number format.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum integer digits.
    pub fn min_integer_digits(mut self, digits: usize) -> Self {
        self.min_integer_digits = digits;
        self
    }

    /// Set fraction digits (both min and max).
    pub fn fraction_digits(mut self, digits: usize) -> Self {
        self.min_fraction_digits = digits;
        self.max_fraction_digits = digits;
        self
    }

    /// Set grouping.
    pub fn use_grouping(mut self, use_grouping: bool) -> Self {
        self.use_grouping = use_grouping;
        self
    }
}

/// Date formatting style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFormat {
    /// Short format (e.g., "1/15/24").
    Short,
    /// Medium format (e.g., "Jan 15, 2024").
    Medium,
    /// Long format (e.g., "January 15, 2024").
    Long,
    /// Full format (e.g., "Monday, January 15, 2024").
    Full,
    /// ISO format (e.g., "2024-01-15").
    Iso,
    /// Custom format pattern.
    Custom(String),
}

/// Time formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    /// Short format (e.g., "3:30 PM").
    Short,
    /// Medium format (e.g., "3:30:45 PM").
    Medium,
    /// Long format (e.g., "3:30:45 PM EST").
    Long,
    /// 24-hour format (e.g., "15:30").
    Hour24,
}

/// Locale-specific formatting data.
struct LocaleFormatData {
    decimal_separator: char,
    grouping_separator: char,
    currency_symbol: &'static str,
    currency_position: CurrencyPosition,
    month_names: [&'static str; 12],
    month_names_short: [&'static str; 12],
    day_names: [&'static str; 7],
    day_names_short: [&'static str; 7],
}

#[derive(Clone, Copy)]
enum CurrencyPosition {
    Before,
    After,
    AfterWithSpace,
}

fn get_locale_format_data(locale: &str) -> LocaleFormatData {
    let lang = locale.split('-').next().unwrap_or(locale);

    match lang {
        "de" => LocaleFormatData {
            decimal_separator: ',',
            grouping_separator: '.',
            currency_symbol: "€",
            currency_position: CurrencyPosition::AfterWithSpace,
            month_names: ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"],
            month_names_short: ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"],
            day_names: ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"],
            day_names_short: ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"],
        },
        "fr" => LocaleFormatData {
            decimal_separator: ',',
            grouping_separator: ' ',
            currency_symbol: "€",
            currency_position: CurrencyPosition::AfterWithSpace,
            month_names: ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"],
            month_names_short: ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."],
            day_names: ["dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi"],
            day_names_short: ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."],
        },
        "es" => LocaleFormatData {
            decimal_separator: ',',
            grouping_separator: '.',
            currency_symbol: "€",
            currency_position: CurrencyPosition::AfterWithSpace,
            month_names: ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"],
            month_names_short: ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic"],
            day_names: ["domingo", "lunes", "martes", "miércoles", "jueves", "viernes", "sábado"],
            day_names_short: ["dom", "lun", "mar", "mié", "jue", "vie", "sáb"],
        },
        "ja" => LocaleFormatData {
            decimal_separator: '.',
            grouping_separator: ',',
            currency_symbol: "¥",
            currency_position: Before,
            month_names: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
            month_names_short: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
            day_names: ["日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日"],
            day_names_short: ["日", "月", "火", "水", "木", "金", "土"],
        },
        "zh" => LocaleFormatData {
            decimal_separator: '.',
            grouping_separator: ',',
            currency_symbol: "¥",
            currency_position: CurrencyPosition::Before,
            month_names: ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"],
            month_names_short: ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"],
            day_names: ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"],
            day_names_short: ["日", "一", "二", "三", "四", "五", "六"],
        },
        _ => LocaleFormatData {
            decimal_separator: '.',
            grouping_separator: ',',
            currency_symbol: "$",
            currency_position: CurrencyPosition::Before,
            month_names: ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
            month_names_short: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
            day_names: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
            day_names_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
        },
    }
}

use CurrencyPosition::*;

/// Format a number according to locale conventions.
pub fn format_number(locale: &str, value: f64) -> String {
    format_number_with_options(locale, value, &NumberFormat::default())
}

/// Format a number with custom options.
pub fn format_number_with_options(locale: &str, value: f64, options: &NumberFormat) -> String {
    let data = get_locale_format_data(locale);

    let abs_value = value.abs();
    let is_negative = value < 0.0;

    // Split into integer and fractional parts
    let integer_part = abs_value.trunc() as i64;
    let fractional_part = abs_value.fract();

    // Format integer part with grouping
    let mut int_str = integer_part.to_string();

    // Pad with leading zeros if needed
    while int_str.len() < options.min_integer_digits {
        int_str.insert(0, '0');
    }

    // Add grouping separators
    if options.use_grouping && int_str.len() > 3 {
        let mut grouped = String::new();
        for (i, c) in int_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                grouped.insert(0, data.grouping_separator);
            }
            grouped.insert(0, c);
        }
        int_str = grouped;
    }

    // Format fractional part
    let mut result = int_str;

    if options.max_fraction_digits > 0 || options.min_fraction_digits > 0 {
        let frac_digits = options.max_fraction_digits;
        let frac_str = format!("{:.prec$}", fractional_part, prec = frac_digits);
        let frac_part = &frac_str[2..]; // Skip "0."

        // Trim trailing zeros but respect min_fraction_digits
        let mut trimmed: String = frac_part.trim_end_matches('0').to_string();
        while trimmed.len() < options.min_fraction_digits {
            trimmed.push('0');
        }

        if !trimmed.is_empty() {
            result.push(data.decimal_separator);
            result.push_str(&trimmed);
        }
    }

    if is_negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Format a currency value.
pub fn format_currency(locale: &str, value: f64, currency_code: &str) -> String {
    let data = get_locale_format_data(locale);
    let options = NumberFormat::new().fraction_digits(2);
    let formatted = format_number_with_options(locale, value.abs(), &options);

    let symbol = match currency_code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        _ => currency_code,
    };

    let is_negative = value < 0.0;
    let sign = if is_negative { "-" } else { "" };

    match data.currency_position {
        CurrencyPosition::Before => format!("{}{}{}", sign, symbol, formatted),
        CurrencyPosition::After => format!("{}{}{}", sign, formatted, symbol),
        CurrencyPosition::AfterWithSpace => format!("{}{} {}", sign, formatted, symbol),
    }
}

/// Simple date structure for formatting.
#[derive(Debug, Clone, Copy)]
pub struct SimpleDate {
    pub year: i32,
    pub month: u8, // 1-12
    pub day: u8,   // 1-31
    pub weekday: u8, // 0-6 (Sunday = 0)
}

impl SimpleDate {
    /// Create a new date.
    pub fn new(year: i32, month: u8, day: u8, weekday: u8) -> Self {
        Self { year, month, day, weekday }
    }

    /// Create from a Unix timestamp (seconds since epoch).
    pub fn from_timestamp(timestamp: i64) -> Self {
        // Simple algorithm for date calculation
        let days = timestamp / 86400;
        let mut year = 1970;
        let mut remaining_days = days;

        loop {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }

        let mut month = 1;
        let days_in_months: [i64; 12] = if is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        for &days_in_month in &days_in_months {
            if remaining_days < days_in_month {
                break;
            }
            remaining_days -= days_in_month;
            month += 1;
        }

        let day = (remaining_days + 1) as u8;
        let weekday = ((days + 4) % 7) as u8; // Jan 1, 1970 was Thursday (4)

        Self {
            year: year as i32,
            month,
            day,
            weekday,
        }
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Format a Unix timestamp (milliseconds) as a date.
pub fn format_timestamp(timestamp: i64, format: &DateFormat, locale: &str) -> String {
    let date = SimpleDate::from_timestamp(timestamp / 1000);
    format_date(locale, &date, format.clone())
}

/// Format a date according to locale conventions.
pub fn format_date(locale: &str, date: &SimpleDate, format: DateFormat) -> String {
    let data = get_locale_format_data(locale);
    let month_idx = (date.month as usize).saturating_sub(1).min(11);
    let day_idx = (date.weekday as usize).min(6);

    match format {
        DateFormat::Short => {
            // Locale-specific short format
            let lang = locale.split('-').next().unwrap_or(locale);
            match lang {
                "de" | "fr" | "es" => format!("{:02}.{:02}.{}", date.day, date.month, date.year),
                "ja" | "zh" => format!("{}/{}/{}", date.year, date.month, date.day),
                _ => format!("{}/{}/{}", date.month, date.day, date.year % 100),
            }
        }
        DateFormat::Medium => {
            format!(
                "{} {}, {}",
                data.month_names_short[month_idx],
                date.day,
                date.year
            )
        }
        DateFormat::Long => {
            format!(
                "{} {}, {}",
                data.month_names[month_idx],
                date.day,
                date.year
            )
        }
        DateFormat::Full => {
            format!(
                "{}, {} {}, {}",
                data.day_names[day_idx],
                data.month_names[month_idx],
                date.day,
                date.year
            )
        }
        DateFormat::Iso => {
            format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
        }
        DateFormat::Custom(pattern) => {
            // Simple pattern replacement
            pattern
                .replace("YYYY", &format!("{:04}", date.year))
                .replace("YY", &format!("{:02}", date.year % 100))
                .replace("MM", &format!("{:02}", date.month))
                .replace("M", &date.month.to_string())
                .replace("DD", &format!("{:02}", date.day))
                .replace("D", &date.day.to_string())
                .replace("MMMM", data.month_names[month_idx])
                .replace("MMM", data.month_names_short[month_idx])
                .replace("EEEE", data.day_names[day_idx])
                .replace("EEE", data.day_names_short[day_idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_english() {
        assert_eq!(format_number("en", 1234567.89), "1,234,567.89");
        assert_eq!(format_number("en", 0.5), "0.5");
        assert_eq!(format_number("en", -1000.0), "-1,000");
    }

    #[test]
    fn test_format_number_german() {
        assert_eq!(format_number("de", 1234567.89), "1.234.567,89");
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency("en", 99.99, "USD"), "$99.99");
        assert_eq!(format_currency("de", 99.99, "EUR"), "99,99 €");
    }

    #[test]
    fn test_format_date() {
        let date = SimpleDate::new(2024, 1, 15, 1); // Monday

        assert_eq!(format_date("en", &date, DateFormat::Short), "1/15/24");
        assert_eq!(format_date("en", &date, DateFormat::Medium), "Jan 15, 2024");
        assert_eq!(format_date("en", &date, DateFormat::Long), "January 15, 2024");
        assert_eq!(format_date("en", &date, DateFormat::Iso), "2024-01-15");
    }
}

