mod primary_scan;
mod secondary_scan;

use crate::db_type::{Key, KeyOptions, Result, ToInput, ToKeyDefinition};
pub use primary_scan::*;
pub use secondary_scan::*;

use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::transaction::internal::r_transaction::InternalRTransaction;
use crate::transaction::internal::rw_transaction::InternalRwTransaction;

/// Get values from the database.
pub struct RScan<'db, 'txn> {
    pub(crate) internal: &'txn InternalRTransaction<'db>,
}

impl<'txn> RScan<'_, 'txn> {
    /// Get a values from the database by primary key.
    pub fn primary<T: ToInput>(
        &self,
    ) -> Result<PrimaryScan<redb::ReadOnlyTable<Key, &'static [u8]>, T>> {
        let model = T::native_db_model();
        let table = self.internal.get_primary_table(&model)?;
        let out = PrimaryScan::new(table);
        Ok(out)
    }

    /// Get a values from the database by secondary key.
    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<
        SecondaryScan<redb::ReadOnlyTable<Key, &'static [u8]>, redb::ReadOnlyTable<Key, Key>, T>,
    > {
        let model = T::native_db_model();
        let primary_table = self.internal.get_primary_table(&model)?;
        let secondary_key = key_def.key_definition();
        let secondary_table = self.internal.get_secondary_table(&model, &secondary_key)?;
        let out = SecondaryScan::new(primary_table, secondary_table);
        Ok(out)
    }
}

pub struct RwScan<'db, 'txn> {
    pub(crate) internal: &'txn InternalRwTransaction<'db>,
}

impl<'db, 'txn> RwScan<'db, 'txn>
where
    'txn: 'db,
{
    pub fn primary<T: ToInput>(
        &self,
    ) -> Result<PrimaryScan<redb::Table<'db, Key, &'static [u8]>, T>> {
        let model = T::native_db_model();
        let table = self.internal.get_primary_table(&model)?;
        let out = PrimaryScan::new(table);
        Ok(out)
    }

    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<SecondaryScan<redb::Table<'db, Key, &'static [u8]>, redb::Table<'db, Key, Key>, T>>
    {
        let model = T::native_db_model();
        let primary_table = self.internal.get_primary_table(&model)?;
        let secondary_key = key_def.key_definition();
        let secondary_table = self.internal.get_secondary_table(&model, &secondary_key)?;
        let out = SecondaryScan::new(primary_table, secondary_table);
        Ok(out)
    }
}
