#![allow(non_snake_case)]
use super::*;

#[wasm_bindgen()]
pub struct VeilidTableDB {
    inner_table_db: Option<TableDB>,
    tableName: String,
    columnCount: u32,
}

#[wasm_bindgen()]
impl VeilidTableDB {
    /// If the column count is greater than an existing TableDB's column count,
    /// the database will be upgraded to add the missing columns.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(tableName: String, columnCount: u32) -> Self {
        Self {
            inner_table_db: None,
            tableName,
            columnCount,
        }
    }

    fn getTableDB(&self) -> VeilidAPIResult<TableDB> {
        let Some(table_db) = &self.inner_table_db else {
            return VeilidAPIResult::Err(veilid_core::VeilidAPIError::generic(
                "Unable to getTableDB instance. Ensure you've called openTable().",
            ));
        };
        Ok(table_db.clone())
    }

    /// Get or create the TableDB database table.
    /// This is called automatically when performing actions on the TableDB.
    pub async fn openTable(&mut self) -> VeilidAPIResult<()> {
        let veilid_api = get_veilid_api()?;
        let tstore = veilid_api.table_store()?;
        let table_db = tstore
            .open(&self.tableName, self.columnCount)
            .await
            .map_err(veilid_core::VeilidAPIError::generic)?;
        self.inner_table_db = Some(table_db);
        Ok(())
    }

    /// Delete this TableDB.
    pub async fn deleteTable(&mut self) -> VeilidAPIResult<bool> {
        self.inner_table_db = None;

        let veilid_api = get_veilid_api()?;
        let tstore = veilid_api.table_store()?;
        tstore.delete(&self.tableName).await
    }

    async fn ensureOpen(&mut self) {
        if self.inner_table_db.is_none() {
            let _ = self.openTable().await;
        }
    }

    /// Read a key from a column in the TableDB immediately.
    pub async fn load(
        &mut self,
        columnId: u32,
        key: Box<[u8]>,
    ) -> VeilidAPIResult<Option<Uint8Array>> {
        self.ensureOpen().await;
        let table_db = self.getTableDB()?;

        let out = table_db.load(columnId, &key).await?;
        let out = out.map(|out| Uint8Array::from(out.as_slice()));
        Ok(out)
    }

    /// Store a key with a value in a column in the TableDB.
    /// Performs a single transaction immediately.
    pub async fn store(
        &mut self,
        columnId: u32,
        key: Box<[u8]>,
        value: Box<[u8]>,
    ) -> VeilidAPIResult<()> {
        self.ensureOpen().await;
        let table_db = self.getTableDB()?;

        table_db.store(columnId, &key, &value).await
    }

    /// Delete key with from a column in the TableDB.
    pub async fn delete(
        &mut self,
        columnId: u32,
        key: Box<[u8]>,
    ) -> VeilidAPIResult<Option<Uint8Array>> {
        self.ensureOpen().await;
        let table_db = self.getTableDB()?;

        let out = table_db.delete(columnId, &key).await?;
        let out = out.map(|out| Uint8Array::from(out.as_slice()));
        Ok(out)
    }

    /// Get the list of keys in a column of the TableDB.
    ///
    /// Returns an array of Uint8Array keys.
    pub async fn getKeys(&mut self, columnId: u32) -> VeilidAPIResult<Uint8ArrayArray> {
        self.ensureOpen().await;
        let table_db = self.getTableDB()?;

        let keys = table_db.clone().get_keys(columnId).await?;
        let out: Vec<Uint8Array> = keys
            .into_iter()
            .map(|k| Uint8Array::from(k.as_slice()))
            .collect();

        let out = into_unchecked_uint8array_array(out);

        Ok(out)
    }

    /// Start a TableDB write transaction.
    /// The transaction object must be committed or rolled back before dropping.
    pub async fn createTransaction(&mut self) -> VeilidAPIResult<VeilidTableDBTransaction> {
        self.ensureOpen().await;
        let table_db = self.getTableDB()?;

        let transaction = table_db.transact();
        Ok(VeilidTableDBTransaction {
            inner_transaction: Some(transaction),
        })
    }
}

#[wasm_bindgen]
pub struct VeilidTableDBTransaction {
    inner_transaction: Option<TableDBTransaction>,
}

#[wasm_bindgen]
impl VeilidTableDBTransaction {
    fn getTransaction(&self) -> VeilidAPIResult<TableDBTransaction> {
        let Some(transaction) = &self.inner_transaction else {
            return VeilidAPIResult::Err(veilid_core::VeilidAPIError::generic(
                "Unable to getTransaction instance. inner_transaction is None.",
            ));
        };
        Ok(transaction.clone())
    }

    /// Commit the transaction. Performs all actions atomically.
    pub async fn commit(&self) -> VeilidAPIResult<()> {
        let transaction = self.getTransaction()?;
        transaction.commit().await
    }

    /// Rollback the transaction. Does nothing to the TableDB.
    #[allow(clippy::unused_async)]
    pub async fn rollback(&self) -> VeilidAPIResult<()> {
        let transaction = self.getTransaction()?;
        transaction.rollback();
        Ok(())
    }

    /// Store a key with a value in a column in the TableDB.
    /// Does not modify TableDB until `.commit()` is called.
    pub async fn store(&self, col: u32, key: Box<[u8]>, value: Box<[u8]>) -> VeilidAPIResult<()> {
        let transaction = self.getTransaction()?;
        transaction.store(col, &key, &value).await
    }

    /// Delete key with from a column in the TableDB
    pub async fn deleteKey(&self, col: u32, key: Box<[u8]>) -> VeilidAPIResult<()> {
        let transaction = self.getTransaction()?;
        transaction.delete(col, &key).await
    }
}
