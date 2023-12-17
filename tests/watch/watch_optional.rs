use native_db::watch::Event;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemAOptional {
    #[primary_key]
    id: u32,
    #[secondary_key(unique, optional)]
    name: Option<String>,
}

#[test]
fn watch_one_secondary_key_some() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemAOptional>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let a = ItemAOptional {
        id: 1,
        name: Some("a".to_string()),
    };

    let (recv, _) = db
        .watch()
        .get()
        .secondary::<ItemAOptional>(ItemAOptionalKey::name, "a")
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(a.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let inner_event: ItemAOptional =
            if let Event::Insert(event) = recv.recv_timeout(super::TIMEOUT).unwrap() {
                event.inner()
            } else {
                panic!("wrong event")
            };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_one_secondary_key_none() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemAOptional>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let a = ItemAOptional { id: 1, name: None };

    let (recv, _) = db
        .watch()
        .get()
        .secondary::<ItemAOptional>(ItemAOptionalKey::name, "a")
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(a.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let result = recv.recv_timeout(super::TIMEOUT);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            std::sync::mpsc::RecvTimeoutError::Timeout
        ));
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_start_with_by_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemAOptional>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a_1_k = ItemAOptional {
        id: 1,
        name: Some("a_1".to_string()),
    };
    let item_a_2_k = ItemAOptional {
        id: 2,
        name: Some("a_2".to_string()),
    };
    let item_a_3_k = ItemAOptional {
        id: 3,
        name: Some("b_1".to_string()),
    };

    let (recv, _) = db
        .watch()
        .scan()
        .secondary(ItemAOptionalKey::name)
        .start_with::<ItemAOptional>("a")
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a_1_k.clone()).unwrap();
    rw.insert(item_a_2_k.clone()).unwrap();
    rw.insert(item_a_3_k.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..2 {
        let inner_event: ItemAOptional =
            if let Event::Insert(event) = recv.recv_timeout(super::TIMEOUT).unwrap() {
                event.inner()
            } else {
                panic!("wrong event")
            };
        assert!(inner_event == item_a_1_k || inner_event == item_a_2_k);
    }
    assert!(recv.try_recv().is_err());
}
