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
    #[secondary_key(optional)]
    name: Option<String>,
}

#[test]
fn insert_len_read_transaction() {
    let tf = TmpFs::new().unwrap();

    let item = Item { id: 1, name: None };

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result_item = r.len().secondary::<Item>(ItemKey::name).unwrap();
    assert_eq!(0, result_item);

    let item = Item {
        id: 2,
        name: Some("test".to_string()),
    };

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result_item = r.len().secondary::<Item>(ItemKey::name).unwrap();
    assert_eq!(1, result_item);
}
