#[cfg(feature = "redb1")]
mod redb1_to_redb2;
use std::{collections::HashMap, path::Path};

use crate::{
    database_instance::DatabaseInstance, db_type::Result, DatabaseConfiguration, ModelBuilder,
};

pub(crate) fn upgrade(
    database_configuration: &DatabaseConfiguration,
    path: impl AsRef<Path>,
    model_builder: &HashMap<String, ModelBuilder>,
) -> Result<DatabaseInstance> {
    #[cfg(feature = "redb1")]
    redb1_to_redb2::upgrade_redb1_to_redb2(&database_configuration, &path, model_builder)?;

    let redb_builder = database_configuration.new_rdb_builder();
    let database_instance = DatabaseInstance::open_on_disk(redb_builder, &path)?;
    Ok(database_instance)
}
