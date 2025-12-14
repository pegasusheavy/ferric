//! Index operations.

use crate::cursor::{Cursor, CursorDirection};
use crate::error::{IdbError, IdbResult};
use crate::key_range::KeyRange;
use crate::request::request_to_future;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;

/// An IndexedDB index.
pub struct Index {
    inner: web_sys::IdbIndex,
}

impl Index {
    pub(crate) fn new(inner: web_sys::IdbIndex) -> Self {
        Self { inner }
    }

    /// Get the index name.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get the key path.
    pub fn key_path(&self) -> Option<String> {
        self.inner.key_path().ok()?.as_string()
    }

    /// Check if the index is unique.
    pub fn unique(&self) -> bool {
        self.inner.unique()
    }

    /// Check if the index has multi-entry.
    pub fn multi_entry(&self) -> bool {
        self.inner.multi_entry()
    }

    /// Get a record by index key.
    pub async fn get<T: DeserializeOwned, K: Into<JsValue>>(&self, key: K) -> IdbResult<Option<T>> {
        let request = self
            .inner
            .get(&key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        let result = request_to_future(&request).await?;

        if result.is_undefined() || result.is_null() {
            return Ok(None);
        }

        let value: T = serde_wasm_bindgen::from_value(result)
            .map_err(|e| IdbError::DeserializationError(e.to_string()))?;
        Ok(Some(value))
    }

    /// Get the primary key for an index key.
    pub async fn get_key<K: Into<JsValue>>(&self, key: K) -> IdbResult<Option<JsValue>> {
        let request = self
            .inner
            .get_key(&key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        let result = request_to_future(&request).await?;

        if result.is_undefined() || result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// Get all records matching an index key.
    pub async fn get_all<T: DeserializeOwned, K: Into<JsValue>>(
        &self,
        key: K,
    ) -> IdbResult<Vec<T>> {
        let request = self
            .inner
            .get_all_with_key(&key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        let result = request_to_future(&request).await?;
        let array = js_sys::Array::from(&result);

        let mut values = Vec::with_capacity(array.length() as usize);
        for i in 0..array.length() {
            let value: T = serde_wasm_bindgen::from_value(array.get(i))
                .map_err(|e| IdbError::DeserializationError(e.to_string()))?;
            values.push(value);
        }

        Ok(values)
    }

    /// Get all records matching a key range.
    pub async fn get_all_in_range<T: DeserializeOwned>(
        &self,
        range: &KeyRange,
    ) -> IdbResult<Vec<T>> {
        let request = self
            .inner
            .get_all_with_key(range.raw())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        let result = request_to_future(&request).await?;
        let array = js_sys::Array::from(&result);

        let mut values = Vec::with_capacity(array.length() as usize);
        for i in 0..array.length() {
            let value: T = serde_wasm_bindgen::from_value(array.get(i))
                .map_err(|e| IdbError::DeserializationError(e.to_string()))?;
            values.push(value);
        }

        Ok(values)
    }

    /// Count records matching an index key.
    pub async fn count<K: Into<JsValue>>(&self, key: K) -> IdbResult<u32> {
        let request = self
            .inner
            .count_with_key(&key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        let result = request_to_future(&request).await?;
        Ok(result.as_f64().unwrap_or(0.0) as u32)
    }

    /// Count all records in the index.
    pub async fn count_all(&self) -> IdbResult<u32> {
        let request = self
            .inner
            .count()
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        let result = request_to_future(&request).await?;
        Ok(result.as_f64().unwrap_or(0.0) as u32)
    }

    /// Open a cursor on the index.
    pub async fn open_cursor(&self) -> IdbResult<Cursor> {
        let request = self
            .inner
            .open_cursor()
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        Cursor::from_request(request).await
    }

    /// Open a cursor with a key range.
    pub async fn open_cursor_range(&self, range: &KeyRange) -> IdbResult<Cursor> {
        let request = self
            .inner
            .open_cursor_with_range(range.raw())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        Cursor::from_request(request).await
    }

    /// Open a cursor with direction.
    pub async fn open_cursor_with_direction(&self, direction: CursorDirection) -> IdbResult<Cursor> {
        let request = self
            .inner
            .open_cursor_with_range_and_direction(&JsValue::UNDEFINED, direction.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        Cursor::from_request(request).await
    }

    /// Get the underlying web-sys index.
    pub fn raw(&self) -> &web_sys::IdbIndex {
        &self.inner
    }
}

/// Builder for creating indexes during database upgrades.
///
/// **Note:** This builder is currently not used in the implementation. Indexes are
/// created directly through [`ObjectStoreBuilder`](crate::ObjectStoreBuilder) methods
/// like `.index()`, `.unique_index()`, and `.multi_entry_index()`.
///
/// This struct is retained in the public API for potential future use cases where
/// more fine-grained control over index creation might be needed.
///
/// # Recommended Usage
///
/// Use the builder methods on `ObjectStoreBuilder` instead:
///
/// ```ignore
/// use ferric_idb::prelude::*;
///
/// let store = ObjectStoreBuilder::new(db, "users")
///     .key_path("id")
///     .index("email", "email")
///     .unique_index("username", "username")
///     .build()?;
/// ```
#[allow(dead_code)] // Reserved for future use; indexes created via ObjectStoreBuilder
pub struct IndexBuilder<'a> {
    store: &'a web_sys::IdbObjectStore,
    name: String,
    key_path: String,
    unique: bool,
    multi_entry: bool,
}

#[allow(dead_code)] // Reserved for future use
impl<'a> IndexBuilder<'a> {
    /// Create a new index builder.
    ///
    /// This is currently not used. Use `ObjectStoreBuilder` methods instead.
    pub(crate) fn new(store: &'a web_sys::IdbObjectStore, name: &str, key_path: &str) -> Self {
        Self {
            store,
            name: name.to_string(),
            key_path: key_path.to_string(),
            unique: false,
            multi_entry: false,
        }
    }

    /// Set the index as unique.
    pub fn unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    /// Set multi-entry mode.
    pub fn multi_entry(mut self, multi_entry: bool) -> Self {
        self.multi_entry = multi_entry;
        self
    }

    /// Build and create the index.
    pub fn build(self) -> IdbResult<Index> {
        let params = web_sys::IdbIndexParameters::new();
        params.set_unique(self.unique);
        params.set_multi_entry(self.multi_entry);

        let index = self
            .store
            .create_index_with_str_and_optional_parameters(&self.name, &self.key_path, &params)
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        Ok(Index::new(index))
    }
}

