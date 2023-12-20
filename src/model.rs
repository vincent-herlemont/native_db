use crate::db_type::{DatabaseKeyDefinition, DatabaseSecondaryKeyOptions, Error, Result};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct DatabaseModel {
    pub primary_key: DatabaseKeyDefinition<()>,
    pub secondary_keys: HashSet<DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>>,
}

impl DatabaseModel {
    pub fn check_secondary_options<F>(
        &self,
        secondary_key: &DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
        check: F,
    ) -> Result<()>
    where
        F: Fn(DatabaseSecondaryKeyOptions) -> bool,
    {
        let key = self.secondary_keys.get(secondary_key).ok_or_else(|| {
            Error::SecondaryKeyDefinitionNotFound {
                table: self.primary_key.unique_table_name.to_string(),
                key: secondary_key.unique_table_name.clone(),
            }
        })?;

        if check(key.options) {
            Ok(())
        } else {
            Err(Error::SecondaryKeyConstraintMismatch {
                table: self.primary_key.unique_table_name.to_string(),
                key: secondary_key.unique_table_name.clone(),
                got: key.options,
            })
        }
    }
}
