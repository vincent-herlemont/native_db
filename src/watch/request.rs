use crate::db_type::{
    DatabaseInnerKeyValue, DatabaseKeyDefinition, DatabaseKeyValue, DatabaseSecondaryKeyOptions,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct WatcherRequest {
    // TODO: Maybe replace table_name by DatabaseKeyDefinition<()> or other
    pub(crate) table_name: String,
    pub(crate) primary_key: DatabaseInnerKeyValue,
    pub(crate) secondary_keys_value:
        HashMap<DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>, DatabaseKeyValue>,
}

impl WatcherRequest {
    pub fn new(
        table_name: String,
        primary_key: DatabaseInnerKeyValue,
        secondary_keys: HashMap<
            DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
            DatabaseKeyValue,
        >,
    ) -> Self {
        Self {
            table_name,
            primary_key,
            secondary_keys_value: secondary_keys,
        }
    }
}
