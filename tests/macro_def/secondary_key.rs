use native_db::db_type::{DatabaseKeyDefinition, DatabaseKeyValue, Input};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(compute_primary_key), secondary_key(compute_secondary_key))]
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
    assert_eq!(primary_key, "1-test".database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                1,
                1,
                "itemsecondary_compute_secondary_key",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test-1".database_inner_key_value())
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db(
    primary_key(compute_primary_key),
    secondary_key(compute_secondary_key, unique)
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
    assert_eq!(primary_key, "1-test".database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                2,
                1,
                "itemsecondaryunique_compute_secondary_key",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test-1".database_inner_key_value())
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db(
    primary_key(compute_primary_key),
    secondary_key(compute_secondary_key, optional)
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
        if let Some(name) = &self.name {
            Some(format!("{}-{}", name, self.id))
        } else {
            None
        }
    }
}

#[test]
fn test_secondary_optional() {
    let item = ItemSecondaryOptional {
        id: 1,
        name: Some("test".to_string()),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, "1".database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                2,
                1,
                "itemsecondaryoptional_compute_secondary_key",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Optional(Some("test-1".database_inner_key_value()))
    );

    let item_none = ItemSecondaryOptional { id: 2, name: None };
    let secondary_key: HashMap<_, DatabaseKeyValue> = item_none.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                2,
                1,
                "itemsecondaryoptional_compute_secondary_key",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Optional(None)
    );
}
