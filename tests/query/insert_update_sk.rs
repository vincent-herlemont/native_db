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
    #[secondary_key(unique)]
    name: String,
}

#[test]
fn insert_update_sk() {
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

    // Insert the item
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    // Check if the item is in the database by primary key
    let r = db.r_transaction().unwrap();
    let item2: Item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(item, item2);

    // Check if the item is in the database by secondary key
    let r = db.r_transaction().unwrap();
    let item2: Item = r.get().secondary(ItemKey::name, "test").unwrap().unwrap();
    assert_eq!(item, item2);

    let item_v2 = Item {
        id: 2,
        name: "test2".to_string(),
    };

    // Update the item
    let rw = db.rw_transaction().unwrap();
    rw.update(item.clone(), item_v2.clone()).unwrap();
    rw.commit().unwrap();

    // Check if the item v1 is not in the database by primary key
    let r = db.r_transaction().unwrap();
    let item2: Option<Item> = r.get().primary(1u32).unwrap();
    assert_eq!(item2, None);

    // Check if the item v1 is not in the database by secondary key
    let r = db.r_transaction().unwrap();
    let item2: Option<Item> = r.get().secondary(ItemKey::name, "test").unwrap();
    assert_eq!(item2, None);

    // Check if the item v2 is in the database by primary key
    let r = db.r_transaction().unwrap();
    let item2: Item = r.get().secondary(ItemKey::name, "test2").unwrap().unwrap();
    assert_eq!(item2, item_v2);

    // Check if the item v2 is in the database by secondary key
    let r = db.r_transaction().unwrap();
    let item2: Item = r.get().primary(2u32).unwrap().unwrap();
    assert_eq!(item2, item_v2);

    // Check length is 1
    let r = db.r_transaction().unwrap();
    let length = r.len().primary::<Item>().unwrap();
    assert_eq!(length, 1);
}
