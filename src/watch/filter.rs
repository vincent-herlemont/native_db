use crate::KeyDefinition;

#[derive(Eq, PartialEq, Clone)]
pub(crate) struct TableFilter {
    pub(crate) table_name: &'static [u8],
    pub(crate) key_filter: KeyFilter,
}

#[derive(Eq, PartialEq, Clone)]
pub(crate) enum KeyFilter {
    Primary(Option<Vec<u8>>),
    PrimaryStartWith(Vec<u8>),
    Secondary(Vec<u8>, Option<Vec<u8>>),
    SecondaryStartWith(Vec<u8>, Vec<u8>),
}

impl TableFilter {
    pub(crate) fn new_primary(table_name: &'static [u8], key: Option<&[u8]>) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::Primary(key.map(|k| k.to_vec())),
        }
    }

    pub(crate) fn new_primary_start_with(table_name: &'static [u8], key_prefix: &[u8]) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::PrimaryStartWith(key_prefix.to_vec()),
        }
    }

    pub(crate) fn new_secondary<K: KeyDefinition>(
        table_name: &'static [u8],
        key: K,
        value: Option<&[u8]>,
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::Secondary(
                key.secondary_table_name().as_bytes().to_vec(),
                value.map(|v| v.to_vec()),
            ),
        }
    }

    pub(crate) fn new_secondary_start_with<K: KeyDefinition>(
        table_name: &'static [u8],
        key: K,
        key_prefix: &[u8],
    ) -> Self {
        Self {
            table_name,
            key_filter: KeyFilter::SecondaryStartWith(
                key.secondary_table_name().as_bytes().to_vec(),
                key_prefix.to_vec(),
            ),
        }
    }
}
