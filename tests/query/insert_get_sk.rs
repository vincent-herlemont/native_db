use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(pk), secondary_key(gk_1, unique))]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn pk(&self) -> String {
        format!("{}", self.id)
    }

    pub fn gk_1(&self) -> String {
        format!("{}-{}", self.name, self.id)
    }
}

#[test]
fn insert_get_read_write_transaction() {
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

    let rw = db.rw_transaction().unwrap();
    let result_item = rw
        .get()
        .secondary(ItemKey::gk_1, "test-1")
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
    rw.commit().unwrap();
}

#[test]
fn insert_get_read_transaction() {
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

    let r = db.r_transaction().unwrap();
    let result_item = r.get().secondary(ItemKey::gk_1, "test-1").unwrap().unwrap();

    assert_eq!(item, result_item);
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemDuplicate {
    #[primary_key]
    id: u32,
    #[secondary_key(unique)]
    name: String,
}

#[test]
fn test_insert_duplicate_key() {
    let tf = TmpFs::new().unwrap();

    let item_1 = ItemDuplicate {
        id: 1,
        name: "test".to_string(),
    };
    let item_2 = ItemDuplicate {
        id: 2,
        name: "test".to_string(),
    };

    let mut models = Models::new();
    models.define::<ItemDuplicate>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_1).unwrap();
    let result = rw.insert(item_2);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        db_type::Error::DuplicateKey { .. }
    ));
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
fn test_insert_optional() {
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

    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(ItemOptionalKey::name, "test")
        .unwrap()
        .unwrap();
    assert_eq!(item_1, result_item);
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemScanRange {
    #[primary_key]
    id: u32,

    #[secondary_key]
    nr: u32,

    #[secondary_key(unique)]
    unique_nr: u32,
}

#[test]
fn test_scan_range() {
    let tf = TmpFs::new().unwrap();

    let item_1 = ItemScanRange { id: 1, nr: 1, unique_nr: 1 };
    let item_2 = ItemScanRange { id: 2, nr: 2, unique_nr: 2 };
    let item_3 = ItemScanRange { id: 3, nr: 2, unique_nr: 3 };
    let item_4 = ItemScanRange { id: 4, nr: 3, unique_nr: 4 };


    let mut models = Models::new();
    models.define::<ItemScanRange>().unwrap();
    let db = Builder::new()
        // .create(&models, tf.path("test").as_std_path())
        .create_in_memory(&models)
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_1.clone()).unwrap();
    rw.insert(item_2.clone()).unwrap();
    rw.insert(item_3.clone()).unwrap();
    rw.insert(item_4.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result = r.scan().secondary(ItemScanRangeKey::nr).unwrap()
        .range(0..10).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.nr).collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 2, 3], "range 0..10 for nr");

    let result = r.scan().secondary(ItemScanRangeKey::nr).unwrap()
        .range(2..3).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.nr).collect::<Vec<_>>();
    assert_eq!(result, vec![2, 2], "range 2..3 for nr");

    let result = r.scan().secondary(ItemScanRangeKey::unique_nr).unwrap()
        .range(1..3).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.unique_nr).collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2], "range 1..3 for unique_nr");

    let result = r.scan().secondary(ItemScanRangeKey::unique_nr).unwrap()
        .range(1..=3).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.unique_nr).collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3], "range 1..=3 for unique_nr");

    let result = r.scan().secondary(ItemScanRangeKey::unique_nr).unwrap()
        .range(3..=3).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.unique_nr).collect::<Vec<_>>();
    assert_eq!(result, vec![3], "range 3..=3 for unique_nr");

    let result = r.scan().secondary(ItemScanRangeKey::nr).unwrap()
        .range(2..=3).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.nr).collect::<Vec<_>>();
    assert_eq!(result, vec![2, 2, 3], "range 2..=3 for nr");

    let result = r.scan().secondary(ItemScanRangeKey::nr).unwrap()
        .range(2..=2).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.nr).collect::<Vec<_>>();
    assert_eq!(result, vec![2, 2], "range 2..=2 for nr");

    let result = r.scan().secondary(ItemScanRangeKey::nr).unwrap()
        .range(0..=2).collect::<Result<Vec<ItemScanRange>,_>>().unwrap().iter().map(|x| x.nr).collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 2], "range 0..=2 for nr");
}
