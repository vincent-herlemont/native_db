use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::query::RGet;
use crate::transaction::query::RLen;
use crate::transaction::query::RScan;

pub struct RTransaction<'db> {
    pub(crate) internal: InternalRTransaction<'db>,
}

impl<'db> RTransaction<'db> {
    pub fn get<'txn>(&'txn self) -> RGet<'db, 'txn> {
        RGet {
            internal: &self.internal,
        }
    }

    pub fn scan<'txn>(&'txn self) -> RScan<'db, 'txn> {
        RScan {
            internal: &self.internal,
        }
    }

    pub fn len<'txn>(&'txn self) -> RLen<'db, 'txn> {
        RLen {
            internal: &self.internal,
        }
    }
}
