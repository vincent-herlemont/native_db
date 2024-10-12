use crate::db_type::{Error, KeyDefinition, KeyOptions, Result};
use std::collections::HashSet;

/// See the documentation [crate::Models::define] to see how to define a model.
#[derive(Clone, Debug)]
pub struct Model {
    pub primary_key: KeyDefinition<()>,
    pub secondary_keys: HashSet<KeyDefinition<KeyOptions>>,
}

impl Model {
    pub fn check_secondary_options<F>(
        &self,
        secondary_key: &KeyDefinition<KeyOptions>,
        check: F,
    ) -> Result<()>
    where
        F: Fn(KeyOptions) -> bool,
    {
        let key = self.secondary_keys.get(secondary_key).ok_or_else(|| {
            Error::SecondaryKeyDefinitionNotFound {
                table: self.primary_key.unique_table_name.to_string(),
                key: secondary_key.unique_table_name.clone(),
            }
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
