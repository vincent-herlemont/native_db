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
    name: String,
}

#[test]
fn auto_update_non_existent() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    let result = rw.auto_update(Item {
        id: 1,
        name: "test".to_string(),
    });
    assert!(result.unwrap().is_none());
    rw.commit().unwrap();
}

#[test]
fn auto_update_existing() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let initial_item = Item {
        id: 1,
        name: "initial".to_string(),
    };
    let updated_item = Item {
        id: 1,
        name: "updated".to_string(),
    };

    // Insert initial item
    let rw = db.rw_transaction().unwrap();
    rw.insert(initial_item.clone()).unwrap();
    rw.commit().unwrap();

    // Update the item
    let rw = db.rw_transaction().unwrap();
    let old_value = rw.auto_update(updated_item.clone()).unwrap();
    assert!(old_value.is_some());
    assert_eq!(old_value.unwrap(), initial_item);

    // Verify the update
    let current: Item = rw.get().primary(1u32).unwrap().unwrap();
    assert_eq!(current, updated_item);
    rw.commit().unwrap();
}

#[test]
fn auto_update_multiple() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    // Insert multiple items
    let rw = db.rw_transaction().unwrap();
    for i in 1..=3 {
        rw.insert(Item {
            id: i,
            name: format!("item{}", i),
        })
        .unwrap();
    }
    rw.commit().unwrap();

    // Update middle item
    let rw = db.rw_transaction().unwrap();
    let old_value = rw
        .auto_update(Item {
            id: 2,
            name: "updated".to_string(),
        })
        .unwrap();
    assert!(old_value.is_some());
    assert_eq!(
        old_value.unwrap(),
        Item {
            id: 2,
            name: "item2".to_string()
        }
    );

    // Verify other items unchanged
    let item1: Item = rw.get().primary(1u32).unwrap().unwrap();
    assert_eq!(item1.name, "item1");
    let item2: Item = rw.get().primary(2u32).unwrap().unwrap();
    assert_eq!(item2.name, "updated");
    let item3: Item = rw.get().primary(3u32).unwrap().unwrap();
    assert_eq!(item3.name, "item3");
    rw.commit().unwrap();
}
