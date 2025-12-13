//! Transaction management.

use crate::error::{IdbError, IdbResult};
use crate::object_store::ObjectStore;
use crate::request::transaction_complete;

/// Transaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    /// Read-only transaction.
    ReadOnly,
    /// Read-write transaction.
    ReadWrite,
}

/// An IndexedDB transaction.
pub struct Transaction {
    inner: web_sys::IdbTransaction,
}

impl Transaction {
    pub(crate) fn new(inner: web_sys::IdbTransaction) -> Self {
        Self { inner }
    }

    /// Get an object store from this transaction.
    pub fn object_store(&self, name: &str) -> IdbResult<ObjectStore> {
        let store = self
            .inner
            .object_store(name)
            .map_err(|_| IdbError::StoreNotFound(name.to_string()))?;

        Ok(ObjectStore::new(store))
    }

    /// Alias for object_store.
    pub fn store(&self, name: &str) -> IdbResult<ObjectStore> {
        self.object_store(name)
    }

    /// Get the transaction mode.
    pub fn mode(&self) -> TransactionMode {
        match self.inner.mode() {
            Ok(m) if m == web_sys::IdbTransactionMode::Readwrite => TransactionMode::ReadWrite,
            _ => TransactionMode::ReadOnly,
        }
    }

    /// Abort the transaction.
    pub fn abort(&self) -> IdbResult<()> {
        self.inner
            .abort()
            .map_err(|e| IdbError::TransactionError(format!("{:?}", e)))
    }

    /// Wait for the transaction to complete.
    pub async fn commit(self) -> IdbResult<()> {
        transaction_complete(&self.inner).await
    }

    /// Get the underlying web-sys transaction.
    pub fn raw(&self) -> &web_sys::IdbTransaction {
        &self.inner
    }

    /// Get the database this transaction belongs to.
    pub fn db(&self) -> web_sys::IdbDatabase {
        self.inner.db()
    }

    /// Get the list of object store names in this transaction.
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
}

