use crate::transaction::{
    internal::{
        private_readable_transaction::PrivateReadableTransaction,
        r_transaction::InternalRTransaction, rw_transaction::InternalRwTransaction,
    },
    query::{RGet, RLen, RScan, RwGet, RwLen, RwScan},
};

pub trait GetInner<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction: PrivateReadableTransaction<'db, 'txn>;
    fn inner(&self) -> &Self::Transaction;
}
impl<'db, 'txn> GetInner<'db, 'txn> for RGet<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction = InternalRTransaction<'db>;

    fn inner(&self) -> &Self::Transaction {
        &self.internal
    }
}
impl<'db, 'txn> GetInner<'db, 'txn> for RwGet<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction = InternalRwTransaction<'db>;

    fn inner(&self) -> &Self::Transaction {
        &self.internal
    }
}

impl<'db, 'txn> GetInner<'db, 'txn> for RScan<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction = InternalRTransaction<'db>;

    fn inner(&self) -> &Self::Transaction {
        &self.internal
    }
}
impl<'db, 'txn> GetInner<'db, 'txn> for RwScan<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction = InternalRwTransaction<'db>;

    fn inner(&self) -> &Self::Transaction {
        &self.internal
    }
}
impl<'db, 'txn> GetInner<'db, 'txn> for RLen<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction = InternalRTransaction<'db>;

    fn inner(&self) -> &Self::Transaction {
        &self.internal
    }
}
impl<'db, 'txn> GetInner<'db, 'txn> for RwLen<'db, 'txn>
where
    'db: 'txn,
{
    type Transaction = InternalRwTransaction<'db>;

    fn inner(&self) -> &Self::Transaction {
        &self.internal
    }
}
