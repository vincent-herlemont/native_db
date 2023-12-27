use crate::db_type::{
    DatabaseInnerKeyValue, DatabaseKeyDefinition, DatabaseSecondaryKeyOptions, KeyDefinition,
};

#[derive(Eq, PartialEq, Clone)]
pub struct TableFilter {
    pub(crate) table_name: String,
    pub(crate) key_filter: KeyFilter,
}

#[derive(Eq, PartialEq, Clone)]
pub enum KeyFilter {
    Primary(Option<DatabaseInnerKeyValue>),
    PrimaryStartWith(DatabaseInnerKeyValue),
    Secondary(
        DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
        Option<DatabaseInnerKeyValue>,
    ),
    SecondaryStartWith(
        DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
        DatabaseInnerKeyValue,
    ),
}

impl TableFilter {
    pub(crate) const fn new_primary(
        table_name: String,
        key: Option<DatabaseInnerKeyValue>,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::Primary(key),
        }
    }

    pub(crate) const fn new_primary_start_with(
        table_name: String,
        key_prefix: DatabaseInnerKeyValue,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::PrimaryStartWith(key_prefix),
        }
    }

    pub(crate) fn new_secondary<K: KeyDefinition<DatabaseSecondaryKeyOptions>>(
        table_name: String,
        key_def: &K,
        key: Option<DatabaseInnerKeyValue>,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::Secondary(key_def.database_key(), key),
        }
    }

    pub(crate) fn new_secondary_start_with<K: KeyDefinition<DatabaseSecondaryKeyOptions>>(
        table_name: String,
        key: &K,
        key_prefix: DatabaseInnerKeyValue,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::SecondaryStartWith(key.database_key(), key_prefix),
        }
    }
}
