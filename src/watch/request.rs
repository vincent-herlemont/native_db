use crate::db_type::{Key, KeyDefinition, KeyEntry, KeyOptions};
use std::collections::HashMap;

#[derive(Clone)]
pub struct WatcherRequest {
    // TODO: Maybe replace table_name by KeyDefinition<()> or other
    pub(crate) table_name: String,
    pub(crate) primary_key: Key,
    pub(crate) secondary_keys_value: HashMap<KeyDefinition<KeyOptions>, KeyEntry>,
}

impl WatcherRequest {
    pub fn new(
        table_name: String,
        primary_key: Key,
        secondary_keys: HashMap<KeyDefinition<KeyOptions>, KeyEntry>,
    ) -> Self {
        Self {
            table_name,
            primary_key,
            secondary_keys_value: secondary_keys,
        }
    }
}
