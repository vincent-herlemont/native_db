use crate::db_type::{
    DatabaseInnerKeyValue, DatabaseKeyDefinition, DatabaseSecondaryKeyOptions, Error, Result,
};
use crate::table_definition::PrimaryTableDefinition;
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::Model;
use std::collections::HashMap;

pub struct InternalRTransaction<'db> {
    pub(crate) redb_transaction: redb::ReadTransaction<'db>,
    pub(crate) table_definitions: &'db HashMap<String, PrimaryTableDefinition<'db>>,
}

impl<'db, 'txn> PrivateReadableTransaction<'db, 'txn> for InternalRTransaction<'db>
where
    Self: 'txn,
    Self: 'db,
{
    type RedbPrimaryTable = redb::ReadOnlyTable<'txn, DatabaseInnerKeyValue, &'static [u8]>;
    type RedbSecondaryTable =
        redb::ReadOnlyTable<'txn, DatabaseInnerKeyValue, DatabaseInnerKeyValue>;

    type RedbTransaction<'db_bis> = redb::ReadTransaction<'db> where Self: 'db_bis;

    fn table_definitions(&self) -> &HashMap<String, PrimaryTableDefinition> {
        &self.table_definitions
    }

    fn get_primary_table(&'txn self, model: &Model) -> Result<Self::RedbPrimaryTable> {
        let table_definition = self
            .table_definitions()
            .get(model.primary_key.unique_table_name.as_str())
            .ok_or_else(|| Error::TableDefinitionNotFound {
                table: model.primary_key.unique_table_name.to_string(),
            })?;
        let table = self.redb_transaction.open_table(table_definition.redb)?;
        Ok(table)
    }

    fn get_secondary_table(
        &'txn self,
        model: &Model,
        secondary_key: &DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> Result<Self::RedbSecondaryTable> {
        let main_table_definition = self
            .table_definitions()
            .get(model.primary_key.unique_table_name.as_str())
            .ok_or_else(|| Error::TableDefinitionNotFound {
                table: model.primary_key.unique_table_name.to_string(),
            })?;
        let secondary_table_definition = main_table_definition
            .secondary_tables
            .get(&secondary_key)
            .ok_or_else(|| Error::TableDefinitionNotFound {
                table: secondary_key.unique_table_name.to_string(),
            })?;
        let table = self
            .redb_transaction
            .open_table(secondary_table_definition.redb)?;
        Ok(table)
    }
}
