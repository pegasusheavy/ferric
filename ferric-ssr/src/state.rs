//! State serialization for client hydration.

use crate::error::{SsrError, SsrResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serialized state for client-side hydration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedState {
    /// The JSON-serialized state data.
    data: String,
}

impl SerializedState {
    /// Create a new serialized state from a JSON string.
    pub fn from_json(json: String) -> Self {
        Self { data: json }
    }

    /// Get the JSON string.
    pub fn as_json(&self) -> &str {
        &self.data
    }

    /// Deserialize into a specific type.
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> SsrResult<T> {
        serde_json::from_str(&self.data).map_err(SsrError::from)
    }

    /// Check if the state is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() || self.data == "{}" || self.data == "null"
    }
}

impl Default for SerializedState {
    fn default() -> Self {
        Self {
            data: "{}".to_string(),
        }
    }
}

/// Utility for serializing component state.
pub struct StateSerializer;

impl StateSerializer {
    /// Serialize a value to a SerializedState.
    pub fn serialize<T: Serialize>(value: &T) -> SsrResult<SerializedState> {
        let json = serde_json::to_string(value)?;
        Ok(SerializedState::from_json(json))
    }

    /// Serialize a map of values.
    pub fn serialize_map(map: &HashMap<String, serde_json::Value>) -> SsrResult<SerializedState> {
        let json = serde_json::to_string(map)?;
        Ok(SerializedState::from_json(json))
    }

    /// Create an empty state.
    pub fn empty() -> SerializedState {
        SerializedState::default()
    }
}

/// Builder for constructing state incrementally.
#[derive(Debug, Default)]
pub struct StateBuilder {
    state: HashMap<String, serde_json::Value>,
}

impl StateBuilder {
    /// Create a new state builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a value to the state.
    pub fn add<T: Serialize>(mut self, key: &str, value: &T) -> SsrResult<Self> {
        let json = serde_json::to_value(value)?;
        self.state.insert(key.to_string(), json);
        Ok(self)
    }

    /// Add a raw JSON value.
    pub fn add_raw(mut self, key: &str, value: serde_json::Value) -> Self {
        self.state.insert(key.to_string(), value);
        self
    }

    /// Build the serialized state.
    pub fn build(self) -> SsrResult<SerializedState> {
        StateSerializer::serialize_map(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        count: i32,
        name: String,
    }

    #[test]
    fn test_serialize_deserialize() {
        let data = TestData {
            count: 42,
            name: "test".to_string(),
        };

        let state = StateSerializer::serialize(&data).unwrap();
        let restored: TestData = state.deserialize().unwrap();

        assert_eq!(data, restored);
    }

    #[test]
    fn test_state_builder() {
        let state = StateBuilder::new()
            .add("count", &42)
            .unwrap()
            .add("name", &"test")
            .unwrap()
            .build()
            .unwrap();

        assert!(state.as_json().contains("\"count\":42"));
        assert!(state.as_json().contains("\"name\":\"test\""));
    }
}

