#![cfg(not(feature = "native_model"))]

use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(pk = generate_my_primary_key)]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        self.id.to_le_bytes().to_vec()
    }

    pub fn inc(&mut self) -> &Self {
        self.id += 1;
        self
    }
}

#[test]
fn drain_all() {
    let tf = TmpFs::new().unwrap();

    let mut item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>().unwrap();

    // Insert 5 items
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item.clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
    }
    txn.commit().unwrap();

    // Count items
    let txn_read = db.read_transaction().unwrap();
    let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
    assert_eq!(len, 5);

    // Drain items
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let items = tables.primary_drain::<Item>(&txn, ..).unwrap();
        assert_eq!(items.len(), 5);
    }
    txn.commit().unwrap();

    // Count items
    let txn_read = db.read_transaction().unwrap();
    let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
    assert_eq!(len, 0);
}

#[test]
fn drain_a_part() {
    let tf = TmpFs::new().unwrap();

    let mut item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>().unwrap();

    // Insert 5 items
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item.clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
        tables.insert(&txn, item.inc().clone()).unwrap();
    }
    txn.commit().unwrap();

    // Count items
    let txn_read = db.read_transaction().unwrap();
    let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
    assert_eq!(len, 5);

    // Drain items
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let items = tables
            .primary_drain::<Item>(&txn, ..3_i32.to_le_bytes().as_slice())
            .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, 1);
        assert_eq!(items[1].id, 2);
    }
    txn.commit().unwrap();

    // Count items
    let txn_read = db.read_transaction().unwrap();
    let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
    assert_eq!(len, 3);
}
