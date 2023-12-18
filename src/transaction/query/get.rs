use crate::db_type::{DatabaseSecondaryKeyOptions, InnerKeyValue, Input, KeyDefinition, Result};
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

/// Get a value from the database.
pub struct RGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}

impl RGet<'_, '_> {
    /// Get a value from the database by primary key.
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
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get a value by primary key
    ///     let _value: Option<Data> = r.get().primary(1u64)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn primary<T: Input>(&self, key: impl InnerKeyValue) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_primary_key(model, key)?;
        Ok(result.map(|value| value.inner()))
    }

    /// Get a value from the database by secondary key.
    ///
    /// /!\ The secondary key **must** be [`unique`](struct.DatabaseBuilder.html#unique) else this method will return an error [`SecondaryKeyConstraintMismatch`](db_type/enum.Error.html#variant.SecondaryKeyConstraintMismatch).
    ///     If the secondary key is not unique, use [`scan()`](transaction/struct.RTransaction.html#method.scan) instead.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
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
    ///     #[secondary_key(unique)] // Must be unique to use get()
    ///     name: String,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get a value by primary key
    ///     let _value: Option<Data> = r.get().secondary(DataKey::name, "test")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn secondary<T: Input>(
        &self,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
        key: impl InnerKeyValue,
    ) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        Ok(result.map(|value| value.inner()))
    }
}

pub struct RwGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl RwGet<'_, '_> {
    /// Get a value from the database by primary key.
    ///
    /// Same as [`RGet::primary()`](struct.RGet.html#method.primary).
    pub fn primary<T: Input>(&self, key: impl InnerKeyValue) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_primary_key(model, key)?;
        Ok(result.map(|value| value.inner()))
    }

    /// Get a value from the database by secondary key.
    ///
    /// Same as [`RGet::secondary()`](struct.RGet.html#method.secondary).
    pub fn secondary<T: Input>(
        &self,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
        key: impl InnerKeyValue,
    ) -> Result<Option<T>> {
        let model = T::native_db_model();
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        Ok(result.map(|value| value.inner()))
    }
}
