use std::collections::HashMap;

use crate::db_type::Result;
use crate::table_definition::RedbSecondaryTableDefinition;
use crate::{database_instance, Key, ModelBuilder, ToKey};
use redb::ReadableMultimapTable;

pub(crate) type OldSecondaryTableDefinition<'a> = redb::TableDefinition<'a, Key, Key>;

pub(crate) fn upgrade_secondary_index_table_multimap(
    database_instance: &database_instance::DatabaseInstance,
    model_builder: &HashMap<String, ModelBuilder>,
) -> Result<()> {
    // List secondary index tables
    for model_builder in model_builder.values() {
        for secondary_key in model_builder.model.secondary_keys.iter() {
            let secondary_table_name = secondary_key.unique_table_name.as_str();
            let secondary_table_name_tmp = secondary_table_name.to_string() + "_tmp";

            let old_table_definition: OldSecondaryTableDefinition =
                redb::TableDefinition::new(secondary_table_name);
            let tmp_table_definition: RedbSecondaryTableDefinition =
                redb::MultimapTableDefinition::new(&secondary_table_name_tmp);
            let new_table_definition: RedbSecondaryTableDefinition =
                redb::MultimapTableDefinition::new(secondary_table_name);

            let db = database_instance.redb_database()?;
            // Drain all data from the old table to the tmp table
            let rw = db.begin_write()?;
            {
                let mut table = if let Ok(table) = rw.open_table(old_table_definition) {
                    table
                } else {
                    continue;
                };
                // Read an insert in the tmp table
                let mut tmp_table = rw.open_multimap_table(tmp_table_definition)?;
                loop {
                    let result = if let Some(result) = table.pop_first()? {
                        result
                    } else {
                        break;
                    };
                    let (key, value) = result;

                    let key_combination = key.value();
                    let key_combinaison = key_combination.as_slice();
                    let primary_key = value.value();
                    let primary_key = primary_key.as_slice();
                    // Secondary key = primary key trim end of primary key
                    let secondary_key =
                        key_combinaison[0..key_combinaison.len() - primary_key.len()].to_vec();

                    tmp_table.insert(secondary_key.to_key(), primary_key.to_key())?;
                }
            }
            // Remove the old table
            rw.delete_table(old_table_definition)?;
            rw.commit()?;

            // Drain all data from the tmp table to the new table
            let rw = db.begin_write()?;
            {
                let tmp_table = rw.open_multimap_table(tmp_table_definition)?;
                let mut table = rw.open_multimap_table(new_table_definition)?;
                for result in tmp_table.iter()? {
                    let (secondary_key, primary_keys) = result?;
                    for primary_key in primary_keys {
                        let primary_key = primary_key?;
                        table.insert(secondary_key.value(), primary_key.value())?;
                    }
                }
            }
            // Remove the tmp table
            rw.delete_multimap_table(tmp_table_definition)?;
            rw.commit()?;
        }
    }

    Ok(())
}
