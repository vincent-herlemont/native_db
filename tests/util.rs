use native_db::*;
use shortcut_assert_fs::TmpFs;

#[test]
fn test_builder() {
    let tf = TmpFs::new().unwrap();
    // Create without error
    let mut _db = DatabaseBuilder::new().create(&tf.path("test")).unwrap();
}

#[test]
fn test_builder_with_set_cache_size() {
    let tf = TmpFs::new().unwrap();
    // Create without error
    let mut builder = DatabaseBuilder::new();
    let _db = builder
        .set_cache_size(100)
        .create(&tf.path("test"))
        .unwrap();
}

#[test]
fn test_open_unexisting_database() {
    let tf = TmpFs::new().unwrap();
    // Open an unexisting database
    assert!(DatabaseBuilder::new().open(&tf.path("test")).is_err());
}

#[test]
fn test_open_existing_database() {
    let tf = TmpFs::new().unwrap();

    // Create a database
    let builder = DatabaseBuilder::new();
    let db = builder.create(&tf.path("test")).unwrap();
    drop(db);

    // Open an existing database
    let _db = DatabaseBuilder::new().open(&tf.path("test")).unwrap();
}

use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item1 {
    #[primary_key]
    id: u32,
    #[secondary_key(unique)]
    name: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct Item2 {
    #[primary_key]
    id: u32,
    #[secondary_key(optional)]
    id2: Option<u32>,
    #[secondary_key]
    name: String,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[test]
fn create_local_database_for_tests() {
    let root_project_path = env!("CARGO_MANIFEST_DIR");
    let tmp_data_dir_path = format!("{}/tests/data", root_project_path);

    std::fs::create_dir_all(tmp_data_dir_path.clone()).unwrap();

    let database_path = format!("{}/db_x_x_x", tmp_data_dir_path);

    if std::fs::metadata(&database_path).is_ok() {
        std::fs::remove_file(&database_path).unwrap();
    }

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item1>().unwrap();
    builder.define::<Item2>().unwrap();
    let db = builder.create(&database_path).unwrap();
    let rw = db.rw_transaction().unwrap();
    let item = Item1 {
        id: 1,
        name: "item1".to_string(),
    };

    // Genereate 1000 Item2 with random values
    for i in 0..1000 {
        let id2 = if i % 2 == 0 { Some(i) } else { None };
        let item = Item2 {
            id: i,
            id2,
            name: format!("item2_{}", i),
        };
        rw.insert(item).unwrap();
    }

    rw.insert(item).unwrap();
    rw.commit().unwrap();

    let ro = db.r_transaction().unwrap();
    let len = ro.len().primary::<Item1>().unwrap();
    assert_eq!(len, 1);

    let len = ro.len().primary::<Item2>().unwrap();
    assert_eq!(len, 1000);
}
