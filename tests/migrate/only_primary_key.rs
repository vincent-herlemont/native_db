use native_model::native_model;
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use struct_db::ReadableTable;
use struct_db::{struct_db, Db};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct ItemV1 {
    id: u32,
    name: String,
}

impl ItemV1 {
    #[allow(dead_code)]
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 2, from = ItemV1)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct ItemV2 {
    id: u64,
    name: String,
}

impl From<ItemV1> for ItemV2 {
    fn from(item: ItemV1) -> Self {
        ItemV2 {
            id: item.id as u64,
            name: item.name,
        }
    }
}

impl From<ItemV2> for ItemV1 {
    fn from(item: ItemV2) -> Self {
        ItemV1 {
            id: item.id as u32,
            name: item.name,
        }
    }
}

impl ItemV2 {
    #[allow(dead_code)]
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn test_migrate() {
    let tf = TmpFs::new().unwrap();
    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<ItemV1>().unwrap();

    let item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let txn = db.transaction().unwrap();
    txn.tables().insert(&txn, item).unwrap();
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();

    let item: ItemV1 = txn.tables().primary_get(&txn, b"1-test").unwrap().unwrap();
    assert_eq!(
        item,
        ItemV1 {
            id: 1,
            name: "test".to_string(),
        }
    );
    drop(txn);
    drop(db);

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<ItemV1>().unwrap();
    db.define::<ItemV2>().unwrap();

    db.migrate::<ItemV2>().unwrap();

    let txn = db.read_transaction().unwrap();
    let item: ItemV2 = txn.tables().primary_get(&txn, b"1-test").unwrap().unwrap();
    assert_eq!(
        item,
        ItemV2 {
            id: 1,
            name: "test".to_string(),
        }
    );

    let redb_stats = db.redb_stats().unwrap();
    assert_eq!(redb_stats.stats_tables.len(), 2);
    assert_eq!(redb_stats.stats_tables[0].name, "itemv1");
    assert_eq!(redb_stats.stats_tables[0].num_raw, 0);
    assert_eq!(redb_stats.stats_tables[1].name, "itemv2");
    assert_eq!(redb_stats.stats_tables[1].num_raw, 1);
}
