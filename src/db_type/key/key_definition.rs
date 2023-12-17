use crate::db_type::DatabaseInnerKeyValue;
use std::hash::Hash;

pub trait KeyDefinition<O> {
    fn database_key(&self) -> DatabaseKeyDefinition<O>;
}

#[derive(Default, Clone, Debug)]
pub struct DatabaseKeyDefinition<O> {
    pub(crate) unique_table_name: String,
    pub(crate) options: O,
}

impl<O: Clone> KeyDefinition<O> for DatabaseKeyDefinition<O> {
    fn database_key(&self) -> DatabaseKeyDefinition<O> {
        self.clone()
    }
}

impl<O> DatabaseKeyDefinition<O> {
    pub fn new(model_id: u32, model_version: u32, name: &'static str, options: O) -> Self {
        let table_name = format!("{}_{}_{}", model_id, model_version, name);
        Self {
            options,
            unique_table_name: table_name,
        }
    }

    pub fn options(&self) -> &O {
        &self.options
    }
}

impl From<&'static str> for DatabaseKeyDefinition<()> {
    fn from(name: &'static str) -> Self {
        Self::new(0, 0, name, ())
    }
}

impl From<&'static str> for DatabaseKeyDefinition<DatabaseSecondaryKeyOptions> {
    fn from(name: &'static str) -> Self {
        Self::new(0, 0, name, DatabaseSecondaryKeyOptions::default())
    }
}

impl PartialEq for DatabaseKeyDefinition<DatabaseSecondaryKeyOptions> {
    fn eq(&self, other: &Self) -> bool {
        self.unique_table_name == other.unique_table_name
    }
}

impl Eq for DatabaseKeyDefinition<DatabaseSecondaryKeyOptions> {}

impl Hash for DatabaseKeyDefinition<DatabaseSecondaryKeyOptions> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_table_name.hash(state);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatabaseSecondaryKeyOptions {
    pub unique: bool,
    pub optional: bool,
}

pub fn composite_key(
    secondary_key: &DatabaseInnerKeyValue,
    primary_key: &DatabaseInnerKeyValue,
) -> DatabaseInnerKeyValue {
    let mut secondary_key = secondary_key.clone();
    secondary_key.extend(primary_key);
    secondary_key
}
