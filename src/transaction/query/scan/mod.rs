mod primary_scan;
mod secondary_scan;

use crate::db_type::{Key, KeyOptions, Result, ToInput, ToKeyDefinition};
use crate::transaction::inner::GetInner;
pub use primary_scan::*;
pub use secondary_scan::*;

use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

/// Get values from the database.
pub struct RScan<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}
pub struct RwScan<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}
impl<'db, 'txn> ScanTrait<'db, 'txn> for RScan<'db, 'txn> {}
impl<'db, 'txn> ScanTrait<'db, 'txn> for RwScan<'db, 'txn> {}

pub trait ScanTrait<'db, 'txn>
where
    'db: 'txn,
    Self: GetInner<'db, 'txn>,
{
    fn primary<T: ToInput>(
        &'txn self,
    ) -> Result<
        PrimaryScan<<<Self as GetInner<'db, 'txn>>::Transaction as PrivateReadableTransaction<'db, 'txn>>::RedbPrimaryTable, T>
    >{
        let model = T::native_db_model();
        let transaction = self.inner();
        let table = PrivateReadableTransaction::get_primary_table(transaction, &model)?;
        let out = PrimaryScan::new(table);
        Ok(out)
    }

    #[allow(clippy::type_complexity)]
    fn secondary<T: ToInput>(
        &'txn self,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<
        SecondaryScan<
            <<Self as GetInner<'db, 'txn>>::Transaction as PrivateReadableTransaction<'db, 'txn>>::RedbPrimaryTable,
            <<Self as GetInner<'db, 'txn>>::Transaction as PrivateReadableTransaction<'db, 'txn>>::RedbSecondaryTable,
            T,
        >,
    >{
        let model = T::native_db_model();
        let transaction = self.inner();
        let primary_table = transaction.get_primary_table(&model)?;

        let secondary_key = key_def.key_definition();
        let secondary_table =
            PrivateReadableTransaction::get_secondary_table(transaction, &model, &secondary_key)?;
        let out = SecondaryScan::new(primary_table, secondary_table, key_def);
        Ok(out)
    }
}
