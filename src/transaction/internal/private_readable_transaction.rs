use crate::db_type::{
    DatabaseInnerKeyValue, DatabaseKeyDefinition, DatabaseOutputValue, DatabaseSecondaryKeyOptions,
    Error, InnerKeyValue, KeyDefinition, Result,
};
use crate::table_definition::PrimaryTableDefinition;
use crate::DatabaseModel;
use redb::ReadableTable;
use std::collections::HashMap;

pub trait PrivateReadableTransaction<'db, 'txn> {
    type RedbPrimaryTable: ReadableTable<DatabaseInnerKeyValue, &'static [u8]>;
    type RedbSecondaryTable: ReadableTable<DatabaseInnerKeyValue, DatabaseInnerKeyValue>;

    type RedbTransaction<'db_bis>
    where
        Self: 'db_bis;

    fn table_definitions(&self) -> &HashMap<String, PrimaryTableDefinition>;

    fn get_primary_table(&'txn self, model: &DatabaseModel) -> Result<Self::RedbPrimaryTable>;

    fn get_secondary_table(
        &'txn self,
        model: &DatabaseModel,
        secondary_key: &DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> Result<Self::RedbSecondaryTable>;

    fn get_by_primary_key(
        &'txn self,
        model: DatabaseModel,
        key: impl InnerKeyValue,
    ) -> Result<Option<DatabaseOutputValue>> {
        let table = self.get_primary_table(&model)?;
        let key = key.database_inner_key_value();
        let item = table.get(key)?;
        Ok(item.map(|item| item.value().into()))
    }

    fn get_by_secondary_key(
        &'txn self,
        model: DatabaseModel,
        key_def: impl KeyDefinition<DatabaseSecondaryKeyOptions>,
        key: impl InnerKeyValue,
    ) -> Result<Option<DatabaseOutputValue>> {
        let secondary_key = key_def.database_key();
        // Provide a better error for the test of unicity of the secondary key
        model.check_secondary_options(&secondary_key, |options| options.unique == true)?;

        let table = self.get_secondary_table(&model, &secondary_key)?;
        let value = table.get(key.database_inner_key_value())?;
        let primary_key = if let Some(value) = value {
            value.value().to_owned()
        } else {
            return Ok(None);
        };

        Ok(Some(
            self.get_by_primary_key(model, primary_key)?
                .ok_or(Error::PrimaryKeyNotFound)?,
        ))
    }

    fn primary_len(&'txn self, model: DatabaseModel) -> Result<u64> {
        let table = self.get_primary_table(&model)?;
        let result = table.len()?;
        Ok(result)
    }
}
