use crate::table_definition::PrimaryTableDefinition;
use crate::Error::TableDefinitionNotFound;
use crate::Result;
use crate::{ReadOnlyTransaction, ReadableTable};
use std::collections::HashMap;

/// A collection of read-only tables. Only read operations available through the [`ReadableTable`](crate::ReadableTable) trait
/// are allowed.
pub struct ReadOnlyTables<'db, 'txn> {
    pub(crate) table_definitions: &'db HashMap<&'static str, PrimaryTableDefinition>,
    pub(crate) opened_read_only_tables:
        HashMap<&'static str, redb::ReadOnlyTable<'txn, &'static [u8], &'static [u8]>>,
}

impl<'db, 'txn> ReadableTable<'db, 'txn> for ReadOnlyTables<'db, 'txn> {
    type Table = redb::ReadOnlyTable<'txn, &'static [u8], &'static [u8]>;
    type Transaction<'x> = ReadOnlyTransaction<'db>;

    fn open_primary_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        table_name: &'static str,
    ) -> Result<()> {
        let table = self
            .table_definitions
            .get(table_name)
            .ok_or(TableDefinitionNotFound {
                table: table_name.to_string(),
            })?;
        if !self.opened_read_only_tables.contains_key(table_name) {
            let table = txn.txn.open_table(table.redb)?;
            self.opened_read_only_tables.insert(table_name, table);
        }
        Ok(())
    }

    fn open_secondary_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        primary_table_name: &'static str,
        secondary_table_name: &'static str,
    ) -> Result<()> {
        let primary_table =
            self.table_definitions
                .get(primary_table_name)
                .ok_or(TableDefinitionNotFound {
                    table: primary_table_name.to_string(),
                })?;
        let secondary_table = primary_table
            .secondary_tables
            .get(secondary_table_name)
            .ok_or(TableDefinitionNotFound {
                table: secondary_table_name.to_string(),
            })?;
        if !self
            .opened_read_only_tables
            .contains_key(secondary_table_name)
        {
            let table = txn.txn.open_table(secondary_table.rdb())?;
            self.opened_read_only_tables
                .insert(secondary_table_name, table);
        }
        Ok(())
    }

    fn get_table(&self, table_name: &'static str) -> Option<&Self::Table> {
        self.opened_read_only_tables.get(table_name)
    }
}
