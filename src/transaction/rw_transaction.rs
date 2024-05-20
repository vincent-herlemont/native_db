use crate::db_type::{Input, Result};
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

pub struct RwTransaction<'db> {
    pub(crate) watcher: &'db Arc<RwLock<watch::Watchers>>,
    pub(crate) batch: RefCell<watch::Batch>,
    pub(crate) internal: InternalRwTransaction<'db>,
}

impl<'db> RwTransaction<'db> {
    /// Get a value from the database.
    ///
    /// Same as [`RTransaction::get()`](struct.RTransaction.html#method.get).
    pub fn get<'txn>(&'txn self) -> RwGet<'db, 'txn> {
        RwGet {
            internal: &self.internal,
        }
    }

    /// Get values from the database.
    ///
    /// Same as [`RTransaction::scan()`](struct.RTransaction.html#method.scan).
    pub fn scan<'txn>(&'txn self) -> RwScan<'db, 'txn> {
        RwScan {
            internal: &self.internal,
        }
    }

    /// Get the number of values in the database.
    ///
    /// Same as [`RTransaction::len()`](struct.RTransaction.html#method.len).
    pub fn len<'txn>(&'txn self) -> RwLen<'db, 'txn> {
        RwLen {
            internal: &self.internal,
        }
    }

    /// Get all values from the database.
    ///
    /// Same as [`RTransaction::drain()`](struct.RTransaction.html#method.drain).
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
    ///     let mut builder = DatabaseBuilder::new();
    ///     let db = builder.create_in_memory()?;
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
        watch::push_batch(Arc::clone(&self.watcher), batch)?;
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
    pub fn insert<T: Input>(&self, item: T) -> Result<()> {
        let (watcher_request, binary_value) = self
            .internal
            .concrete_insert(T::native_db_model(), item.to_item()?)?;
        let event = Event::new_insert(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    /// Remove a value from the database.
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
    ///     let rw = db.rw_transaction()?;
    ///
    ///     // Remove a value
    ///     let old_value = rw.remove(Data { id: 1 })?;
    ///
    ///     // /!\ Don't forget to commit the transaction
    ///     rw.commit()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn remove<T: Input>(&self, item: T) -> Result<T> {
        let (watcher_request, binary_value) = self
            .internal
            .concrete_remove(T::native_db_model(), item.to_item()?)?;
        let event = Event::new_delete(binary_value.clone());
        self.batch.borrow_mut().add(watcher_request, event);
        binary_value.inner()
    }

    /// Update a value in the database.
    ///
    /// That allow to update all keys (primary and secondary) of the value.
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
    ///     let rw = db.rw_transaction()?;
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
    pub fn update<T: Input>(&self, old_item: T, updated_item: T) -> Result<()> {
        let (watcher_request, old_binary_value, new_binary_value) = self.internal.concrete_update(
            T::native_db_model(),
            old_item.to_item()?,
            updated_item.to_item()?,
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
    /// use native_model::{native_model, Model};
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
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Dog>()?;
    ///     builder.define::<Animal>()?;
    ///     let db = builder.create_in_memory()?;
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
        OldType: Input + Clone,
        NewType: Input + From<OldType>,
    {
        let find_all_old: Result<Vec<OldType>> = self.scan().primary()?.all().collect();
        let find_all_old = find_all_old?;
        for old in find_all_old {
            let new: NewType = old.clone().into();
            self.internal
                .concrete_insert(NewType::native_db_model(), new.to_item()?)?;
            self.internal
                .concrete_remove(OldType::native_db_model(), old.to_item()?)?;
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
    /// - You **must** define all models (by calling [`define`](#method.define)) before to call [`migration`](#method.migrate).
    /// - You **must** call use the most recent/bigger version as the target version when you call [`migration`](#method.migrate): `migration::<Data>()`.
    ///   That means you can't call `migration::<LegacyData>()` because `LegacyData` has version `1` and `Data` has version `2`.
    ///
    /// After call `migration::<Data>()` all data of the model `LegacyData` will be migrated to the model `Data`.
    ///
    /// Under the hood, when you call [`migration`](#method.migrate) `native_model` is used to convert the data from the old model to the new model
    /// using the `From` or `TryFrom` implementation for each to target the version defined when you call [`migration::<LastVersion>()`](#method.migrate).
    ///
    /// It's advisable to perform all migrations within a **single transaction** to ensure that all migrations are successfully completed.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
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
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<LegacyData>()?;
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///
    ///     let rw = db.rw_transaction()?;
    ///     rw.migrate::<Data>()?;
    ///     // Other migrations if needed..
    ///     rw.commit()
    /// }
    /// ```
    pub fn migrate<T: Input + Debug>(&self) -> Result<()> {
        self.internal.migrate::<T>()
    }
}
