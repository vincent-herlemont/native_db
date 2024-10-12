#[cfg(feature = "redb1")]
mod redb1_to_redb2;
#[cfg(feature = "upgrade_0_7_x")]
mod secondary_index_table_multimap;

use std::{collections::HashMap, path::Path};

use crate::{database_instance::DatabaseInstance, db_type::Result, Configuration, ModelBuilder};

pub(crate) fn upgrade_redb(
    database_configuration: &Configuration,
    path: impl AsRef<Path>,
    _model_builder: &HashMap<String, ModelBuilder>,
) -> Result<DatabaseInstance> {
    #[cfg(feature = "redb1")]
    redb1_to_redb2::upgrade_redb1_to_redb2(database_configuration, &path, _model_builder)?;

    let redb_builder = database_configuration.new_rdb_builder();
    let database_instance = DatabaseInstance::open_on_disk(redb_builder, &path)?;

    Ok(database_instance)
}

pub(crate) fn upgrade_underlying_database(
    _database_instance: &DatabaseInstance,
    _model_builder: &HashMap<String, ModelBuilder>,
) -> Result<()> {
    #[cfg(feature = "upgrade_0_7_x")]
    secondary_index_table_multimap::upgrade_secondary_index_table_multimap(
        _database_instance,
        _model_builder,
    )?;

    Ok(())
}
