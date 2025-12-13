//! HTTP method types

use std::fmt;

/// HTTP request methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// GET method - retrieve a resource
    Get,
    /// POST method - create a resource
    Post,
    /// PUT method - replace a resource
    Put,
    /// DELETE method - remove a resource
    Delete,
    /// PATCH method - partially update a resource
    Patch,
    /// HEAD method - retrieve headers only
    Head,
    /// OPTIONS method - get supported methods
    Options,
    /// CONNECT method - establish a tunnel
    Connect,
    /// TRACE method - diagnostic trace
    Trace,
}

impl Method {
    /// Returns the method as a string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
        }
    }

    /// Check if the method typically has a request body
    pub fn has_body(&self) -> bool {
        matches!(self, Method::Post | Method::Put | Method::Patch)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::Get
    }
}

impl From<Method> for http::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => http::Method::GET,
            Method::Post => http::Method::POST,
            Method::Put => http::Method::PUT,
            Method::Delete => http::Method::DELETE,
            Method::Patch => http::Method::PATCH,
            Method::Head => http::Method::HEAD,
            Method::Options => http::Method::OPTIONS,
            Method::Connect => http::Method::CONNECT,
            Method::Trace => http::Method::TRACE,
        }
    }
}

impl TryFrom<&str> for Method {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            "CONNECT" => Ok(Method::Connect),
            "TRACE" => Ok(Method::Trace),
            _ => Err(crate::Error::Request(format!("Unknown HTTP method: {}", s))),
        }
    }
}

