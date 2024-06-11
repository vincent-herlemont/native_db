use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemV1 {
    #[primary_key]
    id: u32,
    name: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 2, from = ItemV1)]
#[native_db]
struct ItemV2 {
    #[primary_key]
    id: u32,
    name_v2: String,
}

impl From<ItemV1> for ItemV2 {
    fn from(item: ItemV1) -> Self {
        ItemV2 {
            id: item.id,
            name_v2: item.name,
        }
    }
}

impl From<ItemV2> for ItemV1 {
    fn from(item: ItemV2) -> Self {
        ItemV1 {
            id: item.id,
            name: item.name_v2,
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct Item2 {
    #[primary_key]
    id: u32,
    name2: String,
}

#[test]
fn test_migrate() {
    let tf = TmpFs::new().unwrap();
    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemV1>().unwrap();
    builder.define::<Item2>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_2 = Item2 {
        id: 1,
        name2: "test2".to_string(),
    };
    let rw_txn = db.rw_transaction().unwrap();
    rw_txn.insert(item_2).unwrap();
    rw_txn.commit().unwrap();

    let item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let rw_txn = db.rw_transaction().unwrap();
    rw_txn.insert(item).unwrap();
    rw_txn.commit().unwrap();

    let r_txn = db.r_transaction().unwrap();

    let item: ItemV1 = r_txn.get().primary(1).unwrap().unwrap();
    assert_eq!(
        item,
        ItemV1 {
            id: 1,
            name: "test".to_string(),
        }
    );
    drop(r_txn);
    drop(db);

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemV1>().unwrap();
    builder.define::<ItemV2>().unwrap();
    builder.define::<Item2>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.migrate::<ItemV2>().unwrap();
    rw.commit().unwrap();

    let r_txn = db.r_transaction().unwrap();
    let item: ItemV2 = r_txn.get().primary(1).unwrap().unwrap();
    assert_eq!(
        item,
        ItemV2 {
            id: 1,
            name_v2: "test".to_string(),
        }
    );

    let item: Item2 = r_txn.get().primary(1).unwrap().unwrap();
    assert_eq!(
        item,
        Item2 {
            id: 1,
            name2: "test2".to_string(),
        }
    );

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 3);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.primary_tables[1].name, "1_2_id");
    assert_eq!(stats.primary_tables[1].n_entries, Some(1));
    assert_eq!(stats.primary_tables[2].name, "2_1_id");
    assert_eq!(stats.primary_tables[2].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 0);
}
