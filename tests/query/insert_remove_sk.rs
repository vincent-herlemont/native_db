use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
}

#[test]
fn insert_remove() {
    let tf = TmpFs::new().unwrap();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));

    let rw = db.rw_transaction().unwrap();
    let old_value = rw.remove(item.clone()).unwrap();
    assert_eq!(old_value, item);
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(0));
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemOptional {
    #[primary_key]
    id: u32,
    #[secondary_key(unique, optional)]
    name: Option<String>,
}
#[test]
fn insert_remove_unique_optional() {
    let tf = TmpFs::new().unwrap();

    let item_1 = ItemOptional {
        id: 1,
        name: Some("test".to_string()),
    };
    let item_2 = ItemOptional { id: 2, name: None };

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemOptional>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_1.clone()).unwrap();
    rw.insert(item_2.clone()).unwrap();
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(2));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));

    let rw = db.rw_transaction().unwrap();
    let old_value = rw.remove(item_1.clone()).unwrap();
    assert_eq!(old_value, item_1);
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(0));

    let rw = db.rw_transaction().unwrap();
    let old_value = rw.remove(item_2.clone()).unwrap();
    assert_eq!(old_value, item_2);
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(0));
}
