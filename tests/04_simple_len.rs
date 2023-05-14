mod tests;

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
fn test_simple_len() {
    let tf = tests::init();

    let mut item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item.clone()).unwrap();
    }
    txn.commit().unwrap();

    {
        let txn_read = db.read_transaction().unwrap();
        let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
        assert_eq!(len, 1);
    }

    item.id = 2;
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item.clone()).unwrap();
    }
    txn.commit().unwrap();

    {
        let txn_read = db.read_transaction().unwrap();
        let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
        assert_eq!(len, 2);
    }
}

#[test]
fn test_simple_len_txn_write() {
    let tf = tests::init();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item.clone()).unwrap();
    }
    txn.commit().unwrap();

    let txn_read = db.read_transaction().unwrap();
    {
        let mut tables = txn_read.tables();
        let len = tables.len::<Item>(&txn_read).unwrap();
        assert_eq!(len, 1);
    }
}
