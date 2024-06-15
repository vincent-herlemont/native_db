use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    uuid: Uuid,
}

#[test]
fn insert_get() {
    let item = Item {
        uuid: Uuid::new_v4(),
    };

    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new().create(&models, tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result_item = r.get().primary(&item.uuid).unwrap().unwrap();
    assert_eq!(item, result_item);
}
