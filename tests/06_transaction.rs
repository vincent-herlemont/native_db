#![cfg(not(feature = "use_native_model"))]
mod tests;

use std::panic::AssertUnwindSafe;
use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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
fn test_transaction_obj_1() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };
    {
        let tx_write = db.transaction().unwrap();
        {
            let mut tables = tx_write.tables();
            tables.insert(&tx_write, item).unwrap();
            // Random fail here...
        }
        tx_write.commit().unwrap();
    }

    let txn_read = db.read_transaction().unwrap();
    let result: Item = txn_read
        .tables()
        .primary_get(&txn_read, b"1-test")
        .unwrap()
        .unwrap();
    assert_eq!(result.id, 1);
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct Item2 {
    id: u32,
    name: String,
}

impl Item2 {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn test_transaction_obj_1_and_obj_2() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>();
    db.define::<Item2>();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };
    let item_2 = Item2 {
        id: 2,
        name: "test".to_string(),
    };

    {
        let tx_write = db.transaction().unwrap();
        {
            let mut tables = tx_write.tables();
            tables.insert(&tx_write, item_1).unwrap();
            tables.insert(&tx_write, item_2).unwrap();
        }
        tx_write.commit().unwrap();
    }

    let txn_read = db.read_transaction().unwrap();
    let result: Item = txn_read
        .tables()
        .primary_get(&txn_read, b"1-test")
        .unwrap()
        .unwrap();
    assert_eq!(result.id, 1);
    let result: Item2 = txn_read
        .tables()
        .primary_get(&txn_read, b"2-test")
        .unwrap()
        .unwrap();
    assert_eq!(result.id, 2);
}

#[allow(unreachable_code)]
#[test]
fn test_transaction_fail() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };
    {
        let tx_write = db.transaction().unwrap();
        {
            let mut tables = tx_write.tables();
            tables.insert(&tx_write, item_1).unwrap();
            // Random fail here...
        }
        tx_write.commit().unwrap();
    }
    {
        let txn_read = db.read_transaction().unwrap();
        let result: Item = txn_read
            .tables()
            .primary_get(&txn_read, b"1-test")
            .unwrap()
            .unwrap();
        assert_eq!(result.id, 1);
    }

    let item_2 = Item {
        id: 2,
        name: "test".to_string(),
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let tx_write = db.transaction().unwrap();
        {
            let mut tables = tx_write.tables();
            tables.insert(&tx_write, item_2).unwrap();
            panic!("Random panic here...")
        }

        tx_write.commit().unwrap();
    }));

    assert!(result.is_err());

    let txn_read = db.read_transaction().unwrap();
    let result = txn_read
        .tables()
        .primary_get::<Item>(&txn_read, b"2-test")
        .unwrap();
    assert!(result.is_none());
}
