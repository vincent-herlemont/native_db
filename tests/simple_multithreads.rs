#![cfg(not(target_arch = "wasm32"))]

use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use std::sync::Arc;
use std::thread;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
    name: String,
}

#[test]
fn multi_threads() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let db = Arc::new(db);

    thread::scope(|s| {
        let db_thread_1 = db.clone();
        let handle_thread_1 = s.spawn(move || {
            let item_a = Item {
                id: 1,
                name: "a".to_string(),
            };
            {
                let rw = db_thread_1.rw_transaction().unwrap();
                rw.insert(item_a).unwrap();
                rw.commit().unwrap();
            }
        });

        let db_thread_2 = db.clone();
        let handle_thread_2 = s.spawn(move || {
            let item_b = Item {
                id: 2,
                name: "b".to_string(),
            };
            {
                let rw = db_thread_2.rw_transaction().unwrap();
                rw.insert(item_b).unwrap();
                rw.commit().unwrap();
            }
        });

        handle_thread_1.join().unwrap();
        handle_thread_2.join().unwrap();
    });

    {
        let r = db.r_transaction().unwrap();
        let len = r.len().primary::<Item>().unwrap();
        assert_eq!(len, 2);

        let item_a: Item = r.get().primary(1u32).unwrap().unwrap();
        assert_eq!(item_a.name, "a".to_string());

        let item_b: Item = r.get().primary(2u32).unwrap().unwrap();
        assert_eq!(item_b.name, "b".to_string());
    }
}
