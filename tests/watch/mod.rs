#![cfg(not(feature = "tokio"))]

mod watch_optional;

use native_db::watch::Event;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemA {
    #[primary_key]
    id: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db]
struct ItemB {
    #[primary_key]
    id: u32,
}

#[test]
fn watch_one_primary_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a = ItemA { id: 1 };

    let (recv, _) = db.watch().get().primary::<ItemA>(item_a.id).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let inner_event: ItemA = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, item_a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_primary_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a_1 = ItemA { id: 1 };
    let item_a_2 = ItemA { id: 2 };

    let (recv, _) = db.watch().scan().primary().all::<ItemA>().unwrap();
    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a_1.clone()).unwrap();
    rw.insert(item_a_2.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..2 {
        let inner_event: ItemA = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == item_a_1 || inner_event == item_a_2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_multithreading() {
    let tf = TmpFs::new().unwrap();

    let mut builder: DatabaseBuilder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();
    let db = Arc::new(db);
    let dba = Arc::clone(&db);

    let item_a = ItemA { id: 1 };
    let (recv, _) = dba.watch().get().primary::<ItemA>(item_a.id).unwrap();

    thread::scope(|s| {
        let handle = s.spawn(move || {
            let item_a = ItemA { id: 1 };
            let (recv, _) = dba.watch().get().primary::<ItemA>(item_a.id).unwrap();
            let rw = dba.rw_transaction().unwrap();
            {
                let item_a = ItemA { id: 1 };
                rw.insert(item_a.clone()).unwrap();
            }
            rw.commit().unwrap();
            for _ in 0..1 {
                let inner_event: ItemA =
                    if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
                        event.inner().unwrap()
                    } else {
                        panic!("wrong event")
                    };
                assert_eq!(inner_event, item_a);
            }
        });

        let dbb = Arc::clone(&db);
        let rw = dbb.rw_transaction().unwrap();
        {
            let item_a = ItemA { id: 1 };
            rw.insert(item_a.clone()).unwrap();
        }
        rw.commit().unwrap();

        handle.join().unwrap();
        for _ in 0..2 {
            let inner_event: ItemA =
                if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
                    event.inner().unwrap()
                } else {
                    panic!("wrong event")
                };
            assert_eq!(inner_event, item_a);
        }
        assert!(recv.try_recv().is_err());
    });
}

#[test]
fn watch_outside() {
    let tf = TmpFs::new().unwrap();

    let item_a = ItemA { id: 1 };
    let item_b_1 = ItemB { id: 1 };
    let item_b_2 = ItemB { id: 2 };

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    builder.define::<ItemB>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let (recv, _) = db.watch().get().primary::<ItemB>(item_b_1.id).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    rw.insert(item_b_1.clone()).unwrap();
    rw.insert(item_b_2.clone()).unwrap();
    rw.commit().unwrap();

    // Check that recv receives only 1 insert event
    let inner_event: ItemB = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
        event.inner().unwrap()
    } else {
        panic!("wrong event")
    };
    assert_eq!(inner_event, item_b_1);
    assert!(recv.try_recv().is_err());
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
struct ItemA1K {
    #[primary_key]
    id: u32,
    #[secondary_key(unique)]
    name: String,
}

#[test]
fn watch_one_secondary_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA1K>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let a = ItemA1K {
        id: 1,
        name: "a".to_string(),
    };

    let (recv, _) = db
        .watch()
        .get()
        .secondary::<ItemA1K>(ItemA1KKey::name, &a.name)
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(a.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let inner_event: ItemA1K = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap()
        {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_secondary_keys() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA1K>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let a1 = ItemA1K {
        id: 1,
        name: "a".to_string(),
    };
    let a2 = ItemA1K {
        id: 2,
        name: "b".to_string(),
    };

    let (recv, _) = db
        .watch()
        .scan()
        .secondary(ItemA1KKey::name)
        .all::<ItemA1K>()
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(a1.clone()).unwrap();
    rw.insert(a2.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..2 {
        let inner_event: ItemA1K = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap()
        {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == a1 || inner_event == a2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn unwatch() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a = ItemA { id: 1 };

    let (recv, recv_id) = db.watch().get().primary::<ItemA>(item_a.id).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let inner_event: ItemA = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, item_a);
    }

    assert!(db.unwatch(recv_id).unwrap());

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    rw.commit().unwrap();
    assert!(recv.try_recv().is_err());
}

#[test]
fn unwatch_by_deleted_receiver() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a = ItemA { id: 1 };

    let (recv, recv_id) = db.watch().get().primary::<ItemA>(item_a.id).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let inner_event: ItemA = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, item_a);
    }

    drop(recv);

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    // The watcher is removed during the commit because the receiver is dropped
    rw.commit().unwrap();

    // Check if the watcher is removed
    assert!(!db.unwatch(recv_id).unwrap());
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 4, version = 1)]
#[native_db]
struct ItemC {
    #[primary_key]
    name: String,
}

#[test]
fn watch_start_with() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemC>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let c1 = ItemC {
        name: "a_1".to_string(),
    };
    let c2 = ItemC {
        name: "a_2".to_string(),
    };
    let c3 = ItemC {
        name: "b_1".to_string(),
    };

    let (recv, _) = db
        .watch()
        .scan()
        .primary()
        .start_with::<ItemC>("a")
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(c1.clone()).unwrap();
    rw.insert(c2.clone()).unwrap();
    rw.insert(c3.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..2 {
        let inner_event: ItemC = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == c1 || inner_event == c2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_start_with_by_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA1K>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a_1_k = ItemA1K {
        id: 1,
        name: "a_1".to_string(),
    };
    let item_a_2_k = ItemA1K {
        id: 2,
        name: "a_2".to_string(),
    };
    let item_a_3_k = ItemA1K {
        id: 3,
        name: "b_1".to_string(),
    };

    let (recv, _) = db
        .watch()
        .scan()
        .secondary(ItemA1KKey::name)
        .start_with::<ItemA1K>("a")
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a_1_k.clone()).unwrap();
    rw.insert(item_a_2_k.clone()).unwrap();
    rw.insert(item_a_3_k.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..2 {
        let inner_event: ItemA1K = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap()
        {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == item_a_1_k || inner_event == item_a_2_k);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_delete() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a = ItemA { id: 1 };

    let (recv, _) = db.watch().scan().primary().all::<ItemA>().unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a.clone()).unwrap();
    rw.commit().unwrap();

    recv.recv_timeout(TIMEOUT).unwrap();

    let rw = db.rw_transaction().unwrap();
    let old = rw.remove(item_a.clone()).unwrap();
    assert_eq!(old, item_a);
    rw.commit().unwrap();

    for _ in 0..1 {
        let r_a: ItemA = if let Event::Delete(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner().unwrap()
        } else {
            panic!("wrong event")
        };
        assert_eq!(r_a, item_a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_update() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemA>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let item_a_1 = ItemA { id: 1 };
    let item_a_2 = ItemA { id: 2 };

    let (recv, _) = db.watch().scan().primary().all::<ItemA>().unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_a_1.clone()).unwrap();
    rw.commit().unwrap();

    recv.recv_timeout(TIMEOUT).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.update(item_a_1.clone(), item_a_2.clone()).unwrap();
    rw.commit().unwrap();

    for _ in 0..1 {
        let (old_r_a, new_r_a): (ItemA, ItemA) =
            if let Event::Update(event) = recv.recv_timeout(TIMEOUT).unwrap() {
                (event.inner_old().unwrap(), event.inner_new().unwrap())
            } else {
                panic!("wrong event")
            };
        assert_eq!(old_r_a, item_a_1);
        assert_eq!(new_r_a, item_a_2);
    }
    assert!(recv.try_recv().is_err());
}
