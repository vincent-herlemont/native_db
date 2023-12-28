#![cfg(not(target_arch = "wasm32"))]

use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use std::panic::AssertUnwindSafe;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_transaction_obj_1() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let rw = db.rw_transaction().unwrap();
    rw.insert(item).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(result.id, 1);
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct Item2 {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn test_transaction_obj_1_and_obj_2() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    builder.define::<Item2>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };
    let item_2 = Item2 {
        id: 2,
        name: "test".to_string(),
    };

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_1).unwrap();
    rw.insert(item_2).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(result.id, 1);
    let result: Item2 = r.get().primary(2u32).unwrap().unwrap();
    assert_eq!(result.id, 2);
}

#[allow(unreachable_code)]
#[test]
fn test_transaction_fail() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_1 = Item {
        id: 1,
        name: "test".to_string(),
    };

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_1).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Item = r.get().primary(1u32).unwrap().unwrap();
    assert_eq!(result.id, 1);

    let item_2 = Item {
        id: 2,
        name: "test".to_string(),
    };
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let rw = db.rw_transaction().unwrap();
        rw.insert(item_2).unwrap();
        panic!("Random panic here...")
    }));

    assert!(result.is_err());

    let r = db.r_transaction().unwrap();
    let result: Option<Item> = r.get().primary(2u32).unwrap();
    assert!(result.is_none());
}
