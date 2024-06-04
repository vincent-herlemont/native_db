use crate::db_type::inner_key_value_redb1::DatabaseInnerKeyValue as Redb1DatabaseInnerKeyValue;
use crate::db_type::Key as Redb2DatabaseInnerKeyValue;
use crate::db_type::Result;
use crate::table_definition::{
    RedbPrimaryTableDefinition as Redb2PrimaryTableDefinition,
    RedbSecondaryTableDefinition as Redb2SecondaryTableDefinition,
};
use crate::{DatabaseConfiguration, ModelBuilder};
use redb as redb2;
use redb1;
use std::collections::HashMap;
use std::path::Path;

pub(crate) type Redb1PrimaryTableDefinition<'a> =
    redb1::TableDefinition<'a, Redb1DatabaseInnerKeyValue, &'static [u8]>;
pub(crate) type Redb1SecondaryTableDefinition<'a> =
    redb1::TableDefinition<'a, Redb1DatabaseInnerKeyValue, Redb1DatabaseInnerKeyValue>;

fn upgrade_primary_table(
    table_name: &str,
    db1: &redb1::Database,
    db2: &redb2::Database,
) -> Result<()> {
    let redb1_primary_table_definition: Redb1PrimaryTableDefinition =
        redb1::TableDefinition::new(table_name);
    let redb2_primary_table_definition: Redb2PrimaryTableDefinition =
        redb2::TableDefinition::new(table_name);

    let redb1_read_txn: redb1::ReadTransaction = db1.begin_read()?;
    let redb2_write_txn = db2.begin_write()?;

    {
        let redb1_table = redb1_read_txn
            .open_table(redb1_primary_table_definition)
            .unwrap();
        let mut redb2_table = redb2_write_txn.open_table(redb2_primary_table_definition)?;

        use redb1::ReadableTable;
        for r in redb1_table.iter().unwrap() {
            let (key, value) = r.unwrap();
            let key = Redb2DatabaseInnerKeyValue::from(key.value());
            redb2_table.insert(key, value.value()).unwrap();
        }
    }

    redb2_write_txn.commit()?;

    Ok(())
}

fn upgrade_secondary_table(
    table_name: &str,
    db1: &redb1::Database,
    db2: &redb2::Database,
) -> Result<()> {
    let redb1_primary_table_definition: Redb1SecondaryTableDefinition =
        redb1::TableDefinition::new(table_name);
    let redb2_primary_table_definition: Redb2SecondaryTableDefinition =
        redb2::TableDefinition::new(table_name);

    let redb1_read_txn: redb1::ReadTransaction = db1.begin_read()?;
    let redb2_write_txn = db2.begin_write()?;

    {
        let redb1_table = redb1_read_txn
            .open_table(redb1_primary_table_definition)
            .unwrap();
        let mut redb2_table = redb2_write_txn.open_table(redb2_primary_table_definition)?;

        use redb1::ReadableTable;
        for r in redb1_table.iter().unwrap() {
            let (key, value) = r.unwrap();
            let key = Redb2DatabaseInnerKeyValue::from(key.value());
            let value = Redb2DatabaseInnerKeyValue::from(value.value());
            redb2_table.insert(key, value).unwrap();
        }
    }

    redb2_write_txn.commit()?;

    Ok(())
}

pub(crate) fn upgrade_redb1_to_redb2(
    database_configuration: &DatabaseConfiguration,
    path: impl AsRef<Path>,
    model_builder: &HashMap<String, ModelBuilder>,
) -> Result<()> {
    let redb1_builder = database_configuration.redb1_new_rdb1_builder();
    let redb2_builder = database_configuration.new_rdb_builder();

    let redb1_path = path.as_ref().to_path_buf();
    let redb2_path = redb1_path.with_file_name(format!(
        "{}_redb2",
        redb1_path.file_name().unwrap().to_str().unwrap()
    ));

    let db1 = redb1_builder.open(&redb1_path)?;
    let mut db2 = redb2_builder.create(&redb2_path)?;

    for (_, model_builder) in model_builder {
        upgrade_primary_table(
            model_builder.model.primary_key.unique_table_name.as_str(),
            &db1,
            &db2,
        )?;

        for secondary_key in model_builder.model.secondary_keys.iter() {
            let secondary_table_name = secondary_key.unique_table_name.as_str();
            upgrade_secondary_table(secondary_table_name, &db1, &db2)?;
        }
    }

    db2.compact()?;
    drop(db2);
    drop(db1);

    std::fs::remove_file(&redb1_path)?;
    std::fs::rename(&redb2_path, &redb1_path)?;

    Ok(())
}
