use native_db::db_type::{DatabaseKeyDefinition, DatabaseKeyValue, Input};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemSecondary {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
}

#[test]
fn test_secondary() {
    let item = ItemSecondary {
        id: 1,
        name: "test".to_string(),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, 1u32.database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                1,
                1,
                "name",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test".database_inner_key_value())
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct ItemSecondaryOptional {
    #[primary_key]
    id: u32,
    #[secondary_key(optional)]
    name: Option<String>,
}

#[test]
fn test_secondary_optional() {
    let item = ItemSecondaryOptional {
        id: 1,
        name: Some("test".to_string()),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, 1u32.database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                2,
                1,
                "name",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Optional(Some("test".database_inner_key_value()))
    );

    let item_none = ItemSecondaryOptional { id: 2, name: None };
    let secondary_key: HashMap<_, DatabaseKeyValue> = item_none.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                2,
                1,
                "name",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Optional(None)
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 3, version = 1)]
#[native_db]
struct ItemSecondaryUnique {
    #[primary_key]
    id: u32,
    #[secondary_key(unique)]
    name: String,
}

#[test]
fn test_secondary_unique() {
    let item = ItemSecondaryUnique {
        id: 1,
        name: "test".to_string(),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, 1u32.database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                3,
                1,
                "name",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test".database_inner_key_value())
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 4, version = 1)]
#[native_db]
struct ItemSecondaryOthers {
    #[primary_key]
    id: u32,
    #[secondary_key(unique)]
    name: String,
    #[secondary_key()]
    name2: String,
}

#[test]
fn test_secondary_others() {
    let item = ItemSecondaryOthers {
        id: 1,
        name: "test".to_string(),
        name2: "test2".to_string(),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, 1u32.database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 2);
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                4,
                1,
                "name",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test".database_inner_key_value())
    );
    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                4,
                1,
                "name2",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test2".database_inner_key_value())
    );
}
