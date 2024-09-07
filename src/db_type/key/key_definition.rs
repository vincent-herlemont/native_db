use crate::db_type::Key;
use std::hash::Hash;

pub trait ToKeyDefinition<O> {
    fn key_definition(&self) -> KeyDefinition<O>;
}

#[derive(Default, Clone, Debug)]
pub struct KeyDefinition<O> {
    pub(crate) unique_table_name: String,
    pub(crate) rust_types: Vec<String>,
    pub(crate) options: O,
}

impl<O: Clone> ToKeyDefinition<O> for KeyDefinition<O> {
    fn key_definition(&self) -> KeyDefinition<O> {
        self.clone()
    }
}

impl<O> KeyDefinition<O> {
    pub fn new(
        model_id: u32,
        model_version: u32,
        name: &'static str,
        rust_types: Vec<String>,
        options: O,
    ) -> Self {
        let table_name = format!("{}_{}_{}", model_id, model_version, name);
        Self {
            options,
            rust_types,
            unique_table_name: table_name,
        }
    }

    pub fn options(&self) -> &O {
        &self.options
    }
}

// impl From<&'static str> for KeyDefinition<()> {
//     fn from(name: &'static str) -> Self {
//         Self::new(0, 0, name, ())
//     }
// }

// impl From<&'static str> for KeyDefinition<KeyOptions> {
//     fn from(name: &'static str) -> Self {
//         Self::new(0, 0, name, KeyOptions::default())
//     }
// }

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
    // The addition of a delimiter (a byte set to `0`) used between the concatenation
    // of secondary keys and primary keys ensures that there is always a byte smaller
    // than the value of the `end` of an inclusive range, which always ends with a byte
    // set to `255`. See `KeyRange` the inclusive range defined with `start..=end`.
    secondary_key.extend_with_delimiter(0, primary_key);
    secondary_key
}
