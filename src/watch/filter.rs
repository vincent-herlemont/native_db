use crate::db_type::{Key, KeyDefinition, KeyOptions, KeyRange, ToKeyDefinition};

#[derive(Eq, PartialEq, Clone)]
pub(crate) struct TableFilter {
    pub(crate) table_name: String,
    pub(crate) key_filter: KeyFilter,
}

#[derive(Eq, PartialEq, Clone)]
pub(crate) enum KeyFilter {
    Primary(Option<Key>),
    PrimaryStartWith(Key),
    PrimaryRange(KeyRange),
    Secondary(KeyDefinition<KeyOptions>, Option<Key>),
    SecondaryStartWith(KeyDefinition<KeyOptions>, Key),
    SecondaryRange(KeyDefinition<KeyOptions>, KeyRange),
}

impl TableFilter {
    pub(crate) fn new_primary(table_name: String, key: Option<Key>) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::Primary(key.map(|k| k.to_owned())),
        }
    }

    pub(crate) fn new_primary_start_with(table_name: String, key_prefix: Key) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::PrimaryStartWith(key_prefix.to_owned()),
        }
    }

    pub(crate) fn new_primary_range(table_name: String, range: KeyRange) -> TableFilter {
        Self {
            table_name,
            key_filter: KeyFilter::PrimaryRange(range),
        }
    }

    pub(crate) fn new_secondary<K: ToKeyDefinition<KeyOptions>>(
        table_name: String,
        key_def: &K,
        key: Option<Key>,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::Secondary(key_def.key_definition(), key.map(|k| k.to_owned())),
        }
    }

    pub(crate) fn new_secondary_start_with<K: ToKeyDefinition<KeyOptions>>(
        table_name: String,
        key: &K,
        key_prefix: Key,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::SecondaryStartWith(key.key_definition(), key_prefix.to_owned()),
        }
    }

    pub(crate) fn new_secondary_range<K: ToKeyDefinition<KeyOptions>>(
        table_name: String,
        key_def: &K,
        range: KeyRange,
    ) -> TableFilter {
        Self {
            table_name,
            key_filter: KeyFilter::SecondaryRange(key_def.key_definition(), range),
        }
    }
}
