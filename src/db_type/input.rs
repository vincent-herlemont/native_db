use crate::db_type::{composite_key, Error, Key, KeyDefinition, KeyEntry, KeyOptions, Result};

#[derive(Debug)]
pub(crate) struct DatabaseInput {
    pub(crate) primary_key: Key,
    pub(crate) secondary_keys: std::collections::HashMap<KeyDefinition<KeyOptions>, KeyEntry>,
    pub(crate) value: Vec<u8>,
}

impl DatabaseInput {
    pub(crate) fn secondary_key_value(
        &self,
        secondary_key_def: &KeyDefinition<KeyOptions>,
    ) -> Result<KeyEntry> {
        let secondary_key = self.secondary_keys.get(secondary_key_def).ok_or(
            Error::SecondaryKeyDefinitionNotFound {
                table: "".to_string(),
                key: secondary_key_def.unique_table_name.clone(),
            },
        )?;
        let out = if !secondary_key_def.options.unique {
            match secondary_key {
                KeyEntry::Default(value) => {
                    KeyEntry::Default(composite_key(value, &self.primary_key))
                }
                KeyEntry::Optional(value) => {
                    let value = value
                        .as_ref()
                        .map(|value| composite_key(value, &self.primary_key));
                    KeyEntry::Optional(value)
                }
            }
        } else {
            secondary_key.clone()
        };
        Ok(out)
    }
}

pub trait Input: Sized + native_model::Model {
    fn native_db_model() -> crate::DatabaseModel;

    fn native_db_primary_key(&self) -> Key;

    fn native_db_secondary_keys(
        &self,
    ) -> std::collections::HashMap<KeyDefinition<KeyOptions>, KeyEntry>;
    fn native_db_bincode_encode_to_vec(&self) -> Result<Vec<u8>>;
    fn native_db_bincode_decode_from_slice(slice: &[u8]) -> Result<Self>;

    fn to_item(&self) -> Result<DatabaseInput> {
        Ok(DatabaseInput {
            primary_key: self.native_db_primary_key(),
            secondary_keys: self.native_db_secondary_keys(),
            value: self.native_db_bincode_encode_to_vec()?,
        })
    }
}
