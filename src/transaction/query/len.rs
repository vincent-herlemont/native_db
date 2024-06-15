use crate::db_type::{KeyOptions, Result, ToInput, ToKeyDefinition};
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

/// Get the number of values in the database.
pub struct RLen<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}

impl RLen<'_, '_> {
    /// Get the number of values.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut models = Models::new();
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get all values
    ///     let _number:u64 = r.len().primary::<Data>()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn primary<T: ToInput>(&self) -> Result<u64> {
        let model = T::native_db_model();
        let result = self.internal.primary_len(model)?;
        Ok(result)
    }

    /// **TODO: needs to be implemented**
    ///
    /// Get the number of values by secondary key.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// If the secondary key is [`optional`](struct.Builder.html#optional) you will
    /// get all values that have the secondary key set.
    pub fn secondary<T: ToInput>(
        &self,
        _key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<Option<T>> {
        todo!()
    }
}

pub struct RwLen<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl RwLen<'_, '_> {
    /// Get the number of values.
    ///
    /// Same as [`RLen::primary()`](struct.RLen.html#method.primary).
    pub fn primary<T: ToInput>(&self) -> Result<u64> {
        let model = T::native_db_model();
        let result = self.internal.primary_len(model)?;
        Ok(result)
    }

    /// Get the number of values by secondary key.
    ///
    /// Same as [`RLen::secondary()`](struct.RLen.html#method.secondary).
    pub fn secondary<T: ToInput>(
        &self,
        _key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<Option<T>> {
        todo!()
    }
}
