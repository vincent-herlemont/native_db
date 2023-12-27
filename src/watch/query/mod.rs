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
    pub const fn get<'w>(&'w self) -> WatchGet<'db, 'w> {
        WatchGet {
            internal: &self.internal,
        }
    }
    /// Watch multiple values.
    pub const fn scan<'w>(&'w self) -> WatchScan<'db, 'w> {
        WatchScan {
            internal: &self.internal,
        }
    }
}
