use crate::item::BinaryValue;
use crate::watch::WatcherRequest;
use crate::{Error, Item, Schema, Tables};
use crate::{ReadableTable, Result, Transaction};
use std::collections::{HashMap, HashSet};
use std::ops::{Bound, RangeBounds};

impl<'db, 'txn> Tables<'db, 'txn> {
    pub(crate) fn internal_primary_drain<'a>(
        &mut self,
        txn: &'txn Transaction<'db>,
        primary_table_name: &'static str,
        range_value: impl RangeBounds<&'a [u8]> + 'a + Copy,
    ) -> Result<Vec<BinaryValue>> {
        let mut items = vec![];
        let mut key_items = HashSet::new();
        {
            self.open_primary_table(txn, primary_table_name)?;
            let primary_table = self.opened_tables.get_mut(primary_table_name).unwrap();
            // Drain primary table
            let drain = primary_table.drain::<&'_ [u8]>(range_value)?;
            for result in drain {
                let (primary_key, value) = result?;
                // TODO: we should delay to an drain iterator
                let binary_value = BinaryValue(value.value().to_vec());
                key_items.insert(primary_key.value().to_vec());
                items.push(binary_value);
            }
        }

        let secondary_table_names: Vec<&str> = self
            .table_definitions
            .get(primary_table_name)
            .ok_or(Error::TableDefinitionNotFound {
                table: primary_table_name.to_string(),
            })?
            .secondary_tables
            .iter()
            .map(|(name, _)| *name)
            .collect();

        // Drain secondary tables
        for secondary_table_name in secondary_table_names {
            self.open_secondary_table(txn, primary_table_name, secondary_table_name)?;
            use redb::ReadableTable;
            let secondary_table = self.opened_tables.get_mut(secondary_table_name).unwrap();

            // Detect secondary keys to delete
            let mut secondary_keys_to_delete = vec![];
            let mut number_detected_key_to_delete = key_items.len();
            for secondary_items in secondary_table.iter()? {
                // Ta avoid to iter on all secondary keys if we have already detected all keys to delete
                if number_detected_key_to_delete == 0 {
                    break;
                }
                let (secondary_key, primary_key) = secondary_items?;
                if key_items.contains(primary_key.value()) {
                    secondary_keys_to_delete.push(secondary_key.value().to_vec());
                    number_detected_key_to_delete -= 1;
                }
            }

            // Delete secondary keys
            for secondary_key in secondary_keys_to_delete {
                secondary_table.remove(secondary_key.as_slice())?;
            }
        }

        Ok(items)
    }

    pub(crate) fn internal_insert(
        &mut self,
        txn: &'txn Transaction<'db>,
        schema: Schema,
        item: Item,
    ) -> Result<(WatcherRequest, BinaryValue)> {
        let already_exists;
        {
            self.open_primary_table(txn, schema.table_name)?;
            let table = self.opened_tables.get_mut(schema.table_name).unwrap();
            already_exists = table
                .insert(item.primary_key.as_slice(), item.value.as_slice())?
                .is_some();
        }

        for (secondary_table_name, key) in &item.secondary_keys {
            self.open_secondary_table(txn, schema.table_name, secondary_table_name)?;
            let secondary_table = self.opened_tables.get_mut(secondary_table_name).unwrap();
            let result = secondary_table.insert(key.as_slice(), item.primary_key.as_slice())?;
            if result.is_some() && !already_exists {
                return Err(crate::Error::DuplicateKey {
                    key_name: secondary_table_name,
                }
                .into());
            }
        }

        Ok((
            WatcherRequest::new(schema.table_name, item.primary_key, item.secondary_keys),
            BinaryValue(item.value),
        ))
    }
}
