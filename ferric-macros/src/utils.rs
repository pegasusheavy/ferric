//! Utility functions for macro implementations.

use proc_macro2::Span;
use syn::{Ident, LitStr, Type};

/// Convert a string to PascalCase.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to snake_case.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to camelCase.
pub fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// Create an identifier from a string.
pub fn ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

/// Create a literal string.
pub fn lit_str(s: &str) -> LitStr {
    LitStr::new(s, Span::call_site())
}

/// Extract the inner type from Option<T>.
pub fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Some(inner);
                    }
                }
            }
        }
    }
    None
}

/// Extract the inner type from Vec<T>.
pub fn extract_vec_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Some(inner);
                    }
                }
            }
        }
    }
    None
}

/// Generate a unique identifier based on a prefix and index.
pub fn unique_ident(prefix: &str, index: usize) -> Ident {
    Ident::new(&format!("{}_{}", prefix, index), Span::call_site())
}

/// Parse a binding string like "class.active" or "style.color".
pub fn parse_binding(s: &str) -> (BindingKind, String) {
    if let Some(rest) = s.strip_prefix("class.") {
        (BindingKind::Class, rest.to_string())
    } else if let Some(rest) = s.strip_prefix("style.") {
        (BindingKind::Style, rest.to_string())
    } else if let Some(rest) = s.strip_prefix("attr.") {
        (BindingKind::Attribute, rest.to_string())
    } else {
        (BindingKind::Property, s.to_string())
    }
}

/// Kind of host binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Property,
    Attribute,
    Class,
    Style,
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

