use native_db::db_type::{KeyDefinition, KeyEntry, ToInput};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(compute_primary_key -> String),
    secondary_key(compute_secondary_key -> String),
)]
struct ItemSecondary {
    id: u32,
    name: String,
}

impl ItemSecondary {
    pub fn compute_primary_key(&self) -> String {
        format!("{}-{}", self.id, self.name)
    }
    pub fn compute_secondary_key(&self) -> String {
        format!("{}-{}", self.name, self.id)
    }
}

#[test]
fn test_secondary() {
    let item = ItemSecondary {
        id: 1,
        name: "test".to_string(),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, "1-test".to_key());

    let secondary_key: HashMap<_, KeyEntry> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&KeyDefinition::new(
                1,
                1,
                "compute_secondary_key",
                vec!["String".to_string()],
                Default::default()
            ))
            .unwrap(),
        &KeyEntry::Default("test-1".to_key())
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db(
    primary_key(compute_primary_key -> String),
    secondary_key(compute_secondary_key -> String, unique)
)]
struct ItemSecondaryUnique {
    id: u32,
    name: String,
}

impl ItemSecondaryUnique {
    pub fn compute_primary_key(&self) -> String {
        format!("{}-{}", self.id, self.name)
    }
    pub fn compute_secondary_key(&self) -> String {
        format!("{}-{}", self.name, self.id)
    }
}

#[test]
fn test_secondary_unique() {
    let item = ItemSecondaryUnique {
        id: 1,
        name: "test".to_string(),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, "1-test".to_key());

    let secondary_key: HashMap<_, KeyEntry> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&KeyDefinition::new(
                2,
                1,
                "compute_secondary_key",
                vec!["String".to_string()],
                Default::default()
            ))
            .unwrap(),
        &KeyEntry::Default("test-1".to_key())
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db(
    primary_key(compute_primary_key -> String),
    secondary_key(compute_secondary_key -> Option<String>, optional)
)]
struct ItemSecondaryOptional {
    id: u32,
    name: Option<String>,
}

impl ItemSecondaryOptional {
    pub fn compute_primary_key(&self) -> String {
        format!("{}", self.id)
    }
    pub fn compute_secondary_key(&self) -> Option<String> {
        self.name
            .as_ref()
            .map(|name| format!("{}-{}", name, self.id))
    }
}

#[test]
fn test_secondary_optional() {
    let item = ItemSecondaryOptional {
        id: 1,
        name: Some("test".to_string()),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, "1".to_key());

    let secondary_key: HashMap<_, KeyEntry> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&KeyDefinition::new(
                2,
                1,
                "compute_secondary_key",
                vec!["Option<String>".to_string()],
                Default::default()
            ))
            .unwrap(),
        &KeyEntry::Optional(Some("test-1".to_key()))
    );

    let item_none = ItemSecondaryOptional { id: 2, name: None };
    let secondary_key: HashMap<_, KeyEntry> = item_none.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&KeyDefinition::new(
                2,
                1,
                "compute_secondary_key",
                vec!["Option<String>".to_string()],
                Default::default()
            ))
            .unwrap(),
        &KeyEntry::Optional(None)
    );
}
