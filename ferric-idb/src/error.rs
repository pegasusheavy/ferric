//! Error types for IndexedDB operations.

use thiserror::Error;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;

/// Result type for IndexedDB operations.
pub type IdbResult<T> = Result<T, IdbError>;

/// Errors that can occur during IndexedDB operations.
#[derive(Error, Debug)]
pub enum IdbError {
    /// IndexedDB is not available in this environment.
    #[error("IndexedDB not available: {0}")]
    NotAvailable(&'static str),

    /// Database open failed.
    #[error("Failed to open database: {0}")]
    OpenFailed(String),

    /// Database version error.
    #[error("Database version error: {0}")]
    VersionError(String),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Object store not found.
    #[error("Object store not found: {0}")]
    StoreNotFound(String),

    /// Index not found.
    #[error("Index not found: {0}")]
    IndexNotFound(String),

    /// Key error.
    #[error("Key error: {0}")]
    KeyError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Cursor error.
    #[error("Cursor error: {0}")]
    CursorError(String),

    /// DOM exception.
    #[error("DOM exception: {name} - {message}")]
    DomException { name: String, message: String },

    /// JavaScript error.
    #[error("JavaScript error: {0}")]
    JsError(String),

    /// Request was aborted.
    #[error("Request aborted")]
    Aborted,

    /// Constraint error (e.g., duplicate key).
    #[error("Constraint error: {0}")]
    ConstraintError(String),

    /// Data error.
    #[error("Data error: {0}")]
    DataError(String),

    /// Invalid state.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Read-only transaction.
    #[error("Cannot write in read-only transaction")]
    ReadOnlyTransaction,

    /// Transaction inactive.
    #[error("Transaction is inactive")]
    TransactionInactive,

    /// Unknown error.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<JsValue> for IdbError {
    fn from(value: JsValue) -> Self {
        // Try to extract DOMException details
        if let Some(exception) = value.dyn_ref::<web_sys::DomException>() {
            let name: String = exception.name();
            let message: String = exception.message();
            return IdbError::DomException { name, message };
        }

        // Try to get string representation
        if let Some(s) = value.as_string() {
            return IdbError::JsError(s);
        }

        // Try to convert via debug
        IdbError::JsError(format!("{:?}", value))
    }
}

impl From<serde_wasm_bindgen::Error> for IdbError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        IdbError::SerializationError(err.to_string())
    }
}

impl IdbError {
    /// Create an error from a DOMException name.
    pub fn from_dom_exception_name(name: &str, context: &str) -> Self {
        match name {
            "AbortError" => IdbError::Aborted,
            "ConstraintError" => IdbError::ConstraintError(context.to_string()),
            "DataError" => IdbError::DataError(context.to_string()),
            "InvalidStateError" => IdbError::InvalidState(context.to_string()),
            "NotFoundError" => IdbError::NotFound(context.to_string()),
            "ReadOnlyError" => IdbError::ReadOnlyTransaction,
            "TransactionInactiveError" => IdbError::TransactionInactive,
            "VersionError" => IdbError::VersionError(context.to_string()),
            _ => IdbError::Unknown(format!("{}: {}", name, context)),
        }
    }
}

