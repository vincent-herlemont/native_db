use crate::transaction::inner::GetInner;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::query::GetTrait;
use crate::transaction::query::LenTrait;
use crate::transaction::query::RGet;
use crate::transaction::query::RLen;
use crate::transaction::query::RScan;
use crate::transaction::query::ScanTrait;

pub struct RTransaction<'db> {
    pub(crate) internal: InternalRTransaction<'db>,
}

pub trait ReadTransactionTrait<'db, 'txn>
where
    'db: 'txn,
{
    type Get: GetTrait<'db, 'txn>;
    type Scan: ScanTrait<'db, 'txn>;
    type Len: LenTrait<'db, 'txn>;

    fn get(&'txn self) -> Self::Get;
    fn scan(&'txn self) -> Self::Scan;
    fn len(&'txn self) -> Self::Len;
}
impl<'db, 'txn> ReadTransactionTrait<'db, 'txn> for RTransaction<'db>
where
    'db: 'txn,
{
    type Get = RGet<'db, 'txn>;

    type Scan = RScan<'db, 'txn>;

    type Len = RLen<'db, 'txn>;
    fn get(&'txn self) -> Self::Get {
        RGet {
            internal: &self.internal,
        }
    }

    fn scan(&'txn self) -> Self::Scan {
        RScan {
            internal: &self.internal,
        }
    }

    fn len(&'txn self) -> Self::Len {
        RLen {
            internal: &self.internal,
        }
    }
}
