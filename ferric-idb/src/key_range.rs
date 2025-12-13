//! Key range utilities.

use crate::error::{IdbError, IdbResult};
use wasm_bindgen::JsValue;

/// A key range for querying IndexedDB.
pub struct KeyRange {
    inner: web_sys::IdbKeyRange,
}

impl KeyRange {
    /// Create a range that matches only a single key.
    pub fn only<K: Into<JsValue>>(key: K) -> IdbResult<Self> {
        let inner = web_sys::IdbKeyRange::only(&key.into())
            .map_err(|e| IdbError::KeyError(format!("{:?}", e)))?;
        Ok(Self { inner })
    }

    /// Create a range with only a lower bound.
    pub fn lower_bound<K: Into<JsValue>>(lower: K, open: bool) -> IdbResult<Self> {
        let inner = web_sys::IdbKeyRange::lower_bound_with_open(&lower.into(), open)
            .map_err(|e| IdbError::KeyError(format!("{:?}", e)))?;
        Ok(Self { inner })
    }

    /// Create a range with only an upper bound.
    pub fn upper_bound<K: Into<JsValue>>(upper: K, open: bool) -> IdbResult<Self> {
        let inner = web_sys::IdbKeyRange::upper_bound_with_open(&upper.into(), open)
            .map_err(|e| IdbError::KeyError(format!("{:?}", e)))?;
        Ok(Self { inner })
    }

    /// Create a range with both lower and upper bounds.
    pub fn bound<K1: Into<JsValue>, K2: Into<JsValue>>(
        lower: K1,
        upper: K2,
        lower_open: bool,
        upper_open: bool,
    ) -> IdbResult<Self> {
        let inner = web_sys::IdbKeyRange::bound_with_lower_open_and_upper_open(
            &lower.into(),
            &upper.into(),
            lower_open,
            upper_open,
        )
        .map_err(|e| IdbError::KeyError(format!("{:?}", e)))?;
        Ok(Self { inner })
    }

    /// Create an inclusive range [lower, upper].
    pub fn inclusive<K1: Into<JsValue>, K2: Into<JsValue>>(lower: K1, upper: K2) -> IdbResult<Self> {
        Self::bound(lower, upper, false, false)
    }

    /// Create an exclusive range (lower, upper).
    pub fn exclusive<K1: Into<JsValue>, K2: Into<JsValue>>(lower: K1, upper: K2) -> IdbResult<Self> {
        Self::bound(lower, upper, true, true)
    }

    /// Create a range >= lower.
    pub fn gte<K: Into<JsValue>>(lower: K) -> IdbResult<Self> {
        Self::lower_bound(lower, false)
    }

    /// Create a range > lower.
    pub fn gt<K: Into<JsValue>>(lower: K) -> IdbResult<Self> {
        Self::lower_bound(lower, true)
    }

    /// Create a range <= upper.
    pub fn lte<K: Into<JsValue>>(upper: K) -> IdbResult<Self> {
        Self::upper_bound(upper, false)
    }

    /// Create a range < upper.
    pub fn lt<K: Into<JsValue>>(upper: K) -> IdbResult<Self> {
        Self::upper_bound(upper, true)
    }

    /// Get the lower bound.
    pub fn lower(&self) -> Option<JsValue> {
        self.inner.lower().ok()
    }

    /// Get the upper bound.
    pub fn upper(&self) -> Option<JsValue> {
        self.inner.upper().ok()
    }

    /// Check if the lower bound is open.
    pub fn lower_open(&self) -> bool {
        self.inner.lower_open()
    }

    /// Check if the upper bound is open.
    pub fn upper_open(&self) -> bool {
        self.inner.upper_open()
    }

    /// Check if a key is within this range.
    pub fn includes<K: Into<JsValue>>(&self, key: K) -> IdbResult<bool> {
        self.inner
            .includes(&key.into())
            .map_err(|e| IdbError::KeyError(format!("{:?}", e)))
    }

    /// Get the underlying web-sys key range.
    pub fn raw(&self) -> &web_sys::IdbKeyRange {
        &self.inner
    }
}

/// Builder for creating key ranges with a fluent API.
pub struct KeyRangeBuilder {
    lower: Option<JsValue>,
    upper: Option<JsValue>,
    lower_open: bool,
    upper_open: bool,
}

impl KeyRangeBuilder {
    /// Create a new key range builder.
    pub fn new() -> Self {
        Self {
            lower: None,
            upper: None,
            lower_open: false,
            upper_open: false,
        }
    }

    /// Set the lower bound (inclusive).
    pub fn from<K: Into<JsValue>>(mut self, key: K) -> Self {
        self.lower = Some(key.into());
        self.lower_open = false;
        self
    }

    /// Set the lower bound (exclusive).
    pub fn after<K: Into<JsValue>>(mut self, key: K) -> Self {
        self.lower = Some(key.into());
        self.lower_open = true;
        self
    }

    /// Set the upper bound (inclusive).
    pub fn to<K: Into<JsValue>>(mut self, key: K) -> Self {
        self.upper = Some(key.into());
        self.upper_open = false;
        self
    }

    /// Set the upper bound (exclusive).
    pub fn before<K: Into<JsValue>>(mut self, key: K) -> Self {
        self.upper = Some(key.into());
        self.upper_open = true;
        self
    }

    /// Build the key range.
    pub fn build(self) -> IdbResult<KeyRange> {
        match (self.lower, self.upper) {
            (Some(lower), Some(upper)) => {
                KeyRange::bound(lower, upper, self.lower_open, self.upper_open)
            }
            (Some(lower), None) => KeyRange::lower_bound(lower, self.lower_open),
            (None, Some(upper)) => KeyRange::upper_bound(upper, self.upper_open),
            (None, None) => Err(IdbError::KeyError("No bounds specified".to_string())),
        }
    }
}

impl Default for KeyRangeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

