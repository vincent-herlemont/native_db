mod get;
mod internal;
mod scan;

pub use get::*;
pub(crate) use internal::*;
pub use scan::*;

pub struct Watch<'db> {
    pub(crate) internal: InternalWatch<'db>,
}

impl<'db> Watch<'db> {
    pub fn get<'w>(&'w self) -> WatchGet<'db, 'w> {
        WatchGet {
            internal: &self.internal,
        }
    }

    pub fn scan<'w>(&'w self) -> WatchScan<'db, 'w> {
        WatchScan {
            internal: &self.internal,
        }
    }
}
