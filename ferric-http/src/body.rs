//! Request/Response body types

/// Request body types
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum Body {
    /// Empty body
    #[default]
    Empty,
    /// Raw bytes
    Bytes(Vec<u8>),
    /// Text body
    Text(String),
    /// JSON body (serialized as string)
    #[cfg(feature = "json")]
    Json(String),
    /// Form data (URL encoded)
    Form(Vec<(String, String)>),
}

impl Body {
    /// Create an empty body
    pub fn empty() -> Self {
        Body::Empty
    }

    /// Create a body from bytes
    pub fn bytes(data: impl Into<Vec<u8>>) -> Self {
        Body::Bytes(data.into())
    }

    /// Create a body from text
    pub fn text(data: impl Into<String>) -> Self {
        Body::Text(data.into())
    }

    /// Create a JSON body from a serializable value
    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(value: &T) -> crate::Result<Self> {
        let json = serde_json::to_string(value)?;
        Ok(Body::Json(json))
    }

    /// Create a form-encoded body
    pub fn form(fields: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        Body::Form(
            fields
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }

    /// Get the content type for this body
    pub fn content_type(&self) -> Option<&'static str> {
        match self {
            Body::Empty => None,
            Body::Bytes(_) => Some("application/octet-stream"),
            Body::Text(_) => Some("text/plain; charset=utf-8"),
            #[cfg(feature = "json")]
            Body::Json(_) => Some("application/json"),
            Body::Form(_) => Some("application/x-www-form-urlencoded"),
        }
    }

    /// Convert body to bytes
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Body::Empty => Vec::new(),
            Body::Bytes(b) => b,
            Body::Text(s) => s.into_bytes(),
            #[cfg(feature = "json")]
            Body::Json(s) => s.into_bytes(),
            Body::Form(fields) => {
                url_encode_form(&fields).into_bytes()
            }
        }
    }

    /// Check if the body is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Body::Empty)
    }
}


impl From<Vec<u8>> for Body {
    fn from(data: Vec<u8>) -> Self {
        Body::Bytes(data)
    }
}

impl From<&[u8]> for Body {
    fn from(data: &[u8]) -> Self {
        Body::Bytes(data.to_vec())
    }
}

impl From<String> for Body {
    fn from(data: String) -> Self {
        Body::Text(data)
    }
}

impl From<&str> for Body {
    fn from(data: &str) -> Self {
        Body::Text(data.to_string())
    }
}

/// URL encode form data
fn url_encode_form(fields: &[(String, String)]) -> String {
    fields
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Simple URL encoding
fn url_encode(s: &str) -> String {
    let mut result = String::new();
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

