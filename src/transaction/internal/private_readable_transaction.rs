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

    fn primary_len_visible<T: crate::db_type::ToInput>(
        &'txn self,
        model: Model,
        observed_at: Option<u64>,
    ) -> Result<u64> {
        if model.expiry.expiry.is_none() || observed_at.is_none() {
            return self.primary_len(model);
        }
        let table = self.get_primary_table(&model)?;
        let mut visible = 0u64;
        for entry in table.range::<Key>(..)? {
            let (_, value) = entry?;
            if let Some(Ok(item)) = crate::db_type::unwrap_item::<T>(Some(value)) {
                if crate::expiry::visible_at(&model, &item, observed_at) {
                    visible += 1;
                }
            }
        }
        Ok(visible)
    }

    fn secondary_len_visible<T: crate::db_type::ToInput>(
        &'txn self,
        model: Model,
        key_def: impl ToKeyDefinition<KeyOptions>,
        observed_at: Option<u64>,
    ) -> Result<u64> {
        if model.expiry.expiry.is_none() || observed_at.is_none() {
            return self.secondary_len(model, key_def);
        }
        let primary_table = self.get_primary_table(&model)?;
        let secondary_table = self.get_secondary_table(&model, &key_def.key_definition())?;
        let mut visible = 0u64;
        for entry in secondary_table.iter()? {
            let (_, primary_keys) = entry?;
            for primary_key in primary_keys {
                let primary_key = primary_key?;
                let value = primary_table.get(primary_key.value())?;
                if let Some(Ok(item)) = crate::db_type::unwrap_item::<T>(value) {
                    if crate::expiry::visible_at(&model, &item, observed_at) {
                        visible += 1;
                    }
                }
            }
        }
        Ok(visible)
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
