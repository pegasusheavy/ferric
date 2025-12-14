//! Object store operations.

use crate::cursor::{Cursor, CursorDirection};
use crate::error::{IdbError, IdbResult};
use crate::index::Index;
use crate::key_range::KeyRange;
use crate::request::request_to_future;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::JsValue;
use web_sys::IdbObjectStoreParameters;

/// An IndexedDB object store.
pub struct ObjectStore {
    inner: web_sys::IdbObjectStore,
}

impl ObjectStore {
    pub(crate) fn new(inner: web_sys::IdbObjectStore) -> Self {
        Self { inner }
    }

    /// Get the store name.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get the key path.
    pub fn key_path(&self) -> Option<String> {
        self.inner.key_path().ok()?.as_string()
    }

    /// Check if the store uses auto-increment keys.
    pub fn auto_increment(&self) -> bool {
        self.inner.auto_increment()
    }

    /// Get the list of index names.
    pub fn index_names(&self) -> Vec<String> {
        let list = self.inner.index_names();
        let mut names = Vec::with_capacity(list.length() as usize);
        for i in 0..list.length() {
            if let Some(name) = list.get(i) {
                names.push(name);
            }
        }
        names
    }

    // =========================================================================
    // CRUD Operations
    // =========================================================================

    /// Add a new record. Fails if key already exists.
    pub async fn add<T: Serialize>(&self, value: &T) -> IdbResult<JsValue> {
        let js_value = serde_wasm_bindgen::to_value(value)?;
        let request = self
            .inner
            .add(&js_value)
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await
    }

    /// Add a new record with a specific key. Fails if key already exists.
    pub async fn add_with_key<T: Serialize, K: Into<JsValue>>(
        &self,
        value: &T,
        key: K,
    ) -> IdbResult<JsValue> {
        let js_value = serde_wasm_bindgen::to_value(value)?;
        let request = self
            .inner
            .add_with_key(&js_value, &key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await
    }

    /// Put a record (insert or update).
    pub async fn put<T: Serialize>(&self, value: &T) -> IdbResult<JsValue> {
        let js_value = serde_wasm_bindgen::to_value(value)?;
        let request = self
            .inner
            .put(&js_value)
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await
    }

    /// Put a record with a specific key (insert or update).
    pub async fn put_with_key<T: Serialize, K: Into<JsValue>>(
        &self,
        value: &T,
        key: K,
    ) -> IdbResult<JsValue> {
        let js_value = serde_wasm_bindgen::to_value(value)?;
        let request = self
            .inner
            .put_with_key(&js_value, &key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await
    }

    /// Get a record by key.
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

    /// Get a record by key, returning the raw JsValue.
    pub async fn get_raw<K: Into<JsValue>>(&self, key: K) -> IdbResult<JsValue> {
        let request = self
            .inner
            .get(&key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await
    }

    /// Delete a record by key.
    pub async fn delete<K: Into<JsValue>>(&self, key: K) -> IdbResult<()> {
        let request = self
            .inner
            .delete(&key.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await?;
        Ok(())
    }

    /// Clear all records in the store.
    pub async fn clear(&self) -> IdbResult<()> {
        let request = self
            .inner
            .clear()
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        request_to_future(&request).await?;
        Ok(())
    }

    /// Count all records in the store.
    pub async fn count(&self) -> IdbResult<u32> {
        let request = self
            .inner
            .count()
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        let result = request_to_future(&request).await?;
        Ok(result.as_f64().unwrap_or(0.0) as u32)
    }

    /// Count records matching a key range.
    pub async fn count_range(&self, range: &KeyRange) -> IdbResult<u32> {
        let request = self
            .inner
            .count_with_key(range.raw())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        let result = request_to_future(&request).await?;
        Ok(result.as_f64().unwrap_or(0.0) as u32)
    }

    /// Get all records.
    pub async fn get_all<T: DeserializeOwned>(&self) -> IdbResult<Vec<T>> {
        let request = self
            .inner
            .get_all()
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

    /// Get all records with a limit.
    pub async fn get_all_with_limit<T: DeserializeOwned>(&self, limit: u32) -> IdbResult<Vec<T>> {
        let request = self
            .inner
            .get_all_with_key_and_limit(&JsValue::UNDEFINED, limit)
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

    /// Get all keys.
    pub async fn get_all_keys(&self) -> IdbResult<Vec<JsValue>> {
        let request = self
            .inner
            .get_all_keys()
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        let result = request_to_future(&request).await?;
        let array = js_sys::Array::from(&result);

        let mut keys = Vec::with_capacity(array.length() as usize);
        for i in 0..array.length() {
            keys.push(array.get(i));
        }

        Ok(keys)
    }

    // =========================================================================
    // Cursor Operations
    // =========================================================================

    /// Open a cursor on the store.
    pub async fn open_cursor(&self) -> IdbResult<Cursor> {
        let request = self
            .inner
            .open_cursor()
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        Cursor::from_request(request).await
    }

    /// Open a cursor with a specific direction.
    pub async fn open_cursor_with_direction(&self, direction: CursorDirection) -> IdbResult<Cursor> {
        let request = self
            .inner
            .open_cursor_with_range_and_direction(&JsValue::UNDEFINED, direction.into())
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

    /// Open a cursor with a key range and direction.
    pub async fn open_cursor_range_with_direction(
        &self,
        range: &KeyRange,
        direction: CursorDirection,
    ) -> IdbResult<Cursor> {
        let request = self
            .inner
            .open_cursor_with_range_and_direction(range.raw(), direction.into())
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        Cursor::from_request(request).await
    }

    // =========================================================================
    // Index Operations
    // =========================================================================

    /// Get an index.
    pub fn index(&self, name: &str) -> IdbResult<Index> {
        let index = self
            .inner
            .index(name)
            .map_err(|_| IdbError::IndexNotFound(name.to_string()))?;
        Ok(Index::new(index))
    }

    /// Check if an index exists.
    pub fn has_index(&self, name: &str) -> bool {
        self.inner.index_names().contains(name)
    }

    /// Get the underlying web-sys store.
    pub fn raw(&self) -> &web_sys::IdbObjectStore {
        &self.inner
    }
}

/// Builder for creating object stores during database upgrade.
pub struct ObjectStoreBuilder<'a> {
    db: &'a web_sys::IdbDatabase,
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
    indexes: Vec<(String, String, bool, bool)>, // (name, key_path, unique, multi_entry)
}

impl<'a> ObjectStoreBuilder<'a> {
    pub(crate) fn new(db: &'a web_sys::IdbDatabase, name: &str) -> Self {
        Self {
            db,
            name: name.to_string(),
            key_path: None,
            auto_increment: false,
            indexes: Vec::new(),
        }
    }

    /// Set the key path.
    pub fn key_path(mut self, path: &str) -> Self {
        self.key_path = Some(path.to_string());
        self
    }

    /// Enable auto-increment keys.
    pub fn auto_increment(mut self, enabled: bool) -> Self {
        self.auto_increment = enabled;
        self
    }

    /// Add an index to be created with the store.
    pub fn index(mut self, name: &str, key_path: &str) -> Self {
        self.indexes.push((name.to_string(), key_path.to_string(), false, false));
        self
    }

    /// Add a unique index.
    pub fn unique_index(mut self, name: &str, key_path: &str) -> Self {
        self.indexes.push((name.to_string(), key_path.to_string(), true, false));
        self
    }

    /// Add a multi-entry index.
    pub fn multi_entry_index(mut self, name: &str, key_path: &str) -> Self {
        self.indexes.push((name.to_string(), key_path.to_string(), false, true));
        self
    }

    /// Build and create the object store.
    pub fn build(self) -> IdbResult<ObjectStore> {
        let params = IdbObjectStoreParameters::new();

        if let Some(ref key_path) = self.key_path {
            let key_path_js: JsValue = key_path.into();
            params.set_key_path(&key_path_js);
        }
        params.set_auto_increment(self.auto_increment);

        let store = self
            .db
            .create_object_store_with_optional_parameters(&self.name, &params)
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        // Create indexes
        for (name, key_path, unique, multi_entry) in self.indexes {
            let index_params = web_sys::IdbIndexParameters::new();
            index_params.set_unique(unique);
            index_params.set_multi_entry(multi_entry);

            store
                .create_index_with_str_and_optional_parameters(&name, &key_path, &index_params)
                .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;
        }

        Ok(ObjectStore::new(store))
    }
}

