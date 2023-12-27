use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::query::RGet;
use crate::transaction::query::RLen;
use crate::transaction::query::RScan;

pub struct RTransaction<'db> {
    pub(crate) internal: InternalRTransaction<'db>,
}

impl<'db> RTransaction<'db> {
    /// Get a value from the database.
    pub const fn get<'txn>(&'txn self) -> RGet<'db, 'txn> {
        RGet {
            internal: &self.internal,
        }
    }

    /// Get values from the database.
    pub const fn scan<'txn>(&'txn self) -> RScan<'db, 'txn> {
        RScan {
            internal: &self.internal,
        }
    }

    /// Get the number of values in the database.
    pub const fn len<'txn>(&'txn self) -> RLen<'db, 'txn> {
        RLen {
            internal: &self.internal,
        }
    }
}
