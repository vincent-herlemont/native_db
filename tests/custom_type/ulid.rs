use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct UlidItem {
    #[primary_key]
    ulid: Ulid,
}

#[test]
fn insert_get_borrowed_ulid() {
    let item = UlidItem { ulid: Ulid::new() };
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<UlidItem>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test_borrowed_ulid").as_std_path())
        .unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();
    let r = db.r_transaction().unwrap();
    let result_item = r.get().primary(&item.ulid).unwrap().unwrap();
    assert_eq!(item, result_item);
}

#[test]
fn insert_get_owned_ulid() {
    let item = UlidItem { ulid: Ulid::new() };
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<UlidItem>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test_owned_ulid").as_std_path())
        .unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();
    let r = db.r_transaction().unwrap();
    let result_item = r.get().primary(item.ulid).unwrap().unwrap();
    assert_eq!(item, result_item);
}
