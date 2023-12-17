use crate::db_type::{DatabaseKeyDefinition, DatabaseSecondaryKeyOptions, Error, Result};
use std::collections::HashSet;

/// Model of the Item. Returned by the [`<your_item>::native_db_model()`](crate::Input::native_db_model) method.
#[derive(Clone, Debug)]
pub struct Model {
    pub primary_key: DatabaseKeyDefinition<()>,
    pub secondary_keys: HashSet<DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>>,
}

impl Model {
    pub fn check_secondary_options<F>(
        &self,
        secondary_key: &DatabaseKeyDefinition<DatabaseSecondaryKeyOptions>,
        check: F,
    ) -> Result<()>
    where
        F: Fn(DatabaseSecondaryKeyOptions) -> bool,
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
