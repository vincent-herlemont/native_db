use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(generate_my_primary_key))]
struct ItemV1 {
    id: u32,
    name: String,
}

impl ItemV1 {
    #[allow(dead_code)]
    pub fn generate_my_primary_key(&self) -> String {
        format!("{}-{}", self.id, self.name)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 2, from = ItemV1)]
#[native_db(primary_key(generate_my_primary_key))]
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
    pub fn generate_my_primary_key(&self) -> String {
        format!("{}-{}", self.id, self.name)
    }
}

#[test]
fn test_migrate() {
    let tf = TmpFs::new().unwrap();
    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemV1>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let rw_txn = db.rw_transaction().unwrap();
    rw_txn.insert(item).unwrap();
    rw_txn.commit().unwrap();

    let r_txn = db.r_transaction().unwrap();

    let item: ItemV1 = r_txn.get().primary("1-test").unwrap().unwrap();
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
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.migrate::<ItemV2>().unwrap();
    rw.commit().unwrap();

    let r_txn = db.r_transaction().unwrap();
    let item: ItemV2 = r_txn.get().primary("1-test").unwrap().unwrap();
    assert_eq!(
        item,
        ItemV2 {
            id: 1,
            name: "test".to_string(),
        }
    );

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 2);
    assert_eq!(stats.primary_tables[0].name, "1_1_generate_my_primary_key");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.primary_tables[1].name, "1_2_generate_my_primary_key");
    assert_eq!(stats.primary_tables[1].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 0);
}
