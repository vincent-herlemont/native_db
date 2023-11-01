#![cfg(not(feature = "use_native_model"))]

use redb::TableHandle;
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(
    fn_primary_key(generate_my_primary_key),
    fn_secondary_key(generate_my_secondary_key)
)]
struct Item {
    id: u32,
    name: String,
    tag: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }

    pub fn generate_my_secondary_key(&self) -> Vec<u8> {
        let mut tag = self.tag.clone().into_bytes();
        let primary_key = self.generate_my_primary_key();
        tag.extend_from_slice(&primary_key);
        tag
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
        tag: "red".to_string(),
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

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.stats_tables.len(), 2);
    assert_eq!(stats.stats_tables[0].name, "item");
    assert_eq!(stats.stats_tables[0].num_raw, 5);
    assert_eq!(stats.stats_tables[1].name, "item_generate_my_secondary_key");
    assert_eq!(stats.stats_tables[1].num_raw, 5);

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

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.stats_tables.len(), 2);
    assert_eq!(stats.stats_tables[0].name, "item");
    assert_eq!(stats.stats_tables[0].num_raw, 0);
    assert_eq!(stats.stats_tables[1].name, "item_generate_my_secondary_key");
    assert_eq!(stats.stats_tables[1].num_raw, 0);
}

#[test]
fn drain_a_part() {
    let tf = TmpFs::new().unwrap();

    let mut item = Item {
        id: 1,
        name: "test".to_string(),
        tag: "red".to_string(),
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

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.stats_tables.len(), 2);
    assert_eq!(stats.stats_tables[0].name, "item");
    assert_eq!(stats.stats_tables[0].num_raw, 5);
    assert_eq!(stats.stats_tables[1].name, "item_generate_my_secondary_key");
    assert_eq!(stats.stats_tables[1].num_raw, 5);

    // Count items
    let txn_read = db.read_transaction().unwrap();
    let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
    assert_eq!(len, 5);

    // Drain items
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let items = tables
            .primary_drain::<Item>(&txn, ..3_i32.to_be_bytes().as_slice())
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

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.stats_tables.len(), 2);
    assert_eq!(stats.stats_tables[0].name, "item");
    assert_eq!(stats.stats_tables[0].num_raw, 3);
    assert_eq!(stats.stats_tables[1].name, "item_generate_my_secondary_key");
    assert_eq!(stats.stats_tables[1].num_raw, 3);
}
