use crate::db_type::{
    DatabaseKeyDefinition, DatabaseSecondaryKeyOptions, InnerKeyValue, Input, KeyDefinition, Result,
};
use crate::watch;
use crate::watch::query::internal;
use crate::watch::MpscReceiver;
use std::ops::RangeBounds;

pub struct WatchScan<'db, 'w> {
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchScan<'_, '_> {
    pub fn primary(&self) -> WatchScanPrimary {
        WatchScanPrimary {
            internal: &self.internal,
        }
    }

    pub fn secondary(
        &self,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> WatchScanSecondary {
        WatchScanSecondary {
            key_def: key_def.database_key(),
            internal: &self.internal,
        }
    }
}

pub struct WatchScanPrimary<'db, 'w> {
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchScanPrimary<'_, '_> {
    pub fn all<T: Input>(&self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_primary_all::<T>()
    }
    pub fn range<'a>(
        &self,
        _range: impl RangeBounds<&'a [u8]> + 'a,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        todo!()
    }

    pub fn start_with<T: Input>(
        &self,
        start_with: impl InnerKeyValue,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_primary_start_with::<T>(start_with)
    }
}

pub struct WatchScanSecondary<'db, 'w> {
    pub(crate) key_def: DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
    pub(crate) internal: &'w internal::InternalWatch<'db>,
}

impl WatchScanSecondary<'_, '_> {
    pub fn all<'ws, T: Input>(&'ws self) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal.watch_secondary_all::<T>(&self.key_def)
    }

    pub fn range<'a, 'ws>(
        &'ws self,
        _range: impl RangeBounds<&'a [u8]> + 'a,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        todo!()
    }

    pub fn start_with<T: Input>(
        &self,
        start_with: impl InnerKeyValue,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        self.internal
            .watch_secondary_start_with::<T>(&self.key_def, start_with)
    }
}
