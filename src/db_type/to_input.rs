use crate::Key;

use super::{Input, KeyDefinition, KeyEntry, KeyOptions, Result};

pub trait ToInput: Sized + native_model::Model {
    fn native_db_model() -> crate::Model;
    fn native_db_primary_key(&self) -> Key;
    fn native_db_secondary_keys(
        &self,
    ) -> std::collections::HashMap<KeyDefinition<KeyOptions>, KeyEntry>;
    fn native_db_bincode_encode_to_vec(&self) -> Result<Vec<u8>>;
    fn native_db_bincode_decode_from_slice(slice: &[u8]) -> Result<Self>;

    fn native_db_input(&self) -> Result<Input> {
        Ok(Input {
            primary_key: self.native_db_primary_key(),
            secondary_keys: self.native_db_secondary_keys(),
            value: self.native_db_bincode_encode_to_vec()?,
        })
    }
}
