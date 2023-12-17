use crate::db_type::{DatabaseSecondaryKeyOptions, InnerKeyValue, Input, KeyDefinition, Result};
use crate::watch;
use crate::watch::query::internal;
use crate::watch::MpscReceiver;

pub struct WatchGet<'db, 'w> {
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchGet<'_, '_> {
    pub fn primary<T: Input>(
        &self,
        key: impl InnerKeyValue,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_primary::<T>(key)
    }

    pub fn secondary<T: Input>(
        &self,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
        key: impl InnerKeyValue,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_secondary::<T>(&key_def, key)
    }
}
