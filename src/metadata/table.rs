use super::Metadata;
use crate::db_type::Result;
use redb::TableDefinition;

pub const VERSION_NATIVE_DB_NAME: &str = "version_native_db";
pub const VERSION_NATIVE_MODEL_NAME: &str = "version_native_model";

use crate::database_instance::DatabaseInstance;

const TABLE: TableDefinition<&str, &str> = TableDefinition::new("metadata");

pub fn save_metadata(database_instance: &DatabaseInstance, configuration: &Metadata) -> Result<()> {
    let table = database_instance.redb_database()?;
    let write_thx = table.begin_write()?;
    {
        let mut table = write_thx.open_table(TABLE)?;
        table.insert(VERSION_NATIVE_DB_NAME, configuration.current_version())?;
        table.insert(
            VERSION_NATIVE_MODEL_NAME,
            configuration.current_native_model_version(),
        )?;
    }
    write_thx.commit()?;

    Ok(())
}

pub fn load_or_create_metadata(database_instance: &DatabaseInstance) -> Result<Metadata> {
    let database = database_instance.redb_database()?;
    let read_thx = database.begin_read()?;

    if let Ok(table) = read_thx.open_table(TABLE) {
        let current_version = table
            .get(VERSION_NATIVE_DB_NAME)?
            .expect("Fatal error: current_version not found");
        let current_native_model_version = table
            .get(VERSION_NATIVE_MODEL_NAME)?
            .expect("Fatal error: current_native_model_version not found");
        Ok(Metadata::new(
            current_version.value().to_string(),
            current_native_model_version.value().to_string(),
        ))
    } else {
        // Create the metadata table if it does not exist
        let metadata = Metadata::default();
        save_metadata(database_instance, &metadata)?;
        Ok(metadata)
    }
}
