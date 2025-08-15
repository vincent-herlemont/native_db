use crate::db_type::{
    Error, Key, KeyDefinition, KeyOptions, Output, Result, ToKey, ToKeyDefinition,
};
use crate::table_definition::PrimaryTableDefinition;
use crate::Model;
use redb::ReadableTableMetadata;
use redb::{ReadableMultimapTable, ReadableTable};
use std::collections::HashMap;

pub trait PrivateReadableTransaction<'db, 'txn> {
    type RedbPrimaryTable: ReadableTable<Key, &'static [u8]>;
    type RedbSecondaryTable: ReadableMultimapTable<Key, Key>;

    type RedbTransaction<'db_bis>
    where
        Self: 'db_bis;

    fn table_definitions(&self) -> &HashMap<String, PrimaryTableDefinition<'_>>;

    fn get_primary_table(&'txn self, model: &Model) -> Result<Self::RedbPrimaryTable>;

    fn get_secondary_table(
        &'txn self,
        model: &Model,
        secondary_key: &KeyDefinition<KeyOptions>,
    ) -> Result<Self::RedbSecondaryTable>;

    fn get_by_primary_key(&'txn self, model: Model, key: impl ToKey) -> Result<Option<Output>> {
        let table = self.get_primary_table(&model)?;
        let key = key.to_key();
        let item = table.get(key)?;
        Ok(item.map(|item| item.value().into()))
    }

    fn get_by_secondary_key(
        &'txn self,
        model: Model,
        key_def: impl ToKeyDefinition<KeyOptions>,
        key: impl ToKey,
    ) -> Result<Option<Output>> {
        let secondary_key = key_def.key_definition();
        // Provide a better error for the test of unicity of the secondary key
        model.check_secondary_options(&secondary_key, |options| options.unique)?;

        let table = self.get_secondary_table(&model, &secondary_key)?;

        let mut primary_keys = table.get(key.to_key())?;
        let primary_key = if let Some(primary_key) = primary_keys.next() {
            let primary_key = primary_key?;
            primary_key.value().to_owned()
        } else {
            return Ok(None);
        };

        Ok(Some(
            self.get_by_primary_key(model, primary_key)?
                .ok_or(Error::PrimaryKeyNotFound)?,
        ))
    }

    fn primary_len(&'txn self, model: Model) -> Result<u64> {
        let table = self.get_primary_table(&model)?;
        let result = table.len()?;
        Ok(result)
    }

    fn secondary_len(
        &'txn self,
        model: Model,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Result<u64> {
        let table = self.get_secondary_table(&model, &key_def.key_definition())?;
        let result = table.len()?;
        Ok(result)
    }
}
