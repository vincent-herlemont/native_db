use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::query::RGet;
use crate::transaction::query::RLen;
use crate::transaction::query::RScan;

pub struct RTransaction<'db> {
    pub(crate) internal: InternalRTransaction<'db>,
}

impl<'db> RTransaction<'db> {
    /// Get a value from the database.
    /// 
    /// - [`primary`](crate::transaction::query::RGet::primary) - Get a item by primary key.
    /// - [`secondary`](crate::transaction::query::RGet::secondary) - Get a item by secondary key.
    pub fn get<'txn>(&'txn self) -> RGet<'db, 'txn> {
        RGet {
            internal: &self.internal,
        }
    }

    /// Get values from the database.
    /// 
    /// - [`primary`](crate::transaction::query::RScan::primary) - Scan items by primary key.
    /// - [`secondary`](crate::transaction::query::RScan::secondary) - Scan items by secondary key.
    pub fn scan<'txn>(&'txn self) -> RScan<'db, 'txn> {
        RScan {
            internal: &self.internal,
        }
    }

    /// Get the number of values in the database.
    /// 
    /// - [`primary`](crate::transaction::query::RLen::primary) - Get the number of items by primary key.
    /// - [`secondary`](crate::transaction::query::RLen::secondary) - Get the number of items by secondary key.
    pub fn len<'txn>(&'txn self) -> RLen<'db, 'txn> {
        RLen {
            internal: &self.internal,
        }
    }
}
