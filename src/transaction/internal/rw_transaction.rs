use crate::db_type::{Error, Input, Key, KeyDefinition, KeyEntry, KeyOptions, Output, Result};
use crate::table_definition::PrimaryTableDefinition;
use crate::transaction::internal::private_readable_transaction::PrivateReadableTransaction;
use crate::watch::WatcherRequest;
use crate::{db_type::ToInput, Model};
use redb::ReadableTable;
use redb::ReadableTableMetadata;
use redb::TableHandle;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

pub struct InternalRwTransaction<'db> {
    pub(crate) redb_transaction: redb::WriteTransaction,
    pub(crate) primary_table_definitions: &'db HashMap<String, PrimaryTableDefinition<'db>>,
}

impl<'db, 'txn> PrivateReadableTransaction<'db, 'txn> for InternalRwTransaction<'db>
where
    Self: 'txn,
    Self: 'db,
{
    type RedbPrimaryTable = redb::Table<'txn, Key, &'static [u8]>;
    type RedbSecondaryTable = redb::Table<'txn, Key, Key>;

    type RedbTransaction<'db_bis> = redb::WriteTransaction where Self: 'db_bis;

    fn table_definitions(&self) -> &HashMap<String, PrimaryTableDefinition> {
        &self.primary_table_definitions
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
        secondary_key: &KeyDefinition<KeyOptions>,
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

impl<'db> InternalRwTransaction<'db> {
    pub(crate) fn commit(self) -> Result<()> {
        self.redb_transaction.commit()?;
        Ok(())
    }

    pub(crate) fn concrete_insert(
        &self,
        model: Model,
        item: Input,
    ) -> Result<(WatcherRequest, Output)> {
        let already_exists;
        {
            let mut table = self.get_primary_table(&model)?;
            already_exists = table
                .insert(&item.primary_key, item.value.as_slice())?
                .is_some();
        }

        for (secondary_key_def, _value) in &item.secondary_keys {
            let mut secondary_table = self.get_secondary_table(&model, secondary_key_def)?;
            let result = match item.secondary_key_value(secondary_key_def)? {
                KeyEntry::Default(value) => secondary_table.insert(value, &item.primary_key)?,
                KeyEntry::Optional(value) => {
                    if let Some(value) = value {
                        secondary_table.insert(value, &item.primary_key)?
                    } else {
                        None
                    }
                }
            };
            if result.is_some() && !already_exists {
                return Err(Error::DuplicateKey {
                    key_name: secondary_key_def.unique_table_name.to_string(),
                }
                .into());
            }
        }

        Ok((
            WatcherRequest::new(
                model.primary_key.unique_table_name.clone(),
                item.primary_key,
                item.secondary_keys,
            ),
            Output(item.value),
        ))
    }

    pub(crate) fn concrete_remove(
        &self,
        model: Model,
        item: Input,
    ) -> Result<(WatcherRequest, Output)> {
        let keys = &item.secondary_keys;
        {
            let mut table = self.get_primary_table(&model)?;
            table.remove(&item.primary_key)?;
        }

        for (secondary_key_def, _value) in keys {
            let mut secondary_table = self.get_secondary_table(&model, secondary_key_def)?;
            match &item.secondary_key_value(secondary_key_def)? {
                KeyEntry::Default(value) => {
                    secondary_table.remove(value)?;
                }
                KeyEntry::Optional(value) => {
                    if let Some(value) = value {
                        secondary_table.remove(value)?;
                    }
                }
            }
        }

        Ok((
            WatcherRequest::new(
                model.primary_key.unique_table_name.clone(),
                item.primary_key,
                item.secondary_keys,
            ),
            Output(item.value),
        ))
    }

    pub(crate) fn concrete_update(
        &self,
        model: Model,
        old_item: Input,
        updated_item: Input,
    ) -> Result<(WatcherRequest, Output, Output)> {
        let (_, old_binary_value) = self.concrete_remove(model.clone(), old_item)?;
        let (watcher_request, new_binary_value) = self.concrete_insert(model, updated_item)?;
        Ok((watcher_request, old_binary_value, new_binary_value))
    }

    pub(crate) fn concrete_primary_drain<'a>(&self, model: Model) -> Result<Vec<Output>> {
        let mut items = vec![];
        let mut key_items = HashSet::new();

        let mut primary_table = self.get_primary_table(&model)?;
        // Drain primary table
        let drain = primary_table.extract_from_if::<Key, _>(.., |_, _| true)?;
        for result in drain {
            let (primary_key, value) = result?;
            // TODO: we should delay to an drain scan
            let binary_value = Output(value.value().to_vec());
            key_items.insert(primary_key.value().to_owned());
            items.push(binary_value);
        }

        let secondary_table_names: Vec<&KeyDefinition<KeyOptions>> = self
            .primary_table_definitions
            .get(model.primary_key.unique_table_name.as_str())
            .ok_or(Error::TableDefinitionNotFound {
                table: model.primary_key.unique_table_name.to_string(),
            })?
            .secondary_tables
            .iter()
            .map(|(key, _)| key)
            .collect();

        // Drain secondary tables
        for secondary_table_name in secondary_table_names {
            let mut secondary_table = self.get_secondary_table(&model, secondary_table_name)?;

            // Detect secondary keys to delete
            let mut secondary_keys_to_delete = vec![];
            let mut number_detected_key_to_delete = key_items.len();
            for secondary_items in secondary_table.iter()? {
                // Ta avoid to iter on all secondary keys if we have already detected all keys to delete
                if number_detected_key_to_delete == 0 {
                    break;
                }
                let (secondary_key, primary_key) = secondary_items?;
                if key_items.contains(&primary_key.value().to_owned()) {
                    // TODO remove owned
                    secondary_keys_to_delete.push(secondary_key.value().to_owned());
                    number_detected_key_to_delete -= 1;
                }
            }

            // Delete secondary keys
            for secondary_key in secondary_keys_to_delete {
                secondary_table.remove(secondary_key)?;
            }
        }

        Ok(items)
    }

    pub fn migrate<T: ToInput + Debug>(&self) -> Result<()> {
        let new_table_definition = self
            .primary_table_definitions
            .get(T::native_db_model().primary_key.unique_table_name.as_str())
            .unwrap();
        if new_table_definition
            .native_model_options
            .native_model_legacy
        {
            return Err(Error::MigrateLegacyModel(
                T::native_db_model()
                    .primary_key
                    .unique_table_name
                    .to_string(),
            ));
        }

        let mut old_table_definition: Option<&PrimaryTableDefinition> = None;
        let model_table_definitions = self.primary_table_definitions.values().filter(|t| {
            t.native_model_options.native_model_id
                == new_table_definition.native_model_options.native_model_id
        });

        // Find the old model table with data
        for new_primary_table_definition in model_table_definitions {
            // check if table exists, if the table does not exist continue
            if self
                .redb_transaction
                .list_tables()?
                .find(|table| table.name() == new_primary_table_definition.redb.name())
                .is_none()
            {
                continue;
            }

            let table = self
                .redb_transaction
                .open_table(new_primary_table_definition.redb.clone())?;
            let len = table.len()?;
            if len > 0 && old_table_definition.is_some() {
                panic!(
                    "Impossible to migrate the table {} because multiple old tables with data exist: {}, {}",
                    T::native_db_model().primary_key.unique_table_name,
                    new_primary_table_definition.redb.name(),
                    old_table_definition.unwrap().redb.name()
                );
            } else if table.len()? > 0 {
                old_table_definition = Some(new_primary_table_definition);
            }
        }

        let old_table_definition = if let Some(old_table_definition) = old_table_definition {
            old_table_definition
        } else {
            // Nothing to migrate
            return Ok(());
        };

        // If the old table is the same as the new table, nothing to migrate
        if old_table_definition.redb.name()
            == T::native_db_model().primary_key.unique_table_name.as_str()
        {
            // Nothing to migrate
            return Ok(());
        }

        // List all data from the old table
        for old_data in self.concrete_primary_drain(old_table_definition.model.clone())? {
            let (decoded_item, _) = native_model::decode::<T>(old_data.0).unwrap();
            let decoded_item = decoded_item.native_db_input()?;
            self.concrete_insert(T::native_db_model(), decoded_item)?;
        }

        Ok(())
    }
}
