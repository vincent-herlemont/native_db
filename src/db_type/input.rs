use crate::db_type::{
    composite_key, DatabaseInnerKeyValue, DatabaseKeyDefinition, DatabaseKeyValue,
    DatabaseSecondaryKeyOptions, Error, Result,
};

#[derive(Debug)]
pub struct DatabaseInput {
    pub(crate) primary_key: DatabaseInnerKeyValue,
    pub(crate) secondary_keys: std::collections::HashMap<
        DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
        DatabaseKeyValue,
    >,
    pub(crate) value: Vec<u8>,
}

impl DatabaseInput {
    pub(crate) fn secondary_key_value(
        &self,
        secondary_key_def: &DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
    ) -> Result<DatabaseKeyValue> {
        let secondary_key = self.secondary_keys.get(secondary_key_def).ok_or(
            Error::SecondaryKeyDefinitionNotFound {
                table: "".to_string(),
                key: secondary_key_def.unique_table_name.clone(),
            },
        )?;
        let out = if !secondary_key_def.options.unique {
            match secondary_key {
                DatabaseKeyValue::Default(value) => {
                    DatabaseKeyValue::Default(composite_key(value, &self.primary_key))
                }
                DatabaseKeyValue::Optional(value) => {
                    let value = value
                        .as_ref()
                        .map(|value| composite_key(value, &self.primary_key));
                    DatabaseKeyValue::Optional(value)
                }
            }
        } else {
            secondary_key.clone()
        };
        Ok(out)
    }
}

pub trait Input: Sized + native_model::Model {
    fn native_db_model() -> crate::Model;

    fn native_db_primary_key(&self) -> DatabaseInnerKeyValue;

    fn native_db_secondary_keys(
        &self,
    ) -> std::collections::HashMap<
        DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
        DatabaseKeyValue,
    >;
    fn native_db_bincode_encode_to_vec(&self) -> Vec<u8>;
    fn native_db_bincode_decode_from_slice(slice: &[u8]) -> Self;

    fn to_item(&self) -> DatabaseInput {
        DatabaseInput {
            primary_key: self.native_db_primary_key(),
            secondary_keys: self.native_db_secondary_keys(),
            value: self.native_db_bincode_encode_to_vec(),
        }
    }
}
