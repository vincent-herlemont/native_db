use crate::db_type::{Error, KeyDefinition, KeyOptions, Result};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct DatabaseModel {
    pub primary_key: KeyDefinition<()>,
    pub secondary_keys: HashSet<KeyDefinition<KeyOptions>>,
}

impl DatabaseModel {
    pub fn check_secondary_options<F>(
        &self,
        secondary_key: &KeyDefinition<KeyOptions>,
        check: F,
    ) -> Result<()>
    where
        F: Fn(KeyOptions) -> bool,
    {
        let key = self
            .secondary_keys
            .get(secondary_key.into())
            .ok_or_else(|| Error::SecondaryKeyDefinitionNotFound {
                table: self.primary_key.unique_table_name.to_string(),
                key: secondary_key.unique_table_name.clone(),
            })?;

        if check(key.options.clone()) {
            Ok(())
        } else {
            Err(Error::SecondaryKeyConstraintMismatch {
                table: self.primary_key.unique_table_name.to_string(),
                key: secondary_key.unique_table_name.clone(),
                got: key.options.clone(),
            })
        }
    }
}
