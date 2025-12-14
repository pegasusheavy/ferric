//! Database management.

use crate::error::{IdbError, IdbResult};
use crate::object_store::ObjectStoreBuilder;
use crate::request::open_request_to_future;
use crate::transaction::{Transaction, TransactionMode};
use crate::get_idb_factory;
use wasm_bindgen::JsCast;

/// An IndexedDB database.
#[derive(Clone)]
pub struct Database {
    inner: web_sys::IdbDatabase,
}

impl Database {
    /// Open a database with the given name.
    pub fn open(name: &str) -> DatabaseBuilder {
        DatabaseBuilder::new(name)
    }

    /// Get the database name.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get the database version.
    pub fn version(&self) -> u32 {
        self.inner.version() as u32
    }

    /// Get the list of object store names.
    pub fn object_store_names(&self) -> Vec<String> {
        let list = self.inner.object_store_names();
        let mut names = Vec::with_capacity(list.length() as usize);
        for i in 0..list.length() {
            if let Some(name) = list.get(i) {
                names.push(name);
            }
        }
        names
    }

    /// Check if an object store exists.
    pub fn has_object_store(&self, name: &str) -> bool {
        self.inner.object_store_names().contains(name)
    }

    /// Start a transaction.
    pub fn transaction(&self, store_names: &[&str]) -> TransactionBuilder {
        TransactionBuilder::new(self.inner.clone(), store_names)
    }

    /// Start a transaction on a single store (convenience method).
    pub fn transaction_on(&self, store_name: &str) -> TransactionBuilder {
        TransactionBuilder::new(self.inner.clone(), &[store_name])
    }

    /// Close the database.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Get the underlying web-sys database.
    pub fn raw(&self) -> &web_sys::IdbDatabase {
        &self.inner
    }

    /// Delete a database by name.
    pub async fn delete(name: &str) -> IdbResult<()> {
        let factory = get_idb_factory()?;
        let request = factory
            .delete_database(name)
            .map_err(|e| IdbError::JsError(format!("{:?}", e)))?;

        crate::request::request_to_future(&request.unchecked_into()).await?;
        Ok(())
    }
}

/// Builder for opening a database.
pub struct DatabaseBuilder {
    name: String,
    version: Option<u32>,
    on_upgrade: Option<Box<dyn FnOnce(&UpgradeContext) -> IdbResult<()>>>,
}

impl DatabaseBuilder {
    /// Create a new database builder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: None,
            on_upgrade: None,
        }
    }

    /// Set the database version.
    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    /// Set the upgrade handler.
    ///
    /// This is called when the database is created or upgraded.
    pub fn on_upgrade<F>(mut self, handler: F) -> Self
    where
        F: FnOnce(&UpgradeContext) -> IdbResult<()> + 'static,
    {
        self.on_upgrade = Some(Box::new(handler));
        self
    }

    /// Open the database.
    pub async fn build(self) -> IdbResult<Database> {
        let factory = get_idb_factory()?;

        let request = if let Some(version) = self.version {
            factory
                .open_with_u32(&self.name, version)
                .map_err(|e| IdbError::OpenFailed(format!("{:?}", e)))?
        } else {
            factory
                .open(&self.name)
                .map_err(|e| IdbError::OpenFailed(format!("{:?}", e)))?
        };

        let on_upgrade = self.on_upgrade.map(|handler| {
            move |db: &web_sys::IdbDatabase, old_version: u32, new_version: u32| {
                let ctx = UpgradeContext {
                    db: db.clone(),
                    old_version,
                    new_version,
                };
                handler(&ctx)
            }
        });

        let inner = open_request_to_future(&request, on_upgrade).await?;

        Ok(Database { inner })
    }
}

/// Context provided during database upgrade.
pub struct UpgradeContext {
    db: web_sys::IdbDatabase,
    /// The old database version (0 if new database).
    pub old_version: u32,
    /// The new database version.
    pub new_version: u32,
}

impl UpgradeContext {
    /// Create an object store builder.
    pub fn create_object_store(&self, name: &str) -> ObjectStoreBuilder<'_> {
        ObjectStoreBuilder::new(&self.db, name)
    }

    /// Delete an object store.
    pub fn delete_object_store(&self, name: &str) -> IdbResult<()> {
        self.db
            .delete_object_store(name)
            .map_err(|e| IdbError::StoreNotFound(format!("{}: {:?}", name, e)))
    }

    /// Check if an object store exists.
    pub fn has_object_store(&self, name: &str) -> bool {
        self.db.object_store_names().contains(name)
    }

    /// Get the list of existing object store names.
    pub fn object_store_names(&self) -> Vec<String> {
        let list = self.db.object_store_names();
        let mut names = Vec::with_capacity(list.length() as usize);
        for i in 0..list.length() {
            if let Some(name) = list.get(i) {
                names.push(name);
            }
        }
        names
    }
}

/// Builder for creating transactions.
pub struct TransactionBuilder {
    db: web_sys::IdbDatabase,
    store_names: Vec<String>,
    mode: TransactionMode,
}

impl TransactionBuilder {
    fn new(db: web_sys::IdbDatabase, store_names: &[&str]) -> Self {
        Self {
            db,
            store_names: store_names.iter().map(|s| s.to_string()).collect(),
            mode: TransactionMode::ReadOnly,
        }
    }

    /// Set the transaction mode to read-only (default).
    pub fn readonly(mut self) -> Self {
        self.mode = TransactionMode::ReadOnly;
        self
    }

    /// Set the transaction mode to read-write.
    pub fn rw(mut self) -> Self {
        self.mode = TransactionMode::ReadWrite;
        self
    }

    /// Set the transaction mode to read-write (alias for rw).
    pub fn readwrite(self) -> Self {
        self.rw()
    }

    /// Build and start the transaction.
    pub fn build(self) -> IdbResult<Transaction> {
        let mode = match self.mode {
            TransactionMode::ReadOnly => web_sys::IdbTransactionMode::Readonly,
            TransactionMode::ReadWrite => web_sys::IdbTransactionMode::Readwrite,
        };

        // Convert store names to js array
        let store_names_js = js_sys::Array::new();
        for name in &self.store_names {
            store_names_js.push(&name.into());
        }

        let inner = self
            .db
            .transaction_with_str_sequence_and_mode(&store_names_js, mode)
            .map_err(|e| IdbError::TransactionError(format!("{:?}", e)))?;

        Ok(Transaction::new(inner))
    }
}

