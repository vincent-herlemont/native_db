#![cfg(not(feature = "async_tokio"))]
mod tests;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use struct_db::watch::Event;
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct A(u32);

impl A {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct B(u32);

impl B {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

const TIMEOUT: Duration = Duration::from_secs(1);

#[test]
fn watch_one_primary_key() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let a = A(1);

    let (recv, _) = db.primary_watch::<A>(Some(&a.p_key())).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..1 {
        let inner_event: A = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_primary_key() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let a1 = A(1);
    let a2 = A(2);

    let (recv, _) = db.primary_watch::<A>(None).unwrap();
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a1.clone()).unwrap();
        tables.insert(&tx, a2.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..2 {
        let inner_event: A = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == a1 || inner_event == a2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_multithreading() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let db = Arc::new(db);
    let dba = Arc::clone(&db);

    let a = A(1);
    let (recv, _) = dba.primary_watch::<A>(Some(&a.p_key())).unwrap();

    let handle = std::thread::spawn(move || {
        let a = A(1);
        let (recv, _) = dba.primary_watch::<A>(Some(&a.p_key())).unwrap();
        let tx = dba.transaction().unwrap();
        {
            let a = A(1);
            let mut tables = tx.tables();
            tables.insert(&tx, a.clone()).unwrap();
        }
        tx.commit().unwrap();
        for _ in 0..1 {
            let inner_event: A = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
                event.inner()
            } else {
                panic!("wrong event")
            };
            assert_eq!(inner_event, a);
        }
    });

    let dbb = Arc::clone(&db);
    let tx = dbb.transaction().unwrap();
    {
        let a = A(1);
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    handle.join().unwrap();
    for _ in 0..2 {
        let inner_event: A = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_outside() {
    let tf = tests::init();

    let a = A(1);
    let b1 = B(1);
    let b2 = B(2);

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();
    db.define::<B>();

    let (recv, _) = db.primary_watch::<B>(Some(&a.p_key())).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
        tables.insert(&tx, b1.clone()).unwrap();
        tables.insert(&tx, b2.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check that recv receives only 1 insert event
    let inner_event: B = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
        event.inner()
    } else {
        panic!("wrong event")
    };
    assert_eq!(inner_event, b1);
    assert!(recv.try_recv().is_err());
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key), fn_secondary_key(s_key))]
struct A1K(u32, String);

impl A1K {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    pub fn s_key(&self) -> Vec<u8> {
        self.1.as_bytes().to_vec()
    }
}

#[test]
fn watch_one_secondary_key() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A1K>();

    let a = A1K(1, "a".to_string());

    let (recv, _) = db
        .secondary_watch::<A1K>(A1KKey::s_key, Some(&a.s_key()))
        .unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..1 {
        let inner_event: A1K = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_secondary_keys() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A1K>();

    let a1 = A1K(1, "a".to_string());
    let a2 = A1K(2, "b".to_string());

    let (recv, _) = db.secondary_watch::<A1K>(A1KKey::s_key, None).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a1.clone()).unwrap();
        tables.insert(&tx, a2.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..2 {
        let inner_event: A1K = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == a1 || inner_event == a2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn unwatch() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let a = A(1);

    let (recv, recv_id) = db.primary_watch::<A>(Some(&a.p_key())).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..1 {
        let inner_event: A = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }

    db.unwatch(recv_id).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();
    assert!(recv.try_recv().is_err());
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct C(String);

impl C {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

#[test]
fn watch_start_with() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<C>();

    let c1 = C("a_1".to_string());
    let c2 = C("a_2".to_string());
    let c3 = C("b_1".to_string());

    let (recv, _) = db
        .primary_watch_start_with::<C>(&"a".as_bytes().to_vec())
        .unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, c1.clone()).unwrap();
        tables.insert(&tx, c2.clone()).unwrap();
        tables.insert(&tx, c3.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..2 {
        let inner_event: C = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == c1 || inner_event == c2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_start_with_by_key() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A1K>();

    let a1 = A1K(1, "a_1".to_string());
    let a2 = A1K(2, "a_2".to_string());
    let a3 = A1K(3, "b_1".to_string());

    let (recv, _) = db
        .secondary_watch_start_with::<A1K>(A1KKey::s_key, &"a".as_bytes().to_vec())
        .unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a1.clone()).unwrap();
        tables.insert(&tx, a2.clone()).unwrap();
        tables.insert(&tx, a3.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..2 {
        let inner_event: A1K = if let Event::Insert(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert!(inner_event == a1 || inner_event == a2);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_delete() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let a = A(1);

    let (recv, _) = db.primary_watch::<A>(None).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    recv.recv_timeout(TIMEOUT).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.remove(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..1 {
        let r_a: A = if let Event::Delete(event) = recv.recv_timeout(TIMEOUT).unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(r_a, a);
    }
    assert!(recv.try_recv().is_err());
}

#[test]
fn watch_all_update() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let a1 = A(1);
    let a2 = A(2);

    let (recv, _) = db.primary_watch::<A>(None).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a1.clone()).unwrap();
    }
    tx.commit().unwrap();

    recv.recv_timeout(TIMEOUT).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.update(&tx, a1.clone(), a2.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..1 {
        let (old_r_a, new_r_a): (A, A) =
            if let Event::Update(event) = recv.recv_timeout(TIMEOUT).unwrap() {
                (event.inner_old(), event.inner_new())
            } else {
                panic!("wrong event")
            };
        assert_eq!(old_r_a, a1);
        assert_eq!(new_r_a, a2);
    }
    assert!(recv.try_recv().is_err());
}
