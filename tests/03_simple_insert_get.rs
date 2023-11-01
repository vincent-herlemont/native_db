#![cfg(not(feature = "native_model"))]
mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn test_insert_get_my_item() {
    let tf = tests::init();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item).unwrap();
    }
    txn.commit().unwrap();

    {
        let txn_read = db.read_transaction().unwrap();
        let result: Item = txn_read
            .tables()
            .primary_get(&txn_read, b"1-test")
            .unwrap()
            .unwrap();
        assert_eq!(result.id, 1);
    }
}

#[test]
fn test_insert_get_my_item_write_txn() {
    let tf = tests::init();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item).unwrap();
    }
    txn.commit().unwrap();

    let txn_read = db.read_transaction().unwrap();
    {
        let mut tables = txn_read.tables();
        let result: Item = tables.primary_get(&txn_read, b"1-test").unwrap().unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.name, "test");
    }
}

#[test]
fn test_insert_get_my_item_readonly_txn() {
    let tf = tests::init();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item).unwrap();
    }
    txn.commit().unwrap();

    let txn_read = db.read_transaction().unwrap();
    {
        let mut tables = txn_read.tables();
        let result: Item = tables.primary_get(&txn_read, b"1-test").unwrap().unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.name, "test");
    }
}
