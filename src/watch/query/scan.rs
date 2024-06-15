use crate::db_type::{KeyDefinition, KeyOptions, Result, ToInput, ToKey, ToKeyDefinition};
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
    pub fn primary(&self) -> WatchScanPrimary {
        WatchScanPrimary {
            internal: &self.internal,
        }
    }

    /// Watch all values by secondary key.
    pub fn secondary(&self, key_def: impl ToKeyDefinition<KeyOptions>) -> WatchScanSecondary {
        WatchScanSecondary {
            key_def: key_def.key_definition(),
            internal: &self.internal,
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
    ///     // Watch all values
    ///     let (_recv, _id) = db.watch().scan().primary().all::<Data>()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn all<T: ToInput>(&self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_primary_all::<T>()
    }

    /// **TODO: needs to be implemented**
    pub fn range<'a>(
        &self,
        _range: impl RangeBounds<&'a [u8]> + 'a,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        todo!()
    }

    /// Watch all values starting with the given key.
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
    /// use native_model::{native_model, Model};
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
    pub fn all<'ws, T: ToInput>(&'ws self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_secondary_all::<T>(&self.key_def)
    }

    pub fn range<'a, 'ws>(
        &'ws self,
        _range: impl RangeBounds<&'a [u8]> + 'a,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        todo!()
    }

    /// Watch all values starting with the given key.
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
        self.internal
            .watch_secondary_start_with::<T>(&self.key_def, start_with)
    }
}
