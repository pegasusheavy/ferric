//! Utility functions for macro implementations.

use proc_macro2::Span;
use syn::LitStr;

/// Create a literal string.
pub fn lit_str(s: &str) -> LitStr {
    LitStr::new(s, Span::call_site())
}

/// Parse an event string like "click" or "keydown.enter".
pub fn parse_event(s: &str) -> (String, Option<String>) {
    if let Some(dot_pos) = s.find('.') {
        let event = s[..dot_pos].to_string();
        let modifier = s[dot_pos + 1..].to_string();
        (event, Some(modifier))
    } else {
        (s.to_string(), None)
    }
}
