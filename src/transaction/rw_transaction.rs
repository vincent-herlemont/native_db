use crate::db_type::{Input, Result, ToInput};
use crate::transaction::internal::rw_transaction::InternalRwTransaction;
use crate::transaction::query::RwDrain;
use crate::transaction::query::RwGet;
use crate::transaction::query::RwLen;
use crate::transaction::query::RwScan;
use crate::watch;
use crate::watch::Event;
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use super::internal::private_readable_transaction::PrivateReadableTransaction;

pub struct RwTransaction<'db> {
    pub(crate) watcher: &'db Arc<RwLock<watch::Watchers>>,
    pub(crate) batch: RefCell<watch::Batch>,
    pub(crate) internal: InternalRwTransaction<'db>,
}

impl<'db> RwTransaction<'db> {
    /// Get a value from the database.
    ///
    /// - [`primary`](crate::transaction::query::RGet::primary) - Get a item by primary key.
    /// - [`secondary`](crate::transaction::query::RGet::secondary) - Get a item by secondary key.
    pub fn get<'txn>(&'txn self) -> RwGet<'db, 'txn> {
        RwGet {
            internal: &self.internal,
        }
    }

    /// Get values from the database.
    ///
    /// - [`primary`](crate::transaction::query::RScan::primary) - Scan items by primary key.
    /// - [`secondary`](crate::transaction::query::RScan::secondary) - Scan items by secondary key.
    pub fn scan<'txn>(&'txn self) -> RwScan<'db, 'txn> {
        RwScan {
            internal: &self.internal,
        }
    }

    /// Get the number of values in the database.
    ///
    /// - [`primary`](crate::transaction::query::RLen::primary) - Get the number of items by primary key.
    /// - [`secondary`](crate::transaction::query::RLen::secondary) - Get the number of items by secondary key.
    pub fn len<'txn>(&'txn self) -> RwLen<'db, 'txn> {
        RwLen {
            internal: &self.internal,
        }
    }

    /// Drain values from the database.
    ///
    /// **TODO: needs to be improved, so don't use it yet.**
    pub fn drain<'txn>(&'txn self) -> RwDrain<'db, 'txn> {
        RwDrain {
            internal: &self.internal,
        }
    }
}

impl<'db, 'txn> RwTransaction<'db> {
    /// Commit the transaction.
    /// All changes will be applied to the database. If the commit fails, the transaction will be aborted. The
    /// database will be unchanged.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut models = Models::new();
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let rw = db.rw_transaction()?;
    ///     // Do some stuff..
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn commit(self) -> Result<()> {
        self.internal.commit()?;
        // Send batch to watchers after commit succeeds
        let batch = self.batch.into_inner();
        watch::push_batch(Arc::clone(self.watcher), batch)?;
        Ok(())
    }

    /// Abort the transaction.
    pub fn abort(self) -> Result<()> {
        Ok(self.internal.redb_transaction.abort()?)
    }
}

impl<'db, 'txn> RwTransaction<'db> {
    /// Insert a value into the database.
    ///
    /// If the primary key already exists, an error is returned.
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
    ///     let rw = db.rw_transaction()?;
    ///
    ///     // Insert a value
    ///     rw.insert(Data { id: 1 })?;
    ///
    ///     // /!\ Don't forget to commit the transaction
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn insert<T: ToInput>(&self, item: T) -> Result<()> {
        let (watcher_request, binary_value) = self
            .internal
            .concrete_insert(T::native_db_model(), item.native_db_input()?)?;
        let event = Event::new_insert(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    /// Upsert a value into the database.
    ///
    /// If the primary key already exists, the value is updated.
    ///
    /// Returns: the old value if the primary key already exists.
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
    ///     let rw = db.rw_transaction()?;
    ///
    ///     // Upsert a value
    ///     let old_value: Option<Data> = rw.upsert(Data { id: 1 })?;
    ///     assert!(old_value.is_none()); // Return None because the value does not exist
    ///
    ///     // Upsert the value again
    ///     let old_value: Option<Data> = rw.upsert(Data { id: 1 })?;
    ///     assert!(old_value.is_some()); // Return Some because the value already exist
    ///
    ///     // /!\ Don't forget to commit the transaction
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn upsert<T: ToInput>(&self, item: T) -> Result<Option<T>> {
        let model = T::native_db_model();
        let old_item: Option<Input> = self
            .internal
            .get_by_primary_key(model, item.native_db_primary_key())?
            .map(|item| item.inner())
            .transpose()?
            .map(|item: T| item.native_db_input())
            .transpose()?;
        let (watcher_request, new_binary_value, old_binary_value) = self.internal.concrete_upsert(
            T::native_db_model(),
            old_item,
            item.native_db_input()?,
        )?;
        if let Some(old_binary_value) = old_binary_value {
            let event = Event::new_update(old_binary_value.clone(), new_binary_value);
            self.batch.borrow_mut().add(watcher_request, event);
            let old_binary_value = old_binary_value.inner()?;
            Ok(Some(old_binary_value))
        } else {
            let event = Event::new_insert(new_binary_value);
            self.batch.borrow_mut().add(watcher_request, event);
            Ok(None)
        }
    }

    /// Remove a value from the database.
    ///
    /// Returns error:
    /// - [crate::db_type::Error::KeyNotFound] if the `item` has a primary key that is not found in the database.
    /// - [crate::db_type::Error::IncorrectInputData] if the `item` does not match the one in the database.
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
    ///     // Open a read/write transaction
    ///     let rw = db.rw_transaction()?;
    ///     // Insert a value
    ///     rw.insert(Data { id: 1 })?;
    ///
    ///     // Remove a value
    ///     rw.remove(Data { id: 1 })?;
    ///
    ///     // /!\ Don't forget to commit the transaction
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn remove<T: ToInput>(&self, item: T) -> Result<T> {
        let (watcher_request, binary_value) = self
            .internal
            .concrete_remove(T::native_db_model(), item.native_db_input()?)?;
        let event = Event::new_delete(binary_value.clone());
        self.batch.borrow_mut().add(watcher_request, event);
        binary_value.inner()
    }

    /// Update a value in the database.
    ///
    /// That allow to update all keys (primary and secondary) of the value.
    ///
    /// Returns error:
    /// - [crate::db_type::Error::KeyNotFound] if the `item` has a primary key that is not found in the database.
    /// - [crate::db_type::Error::IncorrectInputData] if the `item` does not match the one in the database.
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
    ///     // Open a read/write transaction
    ///     let rw = db.rw_transaction()?;
    ///     // Insert a value
    ///     rw.insert(Data { id: 1 })?;
    ///
    ///     // Update a value
    ///     rw.update(Data { id: 1 }, Data { id: 2 })?;
    ///
    ///     // /!\ Don't forget to commit the transaction
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn update<T: ToInput>(&self, old_item: T, updated_item: T) -> Result<()> {
        let (watcher_request, old_binary_value, new_binary_value) = self.internal.concrete_update(
            T::native_db_model(),
            old_item.native_db_input()?,
            updated_item.native_db_input()?,
        )?;
        let event = Event::new_update(old_binary_value, new_binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    /// Convert all values from the database.
    ///
    /// This is useful when you want to change the type/model of a value.
    /// You have to define [`From<SourceModel> for TargetModel`](https://doc.rust-lang.org/std/convert/trait.From.html) to convert the value.
    ///
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize, Clone)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Dog {
    ///     #[primary_key]
    ///     name: String,
    /// }
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=2, version=1)]
    /// #[native_db]
    /// struct Animal {
    ///     #[primary_key]
    ///     name: String,
    ///     #[secondary_key]
    ///     specie: String,
    /// }
    ///
    /// impl From<Dog> for Animal {
    ///     fn from(dog: Dog) -> Self {
    ///        Animal {
    ///           name: dog.name,
    ///           specie: "dog".to_string(),
    ///         }
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut models = Models::new();
    ///     models.define::<Dog>()?;
    ///     models.define::<Animal>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let rw = db.rw_transaction()?;
    ///
    ///     // Convert all values from Dog to Animal
    ///     rw.convert_all::<Dog, Animal>()?;
    ///
    ///     // /!\ Don't forget to commit the transaction
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn convert_all<OldType, NewType>(&self) -> Result<()>
    where
        OldType: ToInput + Clone,
        NewType: ToInput + From<OldType>,
    {
        let find_all_old: Result<Vec<OldType>> = self.scan().primary()?.all()?.collect();
        let find_all_old = find_all_old?;
        for old in find_all_old {
            let new: NewType = old.clone().into();
            self.internal
                .concrete_insert(NewType::native_db_model(), new.native_db_input()?)?;
            self.internal
                .concrete_remove(OldType::native_db_model(), old.native_db_input()?)?;
        }
        Ok(())
    }

    /// Automatically migrate the data from the old model to the new model. **No matter the state of the database**,
    /// if all models remain defined in the application as they are, the data will be migrated to the most recent version automatically.
    ///
    /// Native DB use the [`native_model`](https://crates.io/crates/native_model) identifier `id` to identify the model and `version` to identify the version of the model.
    /// We can define a model with the same identifier `id` but with a different version `version`.
    ///
    /// In the example below we define one model with the identifier `id=1` with tow versions `version=1` and `version=2`.
    /// - You **must** link the previous version from the new one with `from` option like `#[native_model(id=1, version=2, from=LegacyData)]`.
    /// - You **must** define the interoperability between the two versions with implement `From<LegacyData> for Data` and `From<Data> for LegacyData` or implement `TryFrom<LegacyData> for Data` and `TryFrom<Data> for LegacyData`.
    /// - You **must** define all models (by calling [`define`](crate::Models::define)) before to call [`migrate`](crate::transaction::RwTransaction::migrate).
    /// - You **must** call use the most recent/bigger version as the target version when you call [`migrate`](crate::transaction::RwTransaction::migrate): `migration::<Data>()`.
    ///   That means you can't call `migration::<LegacyData>()` because `LegacyData` has version `1` and `Data` has version `2`.
    ///
    /// After call `migration::<Data>()` all data of the model `LegacyData` will be migrated to the model `Data`.
    ///
    /// Under the hood, when you call [`migrate`](crate::transaction::RwTransaction::migrate) `native_model` is used to convert the data from the old model to the new model
    /// using the `From` or `TryFrom` implementation for each to target the version defined when you call [`migrate::<LastVersion>()`](crate::transaction::RwTransaction::migrate).
    ///
    /// It's advisable to perform all migrations within a **single transaction** to ensure that all migrations are successfully completed.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct LegacyData {
    ///     #[primary_key]
    ///     id: u32,
    /// }
    ///
    /// impl From<Data> for LegacyData {
    ///     fn from(data: Data) -> Self {
    ///         LegacyData {
    ///             id: data.id as u32,
    ///         }
    ///     }
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// #[native_model(id=1, version=2, from=LegacyData)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    /// }
    ///
    /// impl From<LegacyData> for Data {
    ///     fn from(legacy_data: LegacyData) -> Self {
    ///         Data {
    ///             id: legacy_data.id as u64,
    ///         }
    ///     }
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut models = Models::new();
    ///     models.define::<LegacyData>()?;
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///
    ///     let rw = db.rw_transaction()?;
    ///     rw.migrate::<Data>()?;
    ///     // Other migrations if needed..
    ///     rw.commit()
    /// }
    /// ```
    pub fn migrate<T: ToInput + Debug>(&self) -> Result<()> {
        self.internal.migrate::<T>()
    }

    /// Refresh the data for the given model. Is used generally when during an database upgrade,
    /// using the method [crate::Database::upgrading_from_version] (more details/example). Check release notes to know
    /// when to use this method.
    pub fn refresh<T: ToInput + Debug>(&self) -> Result<()> {
        self.internal.refresh::<T>()
    }
}
