use crate::db_type::Key;
use std::hash::Hash;

pub trait DatabaseKey<O> {
    fn database_key(&self) -> KeyDefinition<O>;
}

#[derive(Default, Clone, Debug)]
pub struct KeyDefinition<O> {
    pub(crate) unique_table_name: String,
    pub(crate) options: O,
}

impl<O: Clone> DatabaseKey<O> for KeyDefinition<O> {
    fn database_key(&self) -> KeyDefinition<O> {
        self.clone()
    }
}

impl<O> KeyDefinition<O> {
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

impl From<&'static str> for KeyDefinition<()> {
    fn from(name: &'static str) -> Self {
        Self::new(0, 0, name, ())
    }
}

impl From<&'static str> for KeyDefinition<KeyOptions> {
    fn from(name: &'static str) -> Self {
        Self::new(0, 0, name, KeyOptions::default())
    }
}

impl PartialEq for KeyDefinition<KeyOptions> {
    fn eq(&self, other: &Self) -> bool {
        self.unique_table_name == other.unique_table_name
    }
}

impl Eq for KeyDefinition<KeyOptions> {}

impl Hash for KeyDefinition<KeyOptions> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_table_name.hash(state);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyOptions {
    pub unique: bool,
    pub optional: bool,
}

pub fn composite_key(secondary_key: &Key, primary_key: &Key) -> Key {
    let mut secondary_key = secondary_key.clone();
    secondary_key.extend(primary_key);
    secondary_key
}
