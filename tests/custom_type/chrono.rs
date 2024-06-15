use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u64,
    #[secondary_key(unique)]
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[test]
fn insert_get() {
    let item = Item {
        id: 1,
        timestamp: chrono::Utc::now(),
    };

    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new().create(&models, tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(ItemKey::timestamp, &item.timestamp)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
}
