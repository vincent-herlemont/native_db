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
    #[secondary_key]
    name: String,
    #[secondary_key(unique)]
    code: String,
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
        code: "A1".to_string(),
    });
    assert!(result.unwrap().is_none());
    rw.commit().unwrap();
}

#[test]
fn auto_update_existing_check_secondary_keys() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let initial_item = Item {
        id: 1,
        name: "initial".to_string(),
        code: "A1".to_string(),
    };
    let updated_item = Item {
        id: 1,
        name: "updated".to_string(),
        code: "B1".to_string(),
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

    // Verify primary key lookup
    let current: Item = rw.get().primary(1u32).unwrap().unwrap();
    assert_eq!(current, updated_item);

    // Verify secondary key lookup (non-unique)
    let by_name: Vec<Item> = rw
        .scan()
        .secondary(ItemKey::name)
        .unwrap()
        .all()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0], updated_item);

    // Verify secondary key lookup (unique)
    let by_code: Item = rw
        .get()
        .secondary(ItemKey::code, "B1".to_string())
        .unwrap()
        .unwrap();
    assert_eq!(by_code, updated_item);

    // Old secondary keys should not exist
    let old_by_code: Option<Item> = rw.get().secondary(ItemKey::code, "A1".to_string()).unwrap();
    assert!(old_by_code.is_none());

    rw.commit().unwrap();
}

#[test]
fn auto_update_multiple_with_secondary_keys() {
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
            code: format!("A{}", i),
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
            code: "B2".to_string(),
        })
        .unwrap();
    assert!(old_value.is_some());
    assert_eq!(
        old_value.unwrap(),
        Item {
            id: 2,
            name: "item2".to_string(),
            code: "A2".to_string(),
        }
    );

    // Verify all items by primary key
    let item1: Item = rw.get().primary(1u32).unwrap().unwrap();
    assert_eq!(item1.name, "item1");
    assert_eq!(item1.code, "A1");

    let item2: Item = rw.get().primary(2u32).unwrap().unwrap();
    assert_eq!(item2.name, "updated");
    assert_eq!(item2.code, "B2");

    let item3: Item = rw.get().primary(3u32).unwrap().unwrap();
    assert_eq!(item3.name, "item3");
    assert_eq!(item3.code, "A3");

    // Verify secondary key updates
    let by_code2: Item = rw
        .get()
        .secondary(ItemKey::code, "B2".to_string())
        .unwrap()
        .unwrap();
    assert_eq!(by_code2, item2);

    // Old secondary key should not exist
    let old_by_code2: Option<Item> = rw.get().secondary(ItemKey::code, "A2".to_string()).unwrap();
    assert!(old_by_code2.is_none());

    rw.commit().unwrap();
}
