mod get;
mod internal;
mod scan;

pub use get::*;
pub(crate) use internal::*;
pub use scan::*;

/// Watch queries.
///
/// **Memory Warning**: Each active watcher consumes memory until explicitly removed. The watch
/// system stores all watchers in a HashMap and keeps channel senders alive. With the `tokio` 
/// feature, unbounded channels are used which can accumulate events if not consumed.
/// 
/// Best practices:
/// - Always call [`Database::unwatch()`](crate::Database::unwatch) when done watching
/// - Consume events promptly to prevent channel backlog
/// - Consider implementing a cleanup strategy for long-running applications
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
