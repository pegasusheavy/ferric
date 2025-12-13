//! HTTP headers abstraction

use std::collections::HashMap;

/// HTTP headers collection
#[derive(Debug, Clone, Default)]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    /// Create a new empty headers collection
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert a header (case-insensitive key)
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.inner.insert(key.into().to_lowercase(), value.into());
        self
    }

    /// Get a header value by key (case-insensitive)
    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(&key.to_lowercase())
    }

    /// Remove a header by key (case-insensitive)
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.inner.remove(&key.to_lowercase())
    }

    /// Check if a header exists (case-insensitive)
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(&key.to_lowercase())
    }

    /// Get the number of headers
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if headers are empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate over headers
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }

    /// Set Content-Type header
    pub fn content_type(&mut self, value: impl Into<String>) -> &mut Self {
        self.insert("content-type", value)
    }

    /// Set Accept header
    pub fn accept(&mut self, value: impl Into<String>) -> &mut Self {
        self.insert("accept", value)
    }

    /// Set Authorization header
    pub fn authorization(&mut self, value: impl Into<String>) -> &mut Self {
        self.insert("authorization", value)
    }

    /// Set Bearer token authorization
    pub fn bearer_auth(&mut self, token: impl Into<String>) -> &mut Self {
        self.insert("authorization", format!("Bearer {}", token.into()))
    }

    /// Set Basic authentication
    pub fn basic_auth(&mut self, username: &str, password: Option<&str>) -> &mut Self {
        use std::io::Write;
        let mut credentials = Vec::new();
        write!(credentials, "{}:", username).unwrap();
        if let Some(pwd) = password {
            write!(credentials, "{}", pwd).unwrap();
        }
        // Simple base64 encoding without external dependency
        let encoded = base64_encode(&credentials);
        self.insert("authorization", format!("Basic {}", encoded))
    }

    /// Set User-Agent header
    pub fn user_agent(&mut self, value: impl Into<String>) -> &mut Self {
        self.insert("user-agent", value)
    }
}

impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(map: HashMap<String, String>) -> Self {
        let mut headers = Headers::new();
        for (k, v) in map {
            headers.insert(k, v);
        }
        headers
    }
}

impl<const N: usize> From<[(&str, &str); N]> for Headers {
    fn from(arr: [(&str, &str); N]) -> Self {
        let mut headers = Headers::new();
        for (k, v) in arr {
            headers.insert(k, v);
        }
        headers
    }
}

// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

