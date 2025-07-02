use crate::db_type::{Error, Result};
use crate::{db_type::Key, Model, ToKey};
use std::{hash::Hash, ops::RangeBounds};

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
        let table_name = format!("{model_id}_{model_version}_{name}");
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
fn _check_key_type_from_key_definition<K: ToKey>(
    key_definition: &KeyDefinition<KeyOptions>,
) -> Result<()> {
    if K::check_type()
        && !K::key_names()
            .iter()
            .any(|name| key_definition.rust_types.contains(name))
    {
        return Err(Error::MismatchedKeyType {
            key_name: key_definition.unique_table_name.to_string(),
            expected_types: key_definition.rust_types.clone(),
            got_types: K::key_names(),
            operation: "get".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod test_check_key_type_from_key_definition {
    use super::_check_key_type_from_key_definition;
    use crate::db_type::KeyDefinition;
    use crate::ToKey;

    #[test]
    fn same_key_never_fails() {
        assert!(
            _check_key_type_from_key_definition::<String>(&KeyDefinition {
                unique_table_name: "test".to_string(),
                rust_types: vec!["String".to_string()],
                options: Default::default(),
            })
            .is_ok()
        );
    }

    #[test]
    fn generic_key_never_fails() {
        assert!(
            _check_key_type_from_key_definition::<crate::db_type::key::Key>(&KeyDefinition {
                unique_table_name: "test".to_string(),
                rust_types: vec!["String".to_string()],
                options: Default::default(),
            })
            .is_ok()
        );
    }

    #[test]
    fn wrong_key_does_fail() {
        assert!(_check_key_type_from_key_definition::<i32>(&KeyDefinition {
            unique_table_name: "test".to_string(),
            rust_types: vec!["String".to_string()],
            options: Default::default(),
        })
        .is_err());
    }

    /// A custom user-provided key that happens to be named the same as `crate::db_type::key::Key`.
    #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
    pub struct Key(Vec<u8>);

    impl ToKey for Key {
        fn to_key(&self) -> crate::db_type::key::Key {
            crate::db_type::key::Key::new(self.0.clone())
        }

        fn key_names() -> Vec<String> {
            vec!["Key".to_string()]
        }
    }

    /// Test that type checking still works if a user supplies a custom `Key`
    /// called "Key".
    #[test]
    fn key_name_collision_does_fail() {
        assert!(_check_key_type_from_key_definition::<Key>(&KeyDefinition {
            unique_table_name: "test".to_string(),
            rust_types: vec!["String".to_string()],
            options: Default::default(),
        })
        .is_err());
    }
}

fn _check_key_type<K: ToKey>(model: &Model) -> Result<()> {
    if !K::key_names()
        .iter()
        .any(|name| model.primary_key.rust_types.contains(name))
    {
        return Err(Error::MismatchedKeyType {
            key_name: model.primary_key.unique_table_name.to_string(),
            expected_types: model.primary_key.rust_types.clone(),
            got_types: K::key_names(),
            operation: "get".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn check_key_type<K: ToKey>(model: &Model, _key: &K) -> Result<()> {
    _check_key_type::<K>(model)
}

pub(crate) fn check_key_type_from_key_definition<K: ToKey>(
    key_definition: &KeyDefinition<KeyOptions>,
    _key: &K,
) -> Result<()> {
    _check_key_type_from_key_definition::<K>(key_definition)
}

pub(crate) fn check_range_key_range_bounds<K: ToKey>(
    model: &Model,
    _range: &impl RangeBounds<K>,
) -> Result<()> {
    _check_key_type::<K>(model)
}

pub(crate) fn check_range_key_range_bounds_from_key_definition<K: ToKey>(
    key_definition: &KeyDefinition<KeyOptions>,
    _range: &impl RangeBounds<K>,
) -> Result<()> {
    _check_key_type_from_key_definition::<K>(key_definition)
}
