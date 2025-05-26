use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_compact() {
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test");

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let mut db = Builder::new().create(&models, db_path.clone()).unwrap();

    // Insert 1000 items
    let rw = db.rw_transaction().unwrap();
    for i in 0..999 {
        rw.insert(Item {
            id: i,
            name: format!("test_{}", i),
        })
        .unwrap();
    }
    rw.commit().unwrap();

    // Check the size of the database
    let metadata = std::fs::metadata(db_path.clone()).unwrap();
    let file_size = metadata.len();
    assert_eq!(file_size, 1589248);
    dbg!(file_size);

    let out = db.compact().unwrap();
    assert!(out);

    // Check the size of the compacted database
    let metadata = std::fs::metadata(db_path.clone()).unwrap();
    let file_size = metadata.len();
    assert_eq!(file_size, 696320);
}
