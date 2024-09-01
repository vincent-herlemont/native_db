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
fn test_check_integrity() {
    let tf = TmpFs::new().unwrap();
    let db_path = tf.path("test");

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let mut db = Builder::new().create(&models, db_path.clone()).unwrap();

    // Insert 1 item
    let rw = db.rw_transaction().unwrap();
    rw.insert(Item {
        id: 1,
        name: "test".to_string(),
    })
    .unwrap();
    rw.commit().unwrap();

    let out = db.check_integrity().unwrap();
    assert!(out);
}
