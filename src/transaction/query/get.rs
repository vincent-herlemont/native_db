use crate::db_type::{Error, KeyOptions, Result, ToInput, ToKey, ToKeyDefinition};
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
    pub fn primary<T: ToInput, K: ToKey>(&self, key: K) -> Result<Option<T>> {
        let model = T::native_db_model();
        {
            let expected_key_types: Vec<String> = model.primary_key.rust_types.clone();
            if !K::key_names()
                .iter()
                .any(|name| expected_key_types.contains(name))
            {
                return Err(Error::MissmatchedKeyType {
                    key_name: model.primary_key.unique_table_name.to_string(),
                    expected_types: expected_key_types,
                    got_types: K::key_names(),
                    operation: "get".to_string(),
                });
            }
        }
        let result = self.internal.get_by_primary_key(model, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }

    /// Get a value from the database by secondary key.
    ///
    /// /!\ The secondary key **must** be [`unique`](crate::models::Models#unique) else this method will return an error [`SecondaryKeyConstraintMismatch`](crate::db_type::Error::SecondaryKeyConstraintMismatch).
    ///     If the secondary key is not unique, use [`scan()`](crate::transaction::RTransaction::scan) instead.
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
        let model = T::native_db_model();
        let result = self.internal.get_by_secondary_key(model, key_def, key)?;
        if let Some(value) = result {
            Ok(Some(value.inner()?))
        } else {
            Ok(None)
        }
    }
}
