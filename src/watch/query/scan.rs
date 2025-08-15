use crate::db_type::{
    check_key_type, check_key_type_from_key_definition, check_range_key_range_bounds,
    check_range_key_range_bounds_from_key_definition, KeyDefinition, KeyOptions, Result, ToInput,
    ToKey, ToKeyDefinition,
};
use crate::watch;
use crate::watch::query::internal;
use crate::watch::MpscReceiver;
use std::ops::RangeBounds;

/// Watch multiple values.
pub struct WatchScan<'db, 'w> {
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

/// Watch multiple values.
impl WatchScan<'_, '_> {
    /// Watch all values.
    ///
    /// - [`all`](crate::watch::query::WatchScanPrimary::all) - Watch all items.
    /// - [`start_with`](crate::watch::query::WatchScanPrimary::start_with) - Watch items with a primary key starting with a key.
    /// - [`range`](crate::watch::query::WatchScanPrimary::range) - Watch items with a primary key in a given range.
    pub fn primary(&self) -> WatchScanPrimary<'_, '_> {
        WatchScanPrimary {
            internal: self.internal,
        }
    }

    /// Watch all values by secondary key.
    ///
    /// - [`all`](crate::watch::query::WatchScanSecondary::all) - Watch items with a given secondary key.
    /// - [`start_with`](crate::watch::query::WatchScanSecondary::start_with) - Watch items with a secondary key starting with a key.
    /// - [`range`](crate::watch::query::WatchScanSecondary::range) - Watch items with a secondary key in a given range.
    pub fn secondary(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> WatchScanSecondary<'_, '_> {
        WatchScanSecondary {
            key_def: key_def.key_definition(),
            internal: self.internal,
        }
    }
}

/// Watch all values.
pub struct WatchScanPrimary<'db, 'w> {
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchScanPrimary<'_, '_> {
    /// Watch all values.
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
    ///     // Watch all values
    ///     let (_recv, _id) = db.watch().scan().primary().all::<Data>()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn all<T: ToInput>(&self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_primary_all::<T>()
    }

    /// Watch all values within a given range.
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
    ///     id: i32,
    ///     #[secondary_key]
    ///     name: String,
    ///     #[secondary_key]
    ///     age: i32,
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
    ///     // Watch all values by primary key between 1 and 10
    ///     let (_recv, _id) = db.watch().scan().primary().range::<Data, _>(1..=10)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn range<T: ToInput, R: RangeBounds<impl ToKey>>(
        &self,
        range: R,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let model = T::native_db_model();
        check_range_key_range_bounds(&model, &range)?;
        self.internal.watch_primary_range::<T, R>(range)
    }

    /// Watch all values starting with the given key.
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
    ///     // Watch all values starting with "test"
    ///     let (_recv, _id) = db.watch().scan().primary().start_with::<Data>("test")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with<T: ToInput>(
        &self,
        start_with: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let model = T::native_db_model();
        check_key_type(&model, &start_with)?;
        self.internal.watch_primary_start_with::<T>(start_with)
    }
}

/// Watch all values by secondary key.
pub struct WatchScanSecondary<'db, 'w> {
    pub(crate) key_def: KeyDefinition<KeyOptions>,
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchScanSecondary<'_, '_> {
    /// Watch all values by secondary key.
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
    ///     #[secondary_key]
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
    ///     // Watch all values by secondary key "name"
    ///     let (_recv, _id) = db.watch().scan().secondary(DataKey::name).all::<Data>()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn all<T: ToInput>(&self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_secondary_all::<T>(&self.key_def)
    }

    /// Watch all values within a given range.
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
    ///     #[secondary_key]
    ///     name: String,
    ///     #[secondary_key]
    ///     age: i32,
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
    ///     // Watch all values by secondary key "age" between 1 and 10
    ///     let (_recv, _id) = db.watch().scan().secondary(DataKey::age).range::<Data, _>(1..=10)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn range<T: ToInput, R: RangeBounds<impl ToKey>>(
        &self,
        range: R,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        check_range_key_range_bounds_from_key_definition(&self.key_def, &range)?;
        self.internal
            .watch_secondary_range::<T, R>(&self.key_def, range)
    }

    /// Watch all values starting with the given key.
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
    ///     #[secondary_key]
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
    ///     // Watch all values by secondary key "name" starting with "test"
    ///     let (_recv, _id) = db.watch().scan().secondary(DataKey::name).start_with::<Data>("test")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with<T: ToInput>(
        &self,
        start_with: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        check_key_type_from_key_definition(&self.key_def, &start_with)?;
        self.internal
            .watch_secondary_start_with::<T>(&self.key_def, start_with)
    }
}
