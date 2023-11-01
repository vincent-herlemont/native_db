#![cfg(not(feature = "use_native_model"))]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(
    fn_primary_key(generate_my_primary_key),
    fn_secondary_key(secondary_key_1),
    fn_secondary_key(secondary_key_2)
)]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
    pub fn secondary_key_1(&self) -> Vec<u8> {
        format!("{}", self.id).into()
    }
    pub fn secondary_key_2(&self) -> Vec<u8> {
        format!("{}", self.name).into()
    }
}

#[test]
fn test_fn_secondary_key() {
    let item = Item {
        id: 1,
        name: "test".to_string(),
    };
    let db_keys = item.struct_db_keys();
    assert_eq!(db_keys.len(), 2);

    assert_eq!(db_keys.get("item_secondary_key_1").unwrap(), b"1");
    assert_eq!(db_keys.get("item_secondary_key_2").unwrap(), b"test");
}

#[test]
fn test_init_table() {
    let init_table = Item::struct_db_schema();
    assert_eq!(init_table.table_name, "item");
    assert_eq!(init_table.primary_key, "generate_my_primary_key");
    assert_eq!(
        init_table.secondary_tables_name,
        HashSet::from_iter(vec!["item_secondary_key_1", "item_secondary_key_2"].into_iter())
    );
}

#[test]
fn test_struct_db_keys() {
    let secondary_table_name_1 = ItemKey::secondary_key_1.secondary_table_name();
    assert_eq!(secondary_table_name_1, "item_secondary_key_1");
    let secondary_table_name_2 = ItemKey::secondary_key_2.secondary_table_name();
    assert_eq!(secondary_table_name_2, "item_secondary_key_2");
}

#[test]
fn test_insert_duplicate_key() {
    let tf = tests::init();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };

    let item_2 = Item {
        id: 2,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    let mut tables = txn.tables();
    tables.insert(&txn, item_1).unwrap();
    let result = tables.insert(&txn, item_2);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::DuplicateKey { .. }));
}

#[test]
fn test_insert_and_get_on_transaction() {
    let tf = tests::init();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };

    let item_2 = Item {
        id: 2,
        name: "test2".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item_1).unwrap();
        tables.insert(&txn, item_2).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let result: Item = tables
            .secondary_get(&txn, ItemKey::secondary_key_1, b"1")
            .unwrap()
            .unwrap();
        assert_eq!(result.name, "test");
        let result: Item = tables
            .secondary_get(&txn, ItemKey::secondary_key_2, b"test2")
            .unwrap()
            .unwrap();
        assert_eq!(result.id, 2);
    }
}

#[test]
fn test_insert_and_get_on_readonly_transaction() {
    let tf = tests::init();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };

    let item_2 = Item {
        id: 2,
        name: "test2".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item_1).unwrap();
        tables.insert(&txn, item_2).unwrap();
    }
    txn.commit().unwrap();

    let txn_read = db.read_transaction().unwrap();
    {
        let mut tables = txn_read.tables();
        let result: Item = tables
            .secondary_get(&txn_read, ItemKey::secondary_key_1, b"1")
            .unwrap()
            .unwrap();
        assert_eq!(result.name, "test");
        let result: Item = tables
            .secondary_get(&txn_read, ItemKey::secondary_key_2, b"test2")
            .unwrap()
            .unwrap();
        assert_eq!(result.id, 2);
    }
}

#[test]
fn test_insert_and_get() {
    let tf = tests::init();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };

    let item_2 = Item {
        id: 2,
        name: "test2".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item_1).unwrap();
        tables.insert(&txn, item_2).unwrap();
    }
    txn.commit().unwrap();

    let txn_read = db.read_transaction().unwrap();
    let mut tables = txn_read.tables();
    let result: Item = tables
        .secondary_get(&txn_read, ItemKey::secondary_key_1, b"1")
        .unwrap()
        .unwrap();
    assert_eq!(result.name, "test");
    let result: Item = tables
        .secondary_get(&txn_read, ItemKey::secondary_key_2, b"test2")
        .unwrap()
        .unwrap();
    assert_eq!(result.id, 2);
}
