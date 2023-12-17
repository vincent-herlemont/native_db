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
    pub fn get<'txn>(&'txn self) -> RwGet<'db, 'txn> {
        RwGet {
            internal: &self.internal,
        }
    }

    pub fn scan<'txn>(&'txn self) -> RwScan<'db, 'txn> {
        RwScan {
            internal: &self.internal,
        }
    }

    pub fn len<'txn>(&'txn self) -> RwLen<'db, 'txn> {
        RwLen {
            internal: &self.internal,
        }
    }

    pub fn drain<'txn>(&'txn self) -> RwDrain<'db, 'txn> {
        RwDrain {
            internal: &self.internal,
        }
    }
}

impl<'db, 'txn> RwTransaction<'db> {
    pub fn commit(self) -> Result<()> {
        self.internal.commit()?;
        // Send batch to watchers after commit succeeds
        let batch = self.batch.into_inner();
        watch::push_batch(Arc::clone(&self.watcher), batch)?;
        Ok(())
    }
}

impl<'db, 'txn> RwTransaction<'db> {
    pub fn insert<T: Input>(&self, item: T) -> Result<()> {
        let (watcher_request, binary_value) = self
            .internal
            .concrete_insert(T::native_db_model(), item.to_item())?;
        let event = Event::new_insert(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    pub fn remove<T: Input>(&self, item: T) -> Result<()> {
        let (watcher_request, binary_value) = self
            .internal
            .concrete_remove(T::native_db_model(), item.to_item())?;
        let event = Event::new_delete(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    pub fn update<T: Input>(&self, old_item: T, updated_item: T) -> Result<()> {
        let (watcher_request, old_binary_value, new_binary_value) = self.internal.concrete_update(
            T::native_db_model(),
            old_item.to_item(),
            updated_item.to_item(),
        )?;
        let event = Event::new_update(old_binary_value, new_binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    pub fn convert_all<OldType, NewType>(&self) -> Result<()>
    where
        OldType: Input + Clone,
        NewType: Input + From<OldType>,
    {
        let find_all_old: Vec<OldType> = self.scan().primary()?.iter().collect();
        for old in find_all_old {
            let new: NewType = old.clone().into();
            self.internal
                .concrete_insert(NewType::native_db_model(), new.to_item())?;
            self.internal
                .concrete_remove(OldType::native_db_model(), old.to_item())?;
        }
        Ok(())
    }

    /// Automatically migrate the data from the old schema to the new schema. **No matter the state of the database**,
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
