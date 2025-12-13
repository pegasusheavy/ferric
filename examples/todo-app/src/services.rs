//! Services for the todo application.

use crate::models::Todo;
use web_sys::Storage;

const STORAGE_KEY: &str = "ferric-todos";

/// Service for persisting todos to localStorage.
pub struct TodoStorage;

impl TodoStorage {
    /// Get the localStorage instance.
    fn storage() -> Option<Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    /// Load todos from localStorage.
    pub fn load() -> Vec<Todo> {
        Self::storage()
            .and_then(|s| s.get_item(STORAGE_KEY).ok())
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    /// Save todos to localStorage.
    pub fn save(todos: &[Todo]) {
        if let Some(storage) = Self::storage() {
            if let Ok(json) = serde_json::to_string(todos) {
                let _ = storage.set_item(STORAGE_KEY, &json);
            }
        }
    }

    /// Clear all todos from localStorage.
    pub fn clear() {
        if let Some(storage) = Self::storage() {
            let _ = storage.remove_item(STORAGE_KEY);
        }
    }
}

