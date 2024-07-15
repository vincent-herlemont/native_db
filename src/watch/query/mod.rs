mod get;
mod internal;
mod scan;

pub use get::*;
pub(crate) use internal::*;
pub use scan::*;

/// Watch queries.
pub struct Watch<'db> {
    pub(crate) internal: InternalWatch<'db>,
}

impl<'db> Watch<'db> {
    /// Watch only one value.
    ///
    /// - [`primary`](crate::watch::query::WatchGet::primary) - Watch a item by primary key.
    /// - [`secondary`](crate::watch::query::WatchGet::secondary) - Watch a item by secondary key.
    pub fn get<'w>(&'w self) -> WatchGet<'db, 'w> {
        WatchGet {
            internal: &self.internal,
        }
    }
    /// Watch multiple values.
    ///
    /// - [`primary`](crate::watch::query::WatchScan::primary) - Watch items by primary key.
    /// - [`secondary`](crate::watch::query::WatchScan::secondary) - Watch items by secondary key.
    pub fn scan<'w>(&'w self) -> WatchScan<'db, 'w> {
        WatchScan {
            internal: &self.internal,
        }
    }
}
