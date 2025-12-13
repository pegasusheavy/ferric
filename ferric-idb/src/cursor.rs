//! Cursor for iterating over records.

use crate::error::{IdbError, IdbResult};
use crate::request::request_to_future;
use js_sys::Promise;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbCursorWithValue, IdbRequest};

/// Cursor direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    /// Forward iteration.
    Next,
    /// Forward iteration, skipping duplicates.
    NextUnique,
    /// Backward iteration.
    Prev,
    /// Backward iteration, skipping duplicates.
    PrevUnique,
}

impl From<CursorDirection> for web_sys::IdbCursorDirection {
    fn from(dir: CursorDirection) -> Self {
        match dir {
            CursorDirection::Next => web_sys::IdbCursorDirection::Next,
            CursorDirection::NextUnique => web_sys::IdbCursorDirection::Nextunique,
            CursorDirection::Prev => web_sys::IdbCursorDirection::Prev,
            CursorDirection::PrevUnique => web_sys::IdbCursorDirection::Prevunique,
        }
    }
}

/// A cursor for iterating over records.
pub struct Cursor {
    request: IdbRequest,
    current: Option<IdbCursorWithValue>,
}

impl Cursor {
    pub(crate) async fn from_request(request: IdbRequest) -> IdbResult<Self> {
        // Wait for initial cursor
        let result = request_to_future(&request).await?;

        let current = if result.is_null() || result.is_undefined() {
            None
        } else {
            result.dyn_into::<IdbCursorWithValue>().ok()
        };

        Ok(Self { request, current })
    }

    /// Check if the cursor has more records.
    pub fn has_value(&self) -> bool {
        self.current.is_some()
    }

    /// Get the current key.
    pub fn key(&self) -> Option<JsValue> {
        self.current.as_ref().and_then(|c| c.key().ok())
    }

    /// Get the current primary key.
    pub fn primary_key(&self) -> Option<JsValue> {
        self.current.as_ref().and_then(|c| c.primary_key().ok())
    }

    /// Get the current value as a typed value.
    pub fn value<T: DeserializeOwned>(&self) -> IdbResult<Option<T>> {
        match &self.current {
            Some(cursor) => {
                let js_value = cursor.value().map_err(IdbError::from)?;
                if js_value.is_null() || js_value.is_undefined() {
                    Ok(None)
                } else {
                    let value: T = serde_wasm_bindgen::from_value(js_value)
                        .map_err(|e| IdbError::DeserializationError(e.to_string()))?;
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }

    /// Get the current value as raw JsValue.
    pub fn value_raw(&self) -> IdbResult<Option<JsValue>> {
        match &self.current {
            Some(cursor) => {
                let js_value = cursor.value().map_err(IdbError::from)?;
                if js_value.is_null() || js_value.is_undefined() {
                    Ok(None)
                } else {
                    Ok(Some(js_value))
                }
            }
            None => Ok(None),
        }
    }

    /// Move to the next record and return the value.
    pub async fn next<T: DeserializeOwned>(&mut self) -> IdbResult<Option<T>> {
        if self.current.is_none() {
            return Ok(None);
        }

        // Get current value before advancing
        let value = self.value()?;

        // Advance cursor
        self.advance_internal().await?;

        Ok(value)
    }

    /// Move to the next record without returning value.
    pub async fn advance(&mut self) -> IdbResult<bool> {
        if self.current.is_none() {
            return Ok(false);
        }

        self.advance_internal().await?;
        Ok(self.current.is_some())
    }

    /// Continue to a specific key.
    pub async fn continue_to<K: Into<JsValue>>(&mut self, key: K) -> IdbResult<bool> {
        if let Some(ref cursor) = self.current {
            cursor.continue_with_key(&key.into()).map_err(IdbError::from)?;
            self.wait_for_cursor().await?;
        }
        Ok(self.current.is_some())
    }

    /// Update the current record.
    pub async fn update<T: serde::Serialize>(&self, value: &T) -> IdbResult<()> {
        if let Some(ref cursor) = self.current {
            let js_value = serde_wasm_bindgen::to_value(value)?;
            let request = cursor.update(&js_value).map_err(IdbError::from)?;
            request_to_future(&request).await?;
        }
        Ok(())
    }

    /// Delete the current record.
    pub async fn delete(&self) -> IdbResult<()> {
        if let Some(ref cursor) = self.current {
            let request = cursor.delete().map_err(IdbError::from)?;
            request_to_future(&request).await?;
        }
        Ok(())
    }

    async fn advance_internal(&mut self) -> IdbResult<()> {
        if let Some(ref cursor) = self.current {
            cursor.continue_().map_err(IdbError::from)?;
            self.wait_for_cursor().await?;
        }
        Ok(())
    }

    async fn wait_for_cursor(&mut self) -> IdbResult<()> {
        let promise = Promise::new(&mut |resolve, reject| {
            let on_success = Closure::once(Box::new(move |_: web_sys::Event| {
                resolve.call0(&JsValue::UNDEFINED).unwrap();
            }) as Box<dyn FnOnce(_)>);

            let on_error = Closure::once(Box::new(move |_: web_sys::Event| {
                reject.call0(&JsValue::UNDEFINED).unwrap();
            }) as Box<dyn FnOnce(_)>);

            self.request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            self.request.set_onerror(Some(on_error.as_ref().unchecked_ref()));

            on_success.forget();
            on_error.forget();
        });

        JsFuture::from(promise).await?;

        let result = self.request.result().map_err(IdbError::from)?;
        self.current = if result.is_null() || result.is_undefined() {
            None
        } else {
            result.dyn_into::<IdbCursorWithValue>().ok()
        };

        Ok(())
    }

    /// Collect all remaining records into a Vec.
    pub async fn collect<T: DeserializeOwned>(&mut self) -> IdbResult<Vec<T>> {
        let mut results = Vec::new();
        while let Some(value) = self.next().await? {
            results.push(value);
        }
        Ok(results)
    }

    /// Iterate with a callback for each record.
    pub async fn for_each<T, F>(&mut self, mut callback: F) -> IdbResult<()>
    where
        T: DeserializeOwned,
        F: FnMut(T),
    {
        while let Some(value) = self.next::<T>().await? {
            callback(value);
        }
        Ok(())
    }

    /// Find the first record matching a predicate.
    pub async fn find<T, F>(&mut self, predicate: F) -> IdbResult<Option<T>>
    where
        T: DeserializeOwned,
        F: Fn(&T) -> bool,
    {
        while let Some(value) = self.next::<T>().await? {
            if predicate(&value) {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    /// Filter records matching a predicate.
    pub async fn filter<T, F>(&mut self, predicate: F) -> IdbResult<Vec<T>>
    where
        T: DeserializeOwned,
        F: Fn(&T) -> bool,
    {
        let mut results = Vec::new();
        while let Some(value) = self.next::<T>().await? {
            if predicate(&value) {
                results.push(value);
            }
        }
        Ok(results)
    }

    /// Take up to n records.
    pub async fn take<T: DeserializeOwned>(&mut self, n: usize) -> IdbResult<Vec<T>> {
        let mut results = Vec::with_capacity(n);
        while results.len() < n {
            match self.next().await? {
                Some(value) => results.push(value),
                None => break,
            }
        }
        Ok(results)
    }

    /// Skip n records.
    pub async fn skip(&mut self, n: usize) -> IdbResult<()> {
        for _ in 0..n {
            if !self.advance().await? {
                break;
            }
        }
        Ok(())
    }
}

