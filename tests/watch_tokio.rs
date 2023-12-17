#![cfg(feature = "tokio")]

use native_db::watch::Event;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemA {
    #[primary_key]
    id: u32,
}

#[tokio::test]
async fn watch_one_primary_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let mut db = builder.create(tf.path("test").as_std_path()).unwrap();

    let a = ItemA { id: 1 };

    let (mut recv, _) = db.watch().get().primary::<ItemA>(a.id).unwrap();

    let tx = db.rw_transaction().unwrap();
    tx.insert(a.clone()).unwrap();
    tx.commit().unwrap();

    for _ in 0..1 {
        let inner_event: ItemA = if let Event::Insert(event) = recv.recv().await.unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

// TODO: maybe do others tests but it should the same as a std::sync::mpsc::channel.
