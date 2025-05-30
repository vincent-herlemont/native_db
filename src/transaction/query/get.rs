use crate::db_type::{
    check_key_type, check_key_type_from_key_definition, KeyOptions, Result, ToInput, ToKey,
    ToKeyDefinition,
};
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
    /// use native_db::native_model::{native_model, Model};
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
    ///     // Get a value by primary key
    ///     let _value: Option<Data> = r.get().primary(1u64)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn primary<T: ToInput>(&self, key: impl ToKey) -> Result<Option<T>> {
        let model = T::native_db_model();
        check_key_type(&model, &key)?;
        let result = self.internal.get_by_primary_key(model, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }

    /// Get a value from the database by secondary key.
    ///
    /// /!\ The secondary key **must** be [`unique`](crate::models::Models#unique); otherwise, this method will return an error [`SecondaryKeyConstraintMismatch`](crate::db_type::Error::SecondaryKeyConstraintMismatch).
    ///     If the secondary key is not unique, use [`scan()`](crate::transaction::RTransaction::scan) instead.
    ///
    /// The anatomy of a secondary key is an `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///     let mut models = Models::new();
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get a value by primary key
    ///     let _value: Option<Data> = r.get().secondary(DataKey::name, "test")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
        key: impl ToKey,
    ) -> Result<Option<T>> {
        let model = T::native_db_model();
        check_key_type_from_key_definition(&key_def.key_definition(), &key)?;
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }
}

pub struct RwGet<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl RwGet<'_, '_> {
    /// Get a value from the database by primary key.
    ///
    /// See [`primary`](crate::transaction::query::RGet::primary).
    pub fn primary<T: ToInput>(&self, key: impl ToKey) -> Result<Option<T>> {
        let model = T::native_db_model();
        check_key_type(&model, &key)?;
        let result = self.internal.get_by_primary_key(model, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }

    /// Get a value from the database by secondary key.
    ///
    /// See [`secondary`](crate::transaction::query::RGet::secondary).
    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
        key: impl ToKey,
    ) -> Result<Option<T>> {
        check_key_type_from_key_definition(&key_def.key_definition(), &key)?;
        let model = T::native_db_model();
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }
}
