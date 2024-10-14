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

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

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

#[test]
fn test_remove_with_invalid_secondary_key() {
    let tf = TmpFs::new().unwrap();

    let mut item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

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

    item.name = "test2".to_string();
    let rw = db.rw_transaction().unwrap();
    let r = rw.remove(item.clone());
    assert!(r.is_err());
    assert!(matches!(
        r.unwrap_err(),
        db_type::Error::IncorrectInputData { .. }
    ));
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));
}

#[test]
fn test_remove_with_invalid_secondary_key_without_catch_error() {
    let tf = TmpFs::new().unwrap();

    let item1 = Item {
        id: 1,
        name: "test".to_string(),
    };
    let mut item2 = Item {
        id: 2,
        name: "test2".to_string(),
    };

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item1.clone()).unwrap();
    rw.insert(item2.clone()).unwrap();
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(2));
    assert_eq!(stats.secondary_tables.len(), 1);

    item2.name = "test3".to_string();
    let rw = db.rw_transaction().unwrap();
    rw.remove(item1.clone()).unwrap();
    let r = rw.remove(item2.clone());
    assert!(r.is_err());
    assert!(matches!(
        r.unwrap_err(),
        db_type::Error::IncorrectInputData { .. }
    ));
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));

    // Get item2
    let r = db.r_transaction().unwrap();
    let item2: Item = r.get().primary(2u32).unwrap().unwrap();
    assert_eq!(item2.id, 2);
    assert_eq!(item2.name, "test2");
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

    let mut models = Models::new();
    models.define::<ItemOptional>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

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

#[test]
fn insert_remove_unique_optional_with_invalid_secondary_key() {
    let tf = TmpFs::new().unwrap();

    let mut item = ItemOptional {
        id: 1,
        name: Some("test".to_string()),
    };

    let mut models = Models::new();
    models.define::<ItemOptional>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

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

    item.name = None;
    let rw = db.rw_transaction().unwrap();
    let result = rw.remove(item.clone());
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        db_type::Error::IncorrectInputData { .. }
    ));
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id");
    assert_eq!(stats.primary_tables[0].n_entries, Some(1));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(1));
}
