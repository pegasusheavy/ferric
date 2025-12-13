//! Route parameters and query string handling.

use crate::reactive::{signal, computed, Signal, Computed};
use std::collections::HashMap;
use std::str::FromStr;

/// Reactive container for route parameters.
#[derive(Clone)]
pub struct Params {
    inner: Signal<HashMap<String, String>>,
}

impl Params {
    /// Create empty params.
    pub fn new() -> Self {
        Self {
            inner: signal(HashMap::new()),
        }
    }

    /// Create params from a HashMap.
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            inner: signal(map),
        }
    }

    /// Get a parameter value.
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.get().get(key).cloned()
    }

    /// Get a parameter as a specific type.
    pub fn get_as<T: FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Get a parameter as i32.
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.get_as(key)
    }

    /// Get a parameter as i64.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_as(key)
    }

    /// Get a parameter as f64.
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get_as(key)
    }

    /// Get a parameter as bool.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).map(|v| {
            matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
        })
    }

    /// Get all parameter keys.
    pub fn keys(&self) -> Vec<String> {
        self.inner.get().keys().cloned().collect()
    }

    /// Check if a parameter exists.
    pub fn has(&self, key: &str) -> bool {
        self.inner.get().contains_key(key)
    }

    /// Get all parameters as a HashMap.
    pub fn to_map(&self) -> HashMap<String, String> {
        self.inner.get()
    }

    /// Check if params is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.get().is_empty()
    }

    /// Get parameter count.
    pub fn len(&self) -> usize {
        self.inner.get().len()
    }

    /// Update a parameter (internal use).
    pub(crate) fn set(&self, key: &str, value: String) {
        self.inner.update(|params| {
            let mut new_params = params.clone();
            new_params.insert(key.to_string(), value);
            new_params
        });
    }

    /// Set all parameters (internal use).
    pub(crate) fn set_all(&self, map: HashMap<String, String>) {
        self.inner.set(map);
    }

    /// Create a computed signal for a specific parameter.
    pub fn observe(&self, key: &str) -> Computed<Option<String>> {
        let key = key.to_string();
        let inner = self.inner.clone();
        computed(move || inner.get().get(&key).cloned())
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Params")
            .field("params", &self.inner.get())
            .finish()
    }
}

/// Reactive container for query parameters.
#[derive(Clone)]
pub struct QueryParams {
    inner: Signal<HashMap<String, Vec<String>>>,
}

impl QueryParams {
    /// Create empty query params.
    pub fn new() -> Self {
        Self {
            inner: signal(HashMap::new()),
        }
    }

    /// Parse query params from a query string.
    pub fn from_query_string(query: &str) -> Self {
        let map = parse_query_string(query);
        Self {
            inner: signal(map),
        }
    }

    /// Get the first value for a key.
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.get().get(key).and_then(|v| v.first()).cloned()
    }

    /// Get all values for a key.
    pub fn get_all(&self, key: &str) -> Vec<String> {
        self.inner.get().get(key).cloned().unwrap_or_default()
    }

    /// Get a parameter as a specific type.
    pub fn get_as<T: FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Get a parameter as i32.
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.get_as(key)
    }

    /// Get a parameter as i64.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_as(key)
    }

    /// Get a parameter as f64.
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get_as(key)
    }

    /// Get a parameter as bool.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).map(|v| {
            matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
        })
    }

    /// Check if a parameter exists.
    pub fn has(&self, key: &str) -> bool {
        self.inner.get().contains_key(key)
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<String> {
        self.inner.get().keys().cloned().collect()
    }

    /// Get as single-value map (first value for each key).
    pub fn to_map(&self) -> HashMap<String, String> {
        self.inner
            .get()
            .iter()
            .filter_map(|(k, v)| v.first().map(|val| (k.clone(), val.clone())))
            .collect()
    }

    /// Convert to query string.
    pub fn to_string(&self) -> String {
        serialize_query_params(&self.inner.get())
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.inner.get().is_empty()
    }

    /// Update params (internal use).
    pub(crate) fn set_all(&self, map: HashMap<String, Vec<String>>) {
        self.inner.set(map);
    }

    /// Add a parameter value.
    pub(crate) fn add(&self, key: &str, value: String) {
        self.inner.update(|params| {
            let mut new_params = params.clone();
            new_params.entry(key.to_string()).or_default().push(value);
            new_params
        });
    }

    /// Create a computed signal for a specific parameter.
    pub fn observe(&self, key: &str) -> Computed<Option<String>> {
        let key = key.to_string();
        let inner = self.inner.clone();
        computed(move || {
            inner.get().get(&key).and_then(|v| v.first()).cloned()
        })
    }
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for QueryParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryParams")
            .field("params", &self.inner.get())
            .finish()
    }
}

/// Parse a query string into a multi-value map.
pub fn parse_query_string(query: &str) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    let query = query.trim_start_matches('?');
    if query.is_empty() {
        return result;
    }

    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let key = url_decode(key);
            let value = parts.next().map(url_decode).unwrap_or_default();
            result.entry(key).or_default().push(value);
        }
    }

    result
}

/// Serialize query params to a query string.
pub fn serialize_query_params(params: &HashMap<String, Vec<String>>) -> String {
    let pairs: Vec<String> = params
        .iter()
        .flat_map(|(key, values)| {
            values.iter().map(move |value| {
                format!("{}={}", url_encode(key), url_encode(value))
            })
        })
        .collect();

    if pairs.is_empty() {
        String::new()
    } else {
        format!("?{}", pairs.join("&"))
    }
}

/// URL decode a string.
pub fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

/// URL encode a string.
pub fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);

    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            ' ' => result.push('+'),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }

    result
}

/// Extract path parameters from a URL given a route pattern.
pub fn extract_params(pattern: &str, url: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let url_parts: Vec<&str> = url
        .split('?')
        .next()
        .unwrap_or(url)
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let has_wildcard = pattern_parts.iter().any(|p| p.starts_with("**"));
    if !has_wildcard && pattern_parts.len() != url_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (i, pattern_part) in pattern_parts.iter().enumerate() {
        if pattern_part.starts_with(':') {
            let param_name = &pattern_part[1..];
            if let Some(value) = url_parts.get(i) {
                params.insert(param_name.to_string(), url_decode(value));
            } else {
                return None;
            }
        } else if pattern_part.starts_with("**") {
            let param_name = if pattern_part.len() > 2 {
                &pattern_part[2..]
            } else {
                "wildcard"
            };
            let rest: Vec<&str> = url_parts[i..].to_vec();
            params.insert(param_name.to_string(), rest.join("/"));
            break;
        } else if url_parts.get(i) != Some(pattern_part) {
            return None;
        }
    }

    Some(params)
}

/// Parse a full URL into components.
#[derive(Debug, Clone, Default)]
pub struct ParsedUrl {
    /// Path without query string.
    pub path: String,
    /// Query parameters.
    pub query_params: HashMap<String, Vec<String>>,
    /// Fragment (after #).
    pub fragment: Option<String>,
}

impl ParsedUrl {
    /// Parse a URL string.
    pub fn parse(url: &str) -> Self {
        let mut path = url.to_string();
        let mut fragment = None;
        let mut query_params = HashMap::new();

        if let Some(hash_pos) = path.find('#') {
            fragment = Some(path[hash_pos + 1..].to_string());
            path = path[..hash_pos].to_string();
        }

        if let Some(query_pos) = path.find('?') {
            let query = &path[query_pos + 1..];
            query_params = parse_query_string(query);
            path = path[..query_pos].to_string();
        }

        Self {
            path,
            query_params,
            fragment,
        }
    }

    /// Reconstruct the URL.
    pub fn to_string(&self) -> String {
        let mut url = self.path.clone();

        if !self.query_params.is_empty() {
            url.push_str(&serialize_query_params(&self.query_params));
        }

        if let Some(ref frag) = self.fragment {
            url.push('#');
            url.push_str(frag);
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_basic() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "42".to_string());
        map.insert("name".to_string(), "test".to_string());

        let params = Params::from_map(map);

        assert_eq!(params.get("id"), Some("42".to_string()));
        assert_eq!(params.get_i32("id"), Some(42));
        assert_eq!(params.get("name"), Some("test".to_string()));
        assert!(params.has("id"));
        assert!(!params.has("missing"));
    }

    #[test]
    fn test_query_params() {
        let query = QueryParams::from_query_string("?name=John&age=30&tags=a&tags=b");

        assert_eq!(query.get("name"), Some("John".to_string()));
        assert_eq!(query.get_i32("age"), Some(30));
        assert_eq!(query.get_all("tags"), vec!["a", "b"]);
    }

    #[test]
    fn test_parse_query_string() {
        let result = parse_query_string("foo=bar&baz=qux");
        assert_eq!(result.get("foo").and_then(|v| v.first()), Some(&"bar".to_string()));
        assert_eq!(result.get("baz").and_then(|v| v.first()), Some(&"qux".to_string()));
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("hello+world"), "hello world");
        assert_eq!(url_decode("%3A%2F%2F"), "://");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("a=b&c=d"), "a%3Db%26c%3Dd");
    }

    #[test]
    fn test_extract_params() {
        let params = extract_params("/users/:userId/posts/:postId", "/users/123/posts/456");
        assert!(params.is_some());
        let params = params.unwrap();
        assert_eq!(params.get("userId"), Some(&"123".to_string()));
        assert_eq!(params.get("postId"), Some(&"456".to_string()));
    }

    #[test]
    fn test_extract_params_wildcard() {
        let params = extract_params("/files/**path", "/files/a/b/c.txt");
        assert!(params.is_some());
        let params = params.unwrap();
        assert_eq!(params.get("path"), Some(&"a/b/c.txt".to_string()));
    }

    #[test]
    fn test_parsed_url() {
        let parsed = ParsedUrl::parse("/users/123?sort=name&order=asc#section");

        assert_eq!(parsed.path, "/users/123");
        assert_eq!(parsed.fragment, Some("section".to_string()));
        assert!(parsed.query_params.contains_key("sort"));
    }
}

