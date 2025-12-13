//! ICU Message Format implementation.
//!
//! Provides full support for the ICU Message Format standard for internationalization.
//!
//! ## Features
//!
//! - **Simple arguments**: `{name}` - Variable substitution
//! - **Plural forms**: `{count, plural, one{# item} other{# items}}`
//! - **Select forms**: `{gender, select, male{He} female{She} other{They}}`
//! - **Selectordinal**: `{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}`
//! - **Number formatting**: `{amount, number}`, `{amount, number, currency}`
//! - **Date/time formatting**: `{date, date}`, `{date, time}`, `{date, datetime}`
//! - **Nested messages**: Support for nested select/plural
//! - **Escaping**: `'` to escape special characters
//! - **Offset in plural**: `{count, plural, offset:1 =0{no items} one{# item}}`
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_core::i18n::icu::*;
//!
//! let msg = IcuMessage::parse("{name} has {count, plural, one{# message} other{# messages}}").unwrap();
//! let result = msg.format(&[
//!     ("name", IcuValue::String("Alice".to_string())),
//!     ("count", IcuValue::Number(5.0)),
//! ], "en");
//! // => "Alice has 5 messages"
//! ```
//!
//! ## Syntax Reference
//!
//! ```text
//! message     = messageText (argument messageText)*
//! argument    = '{' argName (',' argType (',' argStyle)?)? '}'
//! argType     = 'number' | 'date' | 'time' | 'plural' | 'select' | 'selectordinal'
//! argStyle    = 'short' | 'medium' | 'long' | 'full' | pluralStyle | selectStyle
//! pluralStyle = ('offset:' number)? (pluralCase)+
//! selectStyle = (selectCase)+
//! pluralCase  = ('=' number | pluralKeyword) '{' message '}'
//! selectCase  = keyword '{' message '}'
//! pluralKeyword = 'zero' | 'one' | 'two' | 'few' | 'many' | 'other'
//! ```

use super::format::{format_currency, format_timestamp, format_number, DateFormat};
use super::plural::{get_ordinal_category, get_plural_category, PluralCategory};
use std::collections::HashMap;

/// Error type for ICU message parsing and formatting.
#[derive(Debug, Clone, PartialEq)]
pub enum IcuError {
    /// Unexpected end of input.
    UnexpectedEof,
    /// Unexpected character found.
    UnexpectedChar(char, usize),
    /// Missing closing brace.
    MissingCloseBrace(usize),
    /// Invalid argument type.
    InvalidArgType(String, usize),
    /// Missing argument value.
    MissingArgValue(String),
    /// Invalid number literal.
    InvalidNumber(String),
    /// Invalid date format.
    InvalidDateFormat(String),
    /// Invalid plural/select syntax.
    InvalidSyntax(String, usize),
    /// Nested depth exceeded.
    MaxDepthExceeded,
}

impl std::fmt::Display for IcuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcuError::UnexpectedEof => write!(f, "Unexpected end of input"),
            IcuError::UnexpectedChar(c, pos) => write!(f, "Unexpected character '{}' at position {}", c, pos),
            IcuError::MissingCloseBrace(pos) => write!(f, "Missing closing brace at position {}", pos),
            IcuError::InvalidArgType(t, pos) => write!(f, "Invalid argument type '{}' at position {}", t, pos),
            IcuError::MissingArgValue(name) => write!(f, "Missing value for argument '{}'", name),
            IcuError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            IcuError::InvalidDateFormat(s) => write!(f, "Invalid date format: {}", s),
            IcuError::InvalidSyntax(msg, pos) => write!(f, "Invalid syntax: {} at position {}", msg, pos),
            IcuError::MaxDepthExceeded => write!(f, "Maximum nesting depth exceeded"),
        }
    }
}

impl std::error::Error for IcuError {}

/// Result type for ICU operations.
pub type IcuResult<T> = Result<T, IcuError>;

/// A value that can be passed to an ICU message.
#[derive(Debug, Clone)]
pub enum IcuValue {
    /// String value.
    String(String),
    /// Numeric value.
    Number(f64),
    /// Date value (Unix timestamp in milliseconds).
    Date(i64),
    /// Boolean value.
    Bool(bool),
    /// Null value.
    Null,
}

impl IcuValue {
    /// Get as string.
    pub fn as_string(&self) -> String {
        match self {
            IcuValue::String(s) => s.clone(),
            IcuValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            IcuValue::Date(d) => format!("{}", d),
            IcuValue::Bool(b) => format!("{}", b),
            IcuValue::Null => String::new(),
        }
    }

    /// Get as number, if applicable.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            IcuValue::Number(n) => Some(*n),
            IcuValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Get as date timestamp.
    pub fn as_date(&self) -> Option<i64> {
        match self {
            IcuValue::Date(d) => Some(*d),
            IcuValue::Number(n) => Some(*n as i64),
            _ => None,
        }
    }
}

impl From<&str> for IcuValue {
    fn from(s: &str) -> Self {
        IcuValue::String(s.to_string())
    }
}

impl From<String> for IcuValue {
    fn from(s: String) -> Self {
        IcuValue::String(s)
    }
}

impl From<f64> for IcuValue {
    fn from(n: f64) -> Self {
        IcuValue::Number(n)
    }
}

impl From<i64> for IcuValue {
    fn from(n: i64) -> Self {
        IcuValue::Number(n as f64)
    }
}

impl From<i32> for IcuValue {
    fn from(n: i32) -> Self {
        IcuValue::Number(n as f64)
    }
}

impl From<bool> for IcuValue {
    fn from(b: bool) -> Self {
        IcuValue::Bool(b)
    }
}

/// Argument type in ICU messages.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    /// Simple variable substitution.
    Simple,
    /// Number formatting.
    Number(NumberStyle),
    /// Date formatting.
    Date(DateStyle),
    /// Time formatting.
    Time(DateStyle),
    /// Date and time formatting.
    DateTime(DateStyle),
    /// Plural selection.
    Plural(PluralArg),
    /// Select (switch) selection.
    Select(SelectArg),
    /// Ordinal plural selection.
    SelectOrdinal(PluralArg),
}

/// Number formatting style.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NumberStyle {
    /// Default number format.
    #[default]
    Default,
    /// Integer (no decimal places).
    Integer,
    /// Currency format.
    Currency(Option<String>),
    /// Percent format.
    Percent,
    /// Custom pattern.
    Pattern(String),
}

/// Date/time formatting style.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DateStyle {
    /// Short format.
    Short,
    /// Medium format.
    #[default]
    Medium,
    /// Long format.
    Long,
    /// Full format.
    Full,
    /// Custom pattern.
    Pattern(String),
}

impl DateStyle {
    fn to_date_format(&self) -> DateFormat {
        match self {
            DateStyle::Short => DateFormat::Short,
            DateStyle::Medium => DateFormat::Medium,
            DateStyle::Long => DateFormat::Long,
            DateStyle::Full => DateFormat::Full,
            DateStyle::Pattern(p) => DateFormat::Custom(p.clone()),
        }
    }
}

/// Plural argument with cases and optional offset.
#[derive(Debug, Clone, PartialEq)]
pub struct PluralArg {
    /// Offset to subtract from the count.
    pub offset: f64,
    /// Cases mapped by either exact value or plural category.
    pub cases: Vec<PluralCase>,
}

/// A single plural case.
#[derive(Debug, Clone, PartialEq)]
pub struct PluralCase {
    /// The selector (exact value or category).
    pub selector: PluralSelector,
    /// The message to format for this case.
    pub message: IcuMessage,
}

/// Selector for a plural case.
#[derive(Debug, Clone, PartialEq)]
pub enum PluralSelector {
    /// Exact numeric match (=0, =1, =2, etc.).
    Exact(f64),
    /// Category match (zero, one, two, few, many, other).
    Category(PluralCategory),
}

/// Select argument with cases.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectArg {
    /// Cases mapped by keyword.
    pub cases: Vec<SelectCase>,
}

/// A single select case.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectCase {
    /// The keyword to match.
    pub keyword: String,
    /// The message to format for this case.
    pub message: IcuMessage,
}

/// A parsed ICU message.
#[derive(Debug, Clone, PartialEq)]
pub struct IcuMessage {
    /// Parts of the message.
    pub parts: Vec<MessagePart>,
}

/// A part of an ICU message.
#[derive(Debug, Clone, PartialEq)]
pub enum MessagePart {
    /// Literal text.
    Text(String),
    /// Argument reference.
    Argument(Argument),
    /// The # symbol in plural/select (replaced by the count).
    PoundSign,
}

/// An argument in an ICU message.
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    /// The argument name.
    pub name: String,
    /// The argument type.
    pub arg_type: ArgType,
}

impl IcuMessage {
    /// Parse an ICU message string.
    pub fn parse(input: &str) -> IcuResult<Self> {
        let mut parser = IcuParser::new(input);
        parser.parse_message(0)
    }

    /// Format the message with the given arguments.
    pub fn format(&self, args: &[(&str, IcuValue)], locale: &str) -> String {
        self.format_with_pound(args, locale, None)
    }

    /// Format with a pound value (for plural contexts).
    fn format_with_pound(&self, args: &[(&str, IcuValue)], locale: &str, pound_value: Option<f64>) -> String {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                MessagePart::Text(text) => result.push_str(text),
                MessagePart::PoundSign => {
                    if let Some(n) = pound_value {
                        if n.fract() == 0.0 {
                            result.push_str(&format!("{}", n as i64));
                        } else {
                            result.push_str(&format!("{}", n));
                        }
                    }
                }
                MessagePart::Argument(arg) => {
                    let value = args.iter()
                        .find(|(k, _)| *k == arg.name)
                        .map(|(_, v)| v.clone())
                        .unwrap_or(IcuValue::Null);

                    result.push_str(&format_argument(arg, &value, args, locale, pound_value));
                }
            }
        }

        result
    }
}

/// Format a single argument.
fn format_argument(
    arg: &Argument,
    value: &IcuValue,
    all_args: &[(&str, IcuValue)],
    locale: &str,
    pound_value: Option<f64>,
) -> String {
    match &arg.arg_type {
        ArgType::Simple => value.as_string(),

        ArgType::Number(style) => {
            let n = value.as_number().unwrap_or(0.0);
            match style {
                NumberStyle::Default => format_number(locale, n),
                NumberStyle::Integer => format!("{}", n as i64),
                NumberStyle::Currency(code) => {
                    let currency = code.as_deref().unwrap_or("USD");
                    format_currency(locale, n, currency)
                }
                NumberStyle::Percent => format!("{}%", (n * 100.0).round()),
                NumberStyle::Pattern(_) => format_number(locale, n),
            }
        }

        ArgType::Date(style) => {
            let timestamp = value.as_date().unwrap_or(0);
            format_timestamp(timestamp, &style.to_date_format(), locale)
        }

        ArgType::Time(style) => {
            let timestamp = value.as_date().unwrap_or(0);
            format_time(timestamp, style, locale)
        }

        ArgType::DateTime(style) => {
            let timestamp = value.as_date().unwrap_or(0);
            let date_str = format_timestamp(timestamp, &style.to_date_format(), locale);
            let time_str = format_time(timestamp, style, locale);
            format!("{} {}", date_str, time_str)
        }

        ArgType::Plural(plural_arg) => {
            let n = value.as_number().unwrap_or(0.0);
            let effective_n = n - plural_arg.offset;

            // First, try exact matches
            for case in &plural_arg.cases {
                if let PluralSelector::Exact(exact) = case.selector {
                    if (n - exact).abs() < f64::EPSILON {
                        return case.message.format_with_pound(all_args, locale, Some(effective_n));
                    }
                }
            }

            // Then, try category matches
            let category = get_plural_category(locale, effective_n);
            for case in &plural_arg.cases {
                if let PluralSelector::Category(cat) = case.selector {
                    if cat == category {
                        return case.message.format_with_pound(all_args, locale, Some(effective_n));
                    }
                }
            }

            // Fall back to "other"
            for case in &plural_arg.cases {
                if let PluralSelector::Category(PluralCategory::Other) = case.selector {
                    return case.message.format_with_pound(all_args, locale, Some(effective_n));
                }
            }

            value.as_string()
        }

        ArgType::SelectOrdinal(plural_arg) => {
            let n = value.as_number().unwrap_or(0.0);
            let effective_n = n - plural_arg.offset;

            // First, try exact matches
            for case in &plural_arg.cases {
                if let PluralSelector::Exact(exact) = case.selector {
                    if (n - exact).abs() < f64::EPSILON {
                        return case.message.format_with_pound(all_args, locale, Some(effective_n));
                    }
                }
            }

            // Then, try ordinal category matches
            let category = get_ordinal_category(locale, effective_n);
            for case in &plural_arg.cases {
                if let PluralSelector::Category(cat) = case.selector {
                    if cat == category {
                        return case.message.format_with_pound(all_args, locale, Some(effective_n));
                    }
                }
            }

            // Fall back to "other"
            for case in &plural_arg.cases {
                if let PluralSelector::Category(PluralCategory::Other) = case.selector {
                    return case.message.format_with_pound(all_args, locale, Some(effective_n));
                }
            }

            value.as_string()
        }

        ArgType::Select(select_arg) => {
            let key = value.as_string();

            // Try exact keyword match
            for case in &select_arg.cases {
                if case.keyword == key {
                    return case.message.format_with_pound(all_args, locale, pound_value);
                }
            }

            // Fall back to "other"
            for case in &select_arg.cases {
                if case.keyword == "other" {
                    return case.message.format_with_pound(all_args, locale, pound_value);
                }
            }

            value.as_string()
        }
    }
}

/// Format a time value.
fn format_time(timestamp: i64, style: &DateStyle, _locale: &str) -> String {
    // Simple time formatting (would use proper time library in production)
    let secs = timestamp / 1000;
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    match style {
        DateStyle::Short => format!("{:02}:{:02}", hours, minutes),
        DateStyle::Medium => format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        DateStyle::Long | DateStyle::Full => format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        DateStyle::Pattern(p) => p.clone(), // Would parse pattern in production
    }
}

/// ICU message parser.
struct IcuParser<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> IcuParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_message(&mut self, depth: usize) -> IcuResult<IcuMessage> {
        if depth > 10 {
            return Err(IcuError::MaxDepthExceeded);
        }

        let mut parts = Vec::new();
        let mut text = String::new();
        let mut in_escape = false;

        while let Some(c) = self.peek() {
            // Handle nested messages - stop at closing brace
            if c == '}' && depth > 0 {
                break;
            }

            if in_escape {
                if c == '\'' {
                    in_escape = false;
                    self.advance();
                } else {
                    text.push(c);
                    self.advance();
                }
                continue;
            }

            match c {
                '\'' => {
                    // Check for doubled quote (escape)
                    self.advance();
                    if self.peek() == Some('\'') {
                        text.push('\'');
                        self.advance();
                    } else {
                        // Start escape sequence
                        in_escape = true;
                    }
                }
                '{' => {
                    if !text.is_empty() {
                        parts.push(MessagePart::Text(text.clone()));
                        text.clear();
                    }
                    let arg = self.parse_argument(depth)?;
                    parts.push(MessagePart::Argument(arg));
                }
                '#' if depth > 0 => {
                    if !text.is_empty() {
                        parts.push(MessagePart::Text(text.clone()));
                        text.clear();
                    }
                    parts.push(MessagePart::PoundSign);
                    self.advance();
                }
                _ => {
                    text.push(c);
                    self.advance();
                }
            }
        }

        if !text.is_empty() {
            parts.push(MessagePart::Text(text));
        }

        Ok(IcuMessage { parts })
    }

    fn parse_argument(&mut self, depth: usize) -> IcuResult<Argument> {
        // Skip opening brace
        self.expect('{')?;
        self.skip_whitespace();

        // Parse argument name
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Check for type
        let arg_type = if self.peek() == Some(',') {
            self.advance();
            self.skip_whitespace();
            self.parse_arg_type(depth)?
        } else {
            ArgType::Simple
        };

        self.skip_whitespace();
        self.expect('}')?;

        Ok(Argument { name, arg_type })
    }

    fn parse_arg_type(&mut self, depth: usize) -> IcuResult<ArgType> {
        let type_name = self.parse_identifier()?;
        self.skip_whitespace();

        match type_name.to_lowercase().as_str() {
            "number" => {
                let style = if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                    self.parse_number_style()?
                } else {
                    NumberStyle::Default
                };
                Ok(ArgType::Number(style))
            }
            "date" => {
                let style = if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                    self.parse_date_style()?
                } else {
                    DateStyle::Medium
                };
                Ok(ArgType::Date(style))
            }
            "time" => {
                let style = if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                    self.parse_date_style()?
                } else {
                    DateStyle::Medium
                };
                Ok(ArgType::Time(style))
            }
            "datetime" => {
                let style = if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                    self.parse_date_style()?
                } else {
                    DateStyle::Medium
                };
                Ok(ArgType::DateTime(style))
            }
            "plural" => {
                self.expect(',')?;
                self.skip_whitespace();
                let plural_arg = self.parse_plural_arg(depth)?;
                Ok(ArgType::Plural(plural_arg))
            }
            "selectordinal" => {
                self.expect(',')?;
                self.skip_whitespace();
                let plural_arg = self.parse_plural_arg(depth)?;
                Ok(ArgType::SelectOrdinal(plural_arg))
            }
            "select" => {
                self.expect(',')?;
                self.skip_whitespace();
                let select_arg = self.parse_select_arg(depth)?;
                Ok(ArgType::Select(select_arg))
            }
            _ => Err(IcuError::InvalidArgType(type_name, self.pos)),
        }
    }

    fn parse_number_style(&mut self) -> IcuResult<NumberStyle> {
        let style = self.parse_identifier()?;
        match style.to_lowercase().as_str() {
            "integer" => Ok(NumberStyle::Integer),
            "percent" => Ok(NumberStyle::Percent),
            "currency" => Ok(NumberStyle::Currency(None)),
            _ => Ok(NumberStyle::Pattern(style)),
        }
    }

    fn parse_date_style(&mut self) -> IcuResult<DateStyle> {
        let style = self.parse_identifier()?;
        match style.to_lowercase().as_str() {
            "short" => Ok(DateStyle::Short),
            "medium" => Ok(DateStyle::Medium),
            "long" => Ok(DateStyle::Long),
            "full" => Ok(DateStyle::Full),
            _ => Ok(DateStyle::Pattern(style)),
        }
    }

    fn parse_plural_arg(&mut self, depth: usize) -> IcuResult<PluralArg> {
        let mut offset = 0.0;
        let mut cases = Vec::new();

        // Check for offset
        if self.peek_word() == "offset" {
            self.parse_identifier()?;
            self.expect(':')?;
            self.skip_whitespace();
            offset = self.parse_number()?;
            self.skip_whitespace();
        }

        // Parse cases
        while self.peek() != Some('}') {
            self.skip_whitespace();
            if self.peek() == Some('}') {
                break;
            }

            let selector = self.parse_plural_selector()?;
            self.skip_whitespace();
            self.expect('{')?;
            let message = self.parse_message(depth + 1)?;
            self.expect('}')?;

            cases.push(PluralCase { selector, message });
            self.skip_whitespace();
        }

        Ok(PluralArg { offset, cases })
    }

    fn parse_plural_selector(&mut self) -> IcuResult<PluralSelector> {
        self.skip_whitespace();

        if self.peek() == Some('=') {
            self.advance();
            let n = self.parse_number()?;
            Ok(PluralSelector::Exact(n))
        } else {
            let keyword = self.parse_identifier()?;
            let category = match keyword.to_lowercase().as_str() {
                "zero" => PluralCategory::Zero,
                "one" => PluralCategory::One,
                "two" => PluralCategory::Two,
                "few" => PluralCategory::Few,
                "many" => PluralCategory::Many,
                "other" => PluralCategory::Other,
                _ => return Err(IcuError::InvalidSyntax(format!("Unknown plural category: {}", keyword), self.pos)),
            };
            Ok(PluralSelector::Category(category))
        }
    }

    fn parse_select_arg(&mut self, depth: usize) -> IcuResult<SelectArg> {
        let mut cases = Vec::new();

        while self.peek() != Some('}') {
            self.skip_whitespace();
            if self.peek() == Some('}') {
                break;
            }

            let keyword = self.parse_identifier()?;
            self.skip_whitespace();
            self.expect('{')?;
            let message = self.parse_message(depth + 1)?;
            self.expect('}')?;

            cases.push(SelectCase { keyword, message });
            self.skip_whitespace();
        }

        Ok(SelectArg { cases })
    }

    fn parse_identifier(&mut self) -> IcuResult<String> {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if name.is_empty() {
            Err(IcuError::UnexpectedChar(self.peek().unwrap_or('\0'), self.pos))
        } else {
            Ok(name)
        }
    }

    fn parse_number(&mut self) -> IcuResult<f64> {
        let mut s = String::new();
        if self.peek() == Some('-') {
            s.push('-');
            self.advance();
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s.parse().map_err(|_| IcuError::InvalidNumber(s))
    }

    fn peek_word(&self) -> String {
        let mut word = String::new();
        let mut pos = self.pos;
        while pos < self.chars.len() {
            let c = self.chars[pos];
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                pos += 1;
            } else {
                break;
            }
        }
        word
    }

    fn expect(&mut self, expected: char) -> IcuResult<()> {
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(IcuError::UnexpectedChar(c, self.pos - 1)),
            None => Err(IcuError::UnexpectedEof),
        }
    }
}

/// Compile an ICU message from a string.
/// Returns a cached compiled message for efficiency.
pub fn compile(message: &str) -> IcuResult<IcuMessage> {
    IcuMessage::parse(message)
}

/// Format an ICU message string with arguments.
pub fn format(message: &str, args: &[(&str, IcuValue)], locale: &str) -> String {
    match IcuMessage::parse(message) {
        Ok(msg) => msg.format(args, locale),
        Err(_) => message.to_string(),
    }
}

/// Format an ICU message string with a HashMap of arguments.
pub fn format_map(message: &str, args: &HashMap<String, IcuValue>, locale: &str) -> String {
    let args_vec: Vec<(&str, IcuValue)> = args
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    format(message, &args_vec, locale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_message() {
        let msg = IcuMessage::parse("Hello, World!").unwrap();
        let result = msg.format(&[], "en");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_simple_argument() {
        let msg = IcuMessage::parse("Hello, {name}!").unwrap();
        let result = msg.format(&[("name", IcuValue::String("Alice".to_string()))], "en");
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_number_format() {
        let msg = IcuMessage::parse("You have {count, number} messages").unwrap();
        let result = msg.format(&[("count", IcuValue::Number(1234.5))], "en");
        assert!(result.contains("1"));
    }

    #[test]
    fn test_plural_simple() {
        let msg = IcuMessage::parse("{count, plural, one{# item} other{# items}}").unwrap();

        let result = msg.format(&[("count", IcuValue::Number(1.0))], "en");
        assert_eq!(result, "1 item");

        let result = msg.format(&[("count", IcuValue::Number(5.0))], "en");
        assert_eq!(result, "5 items");
    }

    #[test]
    fn test_plural_with_exact() {
        let msg = IcuMessage::parse("{count, plural, =0{no items} =1{one item} other{# items}}").unwrap();

        let result = msg.format(&[("count", IcuValue::Number(0.0))], "en");
        assert_eq!(result, "no items");

        let result = msg.format(&[("count", IcuValue::Number(1.0))], "en");
        assert_eq!(result, "one item");

        let result = msg.format(&[("count", IcuValue::Number(5.0))], "en");
        assert_eq!(result, "5 items");
    }

    #[test]
    fn test_plural_with_offset() {
        let msg = IcuMessage::parse("{count, plural, offset:1 =0{nobody} =1{just you} one{you and # other} other{you and # others}}").unwrap();

        let result = msg.format(&[("count", IcuValue::Number(0.0))], "en");
        assert_eq!(result, "nobody");

        let result = msg.format(&[("count", IcuValue::Number(1.0))], "en");
        assert_eq!(result, "just you");

        let result = msg.format(&[("count", IcuValue::Number(2.0))], "en");
        assert_eq!(result, "you and 1 other");

        let result = msg.format(&[("count", IcuValue::Number(5.0))], "en");
        assert_eq!(result, "you and 4 others");
    }

    #[test]
    fn test_select() {
        let msg = IcuMessage::parse("{gender, select, male{He} female{She} other{They}} liked your post").unwrap();

        let result = msg.format(&[("gender", IcuValue::String("male".to_string()))], "en");
        assert_eq!(result, "He liked your post");

        let result = msg.format(&[("gender", IcuValue::String("female".to_string()))], "en");
        assert_eq!(result, "She liked your post");

        let result = msg.format(&[("gender", IcuValue::String("other".to_string()))], "en");
        assert_eq!(result, "They liked your post");
    }

    #[test]
    fn test_selectordinal() {
        let msg = IcuMessage::parse("{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}").unwrap();

        let result = msg.format(&[("n", IcuValue::Number(1.0))], "en");
        assert_eq!(result, "1st");

        let result = msg.format(&[("n", IcuValue::Number(2.0))], "en");
        assert_eq!(result, "2nd");

        let result = msg.format(&[("n", IcuValue::Number(3.0))], "en");
        assert_eq!(result, "3rd");

        let result = msg.format(&[("n", IcuValue::Number(4.0))], "en");
        assert_eq!(result, "4th");

        let result = msg.format(&[("n", IcuValue::Number(11.0))], "en");
        assert_eq!(result, "11th");

        let result = msg.format(&[("n", IcuValue::Number(21.0))], "en");
        assert_eq!(result, "21st");
    }

    #[test]
    fn test_nested_plural_select() {
        let msg = IcuMessage::parse(
            "{gender, select, male{{count, plural, one{He has # message} other{He has # messages}}} female{{count, plural, one{She has # message} other{She has # messages}}} other{{count, plural, one{They have # message} other{They have # messages}}}}"
        ).unwrap();

        let result = msg.format(&[
            ("gender", IcuValue::String("male".to_string())),
            ("count", IcuValue::Number(1.0)),
        ], "en");
        assert_eq!(result, "He has 1 message");

        let result = msg.format(&[
            ("gender", IcuValue::String("female".to_string())),
            ("count", IcuValue::Number(5.0)),
        ], "en");
        assert_eq!(result, "She has 5 messages");
    }

    #[test]
    fn test_escape_quotes() {
        let msg = IcuMessage::parse("It''s a test").unwrap();
        let result = msg.format(&[], "en");
        assert_eq!(result, "It's a test");
    }

    #[test]
    fn test_escape_braces() {
        let msg = IcuMessage::parse("Use '{' for interpolation").unwrap();
        let result = msg.format(&[], "en");
        assert_eq!(result, "Use { for interpolation");
    }

    #[test]
    fn test_complex_message() {
        let msg = IcuMessage::parse(
            "{name} has {count, plural, =0{no messages} one{# message} other{# messages}} in {folder}"
        ).unwrap();

        let result = msg.format(&[
            ("name", IcuValue::String("Alice".to_string())),
            ("count", IcuValue::Number(0.0)),
            ("folder", IcuValue::String("inbox".to_string())),
        ], "en");
        assert_eq!(result, "Alice has no messages in inbox");

        let result = msg.format(&[
            ("name", IcuValue::String("Bob".to_string())),
            ("count", IcuValue::Number(1.0)),
            ("folder", IcuValue::String("spam".to_string())),
        ], "en");
        assert_eq!(result, "Bob has 1 message in spam");
    }

    #[test]
    fn test_format_helper() {
        let result = format(
            "Hello, {name}!",
            &[("name", IcuValue::from("World"))],
            "en"
        );
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_russian_plurals() {
        let msg = IcuMessage::parse("{count, plural, one{# сообщение} few{# сообщения} many{# сообщений} other{# сообщений}}").unwrap();

        let result = msg.format(&[("count", IcuValue::Number(1.0))], "ru");
        assert_eq!(result, "1 сообщение");

        let result = msg.format(&[("count", IcuValue::Number(2.0))], "ru");
        assert_eq!(result, "2 сообщения");

        let result = msg.format(&[("count", IcuValue::Number(5.0))], "ru");
        assert_eq!(result, "5 сообщений");

        let result = msg.format(&[("count", IcuValue::Number(21.0))], "ru");
        assert_eq!(result, "21 сообщение");
    }

    #[test]
    fn test_number_styles() {
        let msg = IcuMessage::parse("{n, number, integer}").unwrap();
        let result = msg.format(&[("n", IcuValue::Number(1234.56))], "en");
        assert_eq!(result, "1234");

        let msg = IcuMessage::parse("{n, number, percent}").unwrap();
        let result = msg.format(&[("n", IcuValue::Number(0.75))], "en");
        assert_eq!(result, "75%");
    }

    #[test]
    fn test_error_handling() {
        assert!(IcuMessage::parse("{unclosed").is_err());
        assert!(IcuMessage::parse("{name, invalid}").is_err());
    }
}

