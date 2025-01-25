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

impl RScan<'_, '_> {
    /// Get a values from the database by primary key.
    ///
    /// - [`all`](crate::transaction::query::PrimaryScan::all) - Scan all items.
    /// - [`start_with`](crate::transaction::query::PrimaryScan::start_with) - Scan items with a primary key starting with a key.
    /// - [`range`](crate::transaction::query::PrimaryScan::range) - Scan items with a primary key in a given range.
    pub fn primary<T: ToInput>(
        &self,
    ) -> Result<PrimaryScan<redb::ReadOnlyTable<Key, &'static [u8]>, T>> {
        let model = T::native_db_model();
        let table = self.internal.get_primary_table(&model)?;
        let out = PrimaryScan::new(table);
        Ok(out)
    }

    #[allow(clippy::type_complexity)]
    /// Get a values from the database by secondary key.
    ///
    /// - [`all`](crate::transaction::query::SecondaryScan::all) - Scan all items.
    /// - [`start_with`](crate::transaction::query::SecondaryScan::start_with) - Scan items with a secondary key starting with a key.
    /// - [`range`](crate::transaction::query::SecondaryScan::range) - Scan items with a secondary key in a given range.
    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<
        SecondaryScan<
            redb::ReadOnlyTable<Key, &'static [u8]>,
            redb::ReadOnlyMultimapTable<Key, Key>,
            T,
        >,
    > {
        let model = T::native_db_model();
        let primary_table = self.internal.get_primary_table(&model)?;
        let secondary_key = key_def.key_definition();
        let secondary_table = self.internal.get_secondary_table(&model, &secondary_key)?;
        let out = SecondaryScan::new(primary_table, secondary_table, key_def);
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
    /// Get a values from the database by primary key.
    ///
    /// - [`all`](crate::transaction::query::PrimaryScan::all) - Scan all items.
    /// - [`start_with`](crate::transaction::query::PrimaryScan::start_with) - Scan items with a primary key starting with a key.
    /// - [`range`](crate::transaction::query::PrimaryScan::range) - Scan items with a primary key in a given range.
    pub fn primary<T: ToInput>(
        &self,
    ) -> Result<PrimaryScan<redb::Table<'db, Key, &'static [u8]>, T>> {
        let model = T::native_db_model();
        let table = self.internal.get_primary_table(&model)?;
        let out = PrimaryScan::new(table);
        Ok(out)
    }

    #[allow(clippy::type_complexity)]
    /// Get a values from the database by secondary key.
    ///
    /// - [`all`](crate::transaction::query::PrimaryScan::all) - Scan all items.
    /// - [`start_with`](crate::transaction::query::PrimaryScan::start_with) - Scan items with a primary key starting with a key.
    /// - [`range`](crate::transaction::query::PrimaryScan::range) - Scan items with a primary key in a given range.
    pub fn secondary<T: ToInput>(
        &self,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<
        SecondaryScan<redb::Table<'db, Key, &'static [u8]>, redb::MultimapTable<'db, Key, Key>, T>,
    > {
        let model = T::native_db_model();
        let primary_table = self.internal.get_primary_table(&model)?;
        let secondary_key = key_def.key_definition();
        let secondary_table = self.internal.get_secondary_table(&model, &secondary_key)?;
        let out = SecondaryScan::new(primary_table, secondary_table, key_def);
        Ok(out)
    }
}
