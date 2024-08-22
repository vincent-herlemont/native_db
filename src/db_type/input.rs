use crate::db_type::{Error, Key, KeyDefinition, KeyEntry, KeyOptions, Result};

#[derive(Debug)]
pub struct Input {
    pub(crate) primary_key: Key,
    pub(crate) secondary_keys: std::collections::HashMap<KeyDefinition<KeyOptions>, KeyEntry>,
    pub(crate) value: Vec<u8>,
}

impl Input {
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
                    // KeyEntry::Default(composite_key(value, &self.primary_key))
                    KeyEntry::Default(value.to_owned())
                }
                KeyEntry::Optional(value) => {
                    // let value = value
                    //     .as_ref()
                    //     .map(|value| composite_key(value, &self.primary_key));
                    // KeyEntry::Optional(value)
                    KeyEntry::Optional(value.as_ref().map(|value| value.to_owned()))
                }
            }
        } else {
            secondary_key.clone()
        };
        Ok(out)
    }
}
