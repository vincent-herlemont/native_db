use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// TODO somehow test visibility of keys enum from a sibling/child crate?

/// Test struct to ensure `#[native_db(export_keys = true)]` compiles successfully
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(export_keys = true)]
struct Item {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
}

#[test]
fn test_insert_my_item() {
    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let key = item.native_db_primary_key();
    assert_eq!(key, 1_u32.to_key());

    let mut models: Models = Models::new();
    models.define::<Item>().unwrap();

    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item {
        id: 1,
        name: "test".to_string(),
    })
    .unwrap();
    rw.commit().unwrap();

    // Get primary key
    let r = db.r_transaction().unwrap();
    let result_item: Item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(result_item.id, 1);
    assert_eq!(result_item.name, "test");

    // Get secondary key
    let r = db.r_transaction().unwrap();
    let result_item: Vec<Item> = r
        .scan()
        .secondary(ItemKey::name)
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result_item.len(), 1);
    assert_eq!(result_item[0].id, 1);
    assert_eq!(result_item[0].name, "test");
}
