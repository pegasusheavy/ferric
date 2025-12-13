//! # Ferric IDB
//!
//! An ergonomic async wrapper for IndexedDB in WebAssembly.
//!
//! IndexedDB has a callback-based API that is notoriously difficult to use.
//! This crate provides a clean async/await interface that feels natural in Rust.
//!
//! ## Features
//!
//! - **Async/await API** - No callbacks, just clean async code
//! - **Type-safe** - Serde integration for automatic serialization
//! - **Ergonomic** - Builder patterns for database and store configuration
//! - **Complete** - Full IndexedDB feature support (indexes, cursors, transactions)
//!
//! ## Quick Start
//!
//! ```ignore
//! use ferric_idb::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! async fn example() -> Result<(), IdbError> {
//!     // Open database with schema
//!     let db = Database::open("my-app")
//!         .version(1)
//!         .on_upgrade(|db, _old, _new| {
//!             db.create_object_store("users")
//!                 .key_path("id")
//!                 .auto_increment(true)
//!                 .build()?;
//!             Ok(())
//!         })
//!         .build()
//!         .await?;
//!
//!     // Insert data
//!     let store = db.transaction("users").rw().store("users")?;
//!     store.put(&User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }).await?;
//!
//!     // Query data
//!     let user: Option<User> = store.get(1).await?;
//!
//!     // Iterate with cursor
//!     let mut cursor = store.open_cursor().await?;
//!     while let Some(record) = cursor.next::<User>().await? {
//!         println!("{}: {}", record.id, record.name);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod database;
mod error;
mod object_store;
mod transaction;
mod cursor;
mod index;
mod key_range;
mod request;

pub use database::{Database, DatabaseBuilder, UpgradeContext};
pub use error::{IdbError, IdbResult};
pub use object_store::{ObjectStore, ObjectStoreBuilder};
pub use transaction::{Transaction, TransactionMode};
pub use cursor::{Cursor, CursorDirection};
pub use index::{Index, IndexBuilder};
pub use key_range::KeyRange;

/// Prelude module - import everything you need with `use ferric_idb::prelude::*`
pub mod prelude {
    pub use crate::database::{Database, DatabaseBuilder, UpgradeContext};
    pub use crate::error::{IdbError, IdbResult};
    pub use crate::object_store::{ObjectStore, ObjectStoreBuilder};
    pub use crate::transaction::{Transaction, TransactionMode};
    pub use crate::cursor::{Cursor, CursorDirection};
    pub use crate::index::{Index, IndexBuilder};
    pub use crate::key_range::KeyRange;
}

/// Get the IndexedDB factory from the window.
pub(crate) fn get_idb_factory() -> IdbResult<web_sys::IdbFactory> {
    web_sys::window()
        .ok_or(IdbError::NotAvailable("No window object"))?
        .indexed_db()
        .map_err(|_| IdbError::NotAvailable("IndexedDB not supported"))?
        .ok_or(IdbError::NotAvailable("IndexedDB is null"))
}

