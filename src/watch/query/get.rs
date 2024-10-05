use crate::db_type::{check_key_type, check_key_type_from_key_definition, KeyOptions, Result, ToInput, ToKey, ToKeyDefinition};
use crate::watch;
use crate::watch::query::internal;
use crate::watch::MpscReceiver;

/// Watch only one value.
pub struct WatchGet<'db, 'w> {
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchGet<'_, '_> {
    /// Watch the primary key.
    ///
    /// Returns a channel receiver and the watcher id.
    /// The watcher id can be used to unwatch the channel.
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
    ///     // Watch the primary key
    ///     let (_recv, _id) = db.watch().get().primary::<Data>(1u64)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn primary<T: ToInput>(
        &self,
        key: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let model = T::native_db_model();
        check_key_type(&model, &key)?;
        self.internal.watch_primary::<T>(key)
    }

    /// Watch the secondary key.
    ///
    /// Returns a channel receiver and the watcher id.
    /// The watcher id can be used to unwatch the channel.
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
    ///    name: String,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut models = Models::new();
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Watch the secondary key name
    ///     let (_recv, _id) = db.watch().get().secondary::<Data>(DataKey::name, "test")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
        key: impl ToKey,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        check_key_type_from_key_definition(&key_def.key_definition(), &key)?;
        self.internal.watch_secondary::<T>(&key_def, key)
    }
}
