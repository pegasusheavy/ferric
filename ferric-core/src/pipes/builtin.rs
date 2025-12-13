//! Built-in pipes for common transformations.

use super::transform::{Pipe, PipeArgs};

// ============================================================================
// Text Pipes
// ============================================================================

/// Convert string to uppercase.
pub struct UppercasePipe;

impl Pipe for UppercasePipe {
    fn name(&self) -> &'static str {
        "uppercase"
    }

    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        value.to_uppercase()
    }
}

/// Convert string to lowercase.
pub struct LowercasePipe;

impl Pipe for LowercasePipe {
    fn name(&self) -> &'static str {
        "lowercase"
    }

    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        value.to_lowercase()
    }
}

/// Convert string to title case.
pub struct TitlecasePipe;

impl Pipe for TitlecasePipe {
    fn name(&self) -> &'static str {
        "titlecase"
    }

    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        value
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Trim whitespace from string.
pub struct TrimPipe;

impl Pipe for TrimPipe {
    fn name(&self) -> &'static str {
        "trim"
    }

    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        value.trim().to_string()
    }
}

/// Extract substring or subarray.
/// Usage: `value | slice:start` or `value | slice:start:end`
pub struct SlicePipe;

impl Pipe for SlicePipe {
    fn name(&self) -> &'static str {
        "slice"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let start = args.get_i64(0).unwrap_or(0) as usize;
        let chars: Vec<char> = value.chars().collect();
        let len = chars.len();

        let start = start.min(len);
        let end = args
            .get_i64(1)
            .map(|e| (e as usize).min(len))
            .unwrap_or(len);

        if start >= end {
            return String::new();
        }

        chars[start..end].iter().collect()
    }
}

// ============================================================================
// Number Pipes
// ============================================================================

/// Format a number with locale-aware formatting.
/// Usage: `value | number` or `value | number:'1.2-2'` (minInt.minFrac-maxFrac)
pub struct NumberPipe;

impl Pipe for NumberPipe {
    fn name(&self) -> &'static str {
        "number"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let num: f64 = match value.parse() {
            Ok(n) => n,
            Err(_) => return value.to_string(),
        };

        let format = args.get(0).unwrap_or("1.0-3");
        let (min_int, min_frac, max_frac) = parse_number_format(format);

        format_number(num, min_int, min_frac, max_frac)
    }
}

/// Format a number as currency.
/// Usage: `value | currency` or `value | currency:'USD'` or `value | currency:'EUR':'symbol'`
pub struct CurrencyPipe;

impl Pipe for CurrencyPipe {
    fn name(&self) -> &'static str {
        "currency"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let num: f64 = match value.parse() {
            Ok(n) => n,
            Err(_) => return value.to_string(),
        };

        let currency_code = args.get(0).unwrap_or("USD");
        let display = args.get(1).unwrap_or("symbol");

        let symbol = match (currency_code, display) {
            ("USD", "symbol") => "$",
            ("EUR", "symbol") => "€",
            ("GBP", "symbol") => "£",
            ("JPY", "symbol") | ("CNY", "symbol") => "¥",
            (code, _) => code,
        };

        let formatted = format_number(num.abs(), 1, 2, 2);
        let sign = if num < 0.0 { "-" } else { "" };

        format!("{}{}{}", sign, symbol, formatted)
    }
}

/// Format a number as a percentage.
/// Usage: `value | percent` or `value | percent:'1.0-2'`
pub struct PercentPipe;

impl Pipe for PercentPipe {
    fn name(&self) -> &'static str {
        "percent"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let num: f64 = match value.parse() {
            Ok(n) => n,
            Err(_) => return value.to_string(),
        };

        let format = args.get(0).unwrap_or("1.0-2");
        let (min_int, min_frac, max_frac) = parse_number_format(format);

        let percentage = num * 100.0;
        format!("{}%", format_number(percentage, min_int, min_frac, max_frac))
    }
}

// ============================================================================
// Date Pipes
// ============================================================================

/// Format a date/timestamp.
/// Usage: `value | date` or `value | date:'short'` or `value | date:'yyyy-MM-dd'`
pub struct DatePipe;

impl Pipe for DatePipe {
    fn name(&self) -> &'static str {
        "date"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        // Try to parse as timestamp (seconds since epoch)
        let timestamp: i64 = match value.parse() {
            Ok(t) => t,
            Err(_) => return value.to_string(), // Return as-is if not parseable
        };

        let format = args.get(0).unwrap_or("medium");

        format_timestamp(timestamp, format)
    }
}

// ============================================================================
// Utility Pipes
// ============================================================================

/// Convert value to JSON string.
pub struct JsonPipe;

impl Pipe for JsonPipe {
    fn name(&self) -> &'static str {
        "json"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let pretty = args.get(0).map(|s| s == "pretty").unwrap_or(false);

        // If it's already valid JSON, format it
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) {
            if pretty {
                serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| value.to_string())
            } else {
                serde_json::to_string(&parsed).unwrap_or_else(|_| value.to_string())
            }
        } else {
            // Treat as string and JSON-encode it
            serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value))
        }
    }
}

/// Provide a default value if input is empty or null.
/// Usage: `value | default:'N/A'`
pub struct DefaultPipe;

impl Pipe for DefaultPipe {
    fn name(&self) -> &'static str {
        "default"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        if value.is_empty() || value == "null" || value == "undefined" {
            args.get(0).unwrap_or("").to_string()
        } else {
            value.to_string()
        }
    }
}

/// Replace occurrences in a string.
/// Usage: `value | replace:'old':'new'`
pub struct ReplacePipe;

impl Pipe for ReplacePipe {
    fn name(&self) -> &'static str {
        "replace"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let search = args.get(0).unwrap_or("");
        let replacement = args.get(1).unwrap_or("");

        value.replace(search, replacement)
    }
}

/// Pad string at start.
/// Usage: `value | padStart:5:'0'`
pub struct PadStartPipe;

impl Pipe for PadStartPipe {
    fn name(&self) -> &'static str {
        "padStart"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let length = args.get_i64(0).unwrap_or(0) as usize;
        let pad_char = args.get(1).unwrap_or(" ");
        let pad_char = pad_char.chars().next().unwrap_or(' ');

        if value.len() >= length {
            return value.to_string();
        }

        let padding: String = std::iter::repeat(pad_char)
            .take(length - value.len())
            .collect();
        format!("{}{}", padding, value)
    }
}

/// Pad string at end.
/// Usage: `value | padEnd:10:'.'`
pub struct PadEndPipe;

impl Pipe for PadEndPipe {
    fn name(&self) -> &'static str {
        "padEnd"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let length = args.get_i64(0).unwrap_or(0) as usize;
        let pad_char = args.get(1).unwrap_or(" ");
        let pad_char = pad_char.chars().next().unwrap_or(' ');

        if value.len() >= length {
            return value.to_string();
        }

        let padding: String = std::iter::repeat(pad_char)
            .take(length - value.len())
            .collect();
        format!("{}{}", value, padding)
    }
}

/// Truncate string with ellipsis.
/// Usage: `value | truncate:20` or `value | truncate:20:'...'`
pub struct TruncatePipe;

impl Pipe for TruncatePipe {
    fn name(&self) -> &'static str {
        "truncate"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let max_length = args.get_i64(0).unwrap_or(50) as usize;
        let suffix = args.get(1).unwrap_or("...");

        if value.len() <= max_length {
            return value.to_string();
        }

        let truncate_at = max_length.saturating_sub(suffix.len());
        let truncated: String = value.chars().take(truncate_at).collect();
        format!("{}{}", truncated, suffix)
    }
}

/// Reverse a string.
pub struct ReversePipe;

impl Pipe for ReversePipe {
    fn name(&self) -> &'static str {
        "reverse"
    }

    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        value.chars().rev().collect()
    }
}

/// Repeat a string.
/// Usage: `value | repeat:3`
pub struct RepeatPipe;

impl Pipe for RepeatPipe {
    fn name(&self) -> &'static str {
        "repeat"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let count = args.get_i64(0).unwrap_or(1) as usize;
        value.repeat(count)
    }
}

/// Join array elements.
/// Usage: `value | join:','`
pub struct JoinPipe;

impl Pipe for JoinPipe {
    fn name(&self) -> &'static str {
        "join"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let separator = args.get(0).unwrap_or(",");

        // Try to parse as JSON array
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(value) {
            arr.iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect::<Vec<_>>()
                .join(separator)
        } else {
            value.to_string()
        }
    }
}

/// Split string into array.
/// Usage: `value | split:','`
pub struct SplitPipe;

impl Pipe for SplitPipe {
    fn name(&self) -> &'static str {
        "split"
    }

    fn transform(&self, value: &str, args: &PipeArgs) -> String {
        let separator = args.get(0).unwrap_or(",");
        let parts: Vec<&str> = value.split(separator).collect();
        serde_json::to_string(&parts).unwrap_or_else(|_| value.to_string())
    }
}

/// Convert object to key-value pairs for iteration.
pub struct KeyvaluePipe;

impl Pipe for KeyvaluePipe {
    fn name(&self) -> &'static str {
        "keyvalue"
    }

    fn transform(&self, value: &str, _args: &PipeArgs) -> String {
        if let Ok(obj) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(value) {
            let pairs: Vec<serde_json::Value> = obj
                .into_iter()
                .map(|(k, v)| {
                    serde_json::json!({
                        "key": k,
                        "value": v
                    })
                })
                .collect();
            serde_json::to_string(&pairs).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse number format string like "1.2-2" into (minInt, minFrac, maxFrac).
fn parse_number_format(format: &str) -> (usize, usize, usize) {
    let parts: Vec<&str> = format.split('.').collect();

    let min_int = parts
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let (min_frac, max_frac) = if let Some(frac_part) = parts.get(1) {
        let frac_parts: Vec<&str> = frac_part.split('-').collect();
        let min = frac_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let max = frac_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(min);
        (min, max)
    } else {
        (0, 3)
    };

    (min_int, min_frac, max_frac)
}

/// Format a number with specified precision.
fn format_number(num: f64, min_int: usize, min_frac: usize, max_frac: usize) -> String {
    // Round to max fractional digits
    let multiplier = 10f64.powi(max_frac as i32);
    let rounded = (num * multiplier).round() / multiplier;

    // Format with max fractional digits
    let formatted = format!("{:.prec$}", rounded, prec = max_frac);

    // Split into integer and fractional parts
    let parts: Vec<&str> = formatted.split('.').collect();
    let mut int_part = parts[0].to_string();
    let frac_part = parts.get(1).copied().unwrap_or("");

    // Pad integer part
    while int_part.len() < min_int {
        int_part.insert(0, '0');
    }

    // Add thousand separators
    let int_with_sep = add_thousand_separators(&int_part);

    // Trim trailing zeros from fractional part, but keep min_frac
    let mut frac_trimmed = frac_part.trim_end_matches('0').to_string();
    while frac_trimmed.len() < min_frac {
        frac_trimmed.push('0');
    }

    if frac_trimmed.is_empty() {
        int_with_sep
    } else {
        format!("{}.{}", int_with_sep, frac_trimmed)
    }
}

/// Add thousand separators to integer string.
fn add_thousand_separators(int_str: &str) -> String {
    let is_negative = int_str.starts_with('-');
    let digits: &str = if is_negative { &int_str[1..] } else { int_str };

    let mut result = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }

    if is_negative {
        result.insert(0, '-');
    }

    result
}

/// Format a Unix timestamp.
fn format_timestamp(timestamp: i64, format: &str) -> String {
    // Simple date calculation from Unix timestamp
    let secs_per_day = 86400;
    let days_since_epoch = timestamp / secs_per_day;

    // Calculate year, month, day
    let (year, month, day, weekday) = days_to_ymd(days_since_epoch);

    // Calculate time components
    let remaining_secs = timestamp % secs_per_day;
    let hours = (remaining_secs / 3600) % 24;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    match format {
        "short" => format!("{}/{}/{}", month, day, year % 100),
        "medium" => {
            let month_name = month_abbrev(month as u8);
            format!("{} {}, {}", month_name, day, year)
        }
        "long" => {
            let month_name = month_name(month as u8);
            format!("{} {}, {}", month_name, day, year)
        }
        "full" => {
            let day_name = weekday_name(weekday);
            let month_name = month_name(month as u8);
            format!("{}, {} {}, {}", day_name, month_name, day, year)
        }
        "shortTime" => {
            let (hour12, ampm) = to_12_hour(hours as u8);
            format!("{}:{:02} {}", hour12, minutes, ampm)
        }
        "mediumTime" => {
            let (hour12, ampm) = to_12_hour(hours as u8);
            format!("{}:{:02}:{:02} {}", hour12, minutes, seconds, ampm)
        }
        "shortDate" => format!("{}/{}/{}", month, day, year % 100),
        "mediumDate" => {
            let month_name = month_abbrev(month as u8);
            format!("{} {}, {}", month_name, day, year)
        }
        "longDate" => {
            let month_name = month_name(month as u8);
            format!("{} {}, {}", month_name, day, year)
        }
        "fullDate" => {
            let day_name = weekday_name(weekday);
            let month_name = month_name(month as u8);
            format!("{}, {} {}, {}", day_name, month_name, day, year)
        }
        _ => {
            // Custom format (simplified)
            format
                .replace("yyyy", &format!("{:04}", year))
                .replace("yy", &format!("{:02}", year % 100))
                .replace("MM", &format!("{:02}", month))
                .replace("M", &month.to_string())
                .replace("dd", &format!("{:02}", day))
                .replace("d", &day.to_string())
                .replace("HH", &format!("{:02}", hours))
                .replace("H", &hours.to_string())
                .replace("mm", &format!("{:02}", minutes))
                .replace("m", &minutes.to_string())
                .replace("ss", &format!("{:02}", seconds))
                .replace("s", &seconds.to_string())
        }
    }
}

/// Convert days since epoch to year, month, day, weekday.
fn days_to_ymd(days: i64) -> (i64, i64, i64, u8) {
    let mut remaining_days = days;
    let mut year = 1970;

    // Calculate year
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    // Calculate month and day
    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;
    let weekday = ((days + 4) % 7) as u8; // Jan 1, 1970 was Thursday (4)

    (year, month, day, weekday)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn month_abbrev(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn weekday_name(weekday: u8) -> &'static str {
    match weekday {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Unknown",
    }
}

fn to_12_hour(hour: u8) -> (u8, &'static str) {
    match hour {
        0 => (12, "AM"),
        1..=11 => (hour, "AM"),
        12 => (12, "PM"),
        13..=23 => (hour - 12, "PM"),
        _ => (hour, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uppercase() {
        let pipe = UppercasePipe;
        assert_eq!(pipe.transform("hello", &PipeArgs::new()), "HELLO");
    }

    #[test]
    fn test_lowercase() {
        let pipe = LowercasePipe;
        assert_eq!(pipe.transform("HELLO", &PipeArgs::new()), "hello");
    }

    #[test]
    fn test_titlecase() {
        let pipe = TitlecasePipe;
        assert_eq!(
            pipe.transform("hello world", &PipeArgs::new()),
            "Hello World"
        );
    }

    #[test]
    fn test_slice() {
        let pipe = SlicePipe;
        let args = PipeArgs::from_vec(vec!["0".to_string(), "5".to_string()]);
        assert_eq!(pipe.transform("hello world", &args), "hello");
    }

    #[test]
    fn test_number() {
        let pipe = NumberPipe;
        assert_eq!(pipe.transform("1234.5678", &PipeArgs::new()), "1,234.568");

        let args = PipeArgs::from_vec(vec!["1.2-2".to_string()]);
        assert_eq!(pipe.transform("1234.5", &args), "1,234.50");
    }

    #[test]
    fn test_currency() {
        let pipe = CurrencyPipe;
        let args = PipeArgs::from_vec(vec!["USD".to_string()]);
        assert_eq!(pipe.transform("99.99", &args), "$99.99");
    }

    #[test]
    fn test_percent() {
        let pipe = PercentPipe;
        assert_eq!(pipe.transform("0.25", &PipeArgs::new()), "25%");
    }

    #[test]
    fn test_date() {
        let pipe = DatePipe;
        // Jan 15, 2024 00:00:00 UTC = 1705276800
        let args = PipeArgs::from_vec(vec!["short".to_string()]);
        assert_eq!(pipe.transform("1705276800", &args), "1/15/24");
    }

    #[test]
    fn test_default() {
        let pipe = DefaultPipe;
        let args = PipeArgs::from_vec(vec!["N/A".to_string()]);
        assert_eq!(pipe.transform("", &args), "N/A");
        assert_eq!(pipe.transform("value", &args), "value");
    }

    #[test]
    fn test_truncate() {
        let pipe = TruncatePipe;
        let args = PipeArgs::from_vec(vec!["10".to_string()]);
        assert_eq!(
            pipe.transform("This is a long string", &args),
            "This is..."
        );
    }

    #[test]
    fn test_replace() {
        let pipe = ReplacePipe;
        let args = PipeArgs::from_vec(vec!["world".to_string(), "Rust".to_string()]);
        assert_eq!(pipe.transform("Hello world", &args), "Hello Rust");
    }

    #[test]
    fn test_reverse() {
        let pipe = ReversePipe;
        assert_eq!(pipe.transform("hello", &PipeArgs::new()), "olleh");
    }

    #[test]
    fn test_pad_start() {
        let pipe = PadStartPipe;
        let args = PipeArgs::from_vec(vec!["5".to_string(), "0".to_string()]);
        assert_eq!(pipe.transform("42", &args), "00042");
    }
}

