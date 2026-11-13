use crate::db_type::{KeyOptions, Result, ToInput, ToKeyDefinition};
use crate::transaction::inner::GetInner;
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

/// Get the number of values in the database.
pub struct RLen<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}
impl<'db, 'txn> LenTrait<'db, 'txn> for RLen<'db, 'txn> {}

pub struct RwLen<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}
impl<'db, 'txn> LenTrait<'db, 'txn> for RwLen<'db, 'txn> {}

pub trait LenTrait<'db, 'txn>: GetInner<'db, 'txn>
where
    'db: 'txn,
{
    fn primary<T: ToInput>(&'txn self) -> Result<u64> {
        let model = T::native_db_model();
        let result = self.inner().primary_len(model)?;
        Ok(result)
    }
    fn secondary<T: ToInput>(&'txn self, key_def: impl ToKeyDefinition<KeyOptions>) -> Result<u64> {
        let model = T::native_db_model();
        let result = self.inner().secondary_len(model, key_def)?;
        Ok(result)
    }
}
