#![cfg(not(feature = "use_native_model"))]
#![cfg(feature = "tokio")]
mod tests;

use serde::{Deserialize, Serialize};
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

#[tokio::test]
async fn watch_one_primary_key() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<A>();

    let a = A(1);

    let (mut recv, _) = db.primary_watch::<A>(Some(&a.p_key())).unwrap();

    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, a.clone()).unwrap();
    }
    tx.commit().unwrap();

    for _ in 0..1 {
        let inner_event: A = if let Event::Insert(event) = recv.recv().await.unwrap() {
            event.inner()
        } else {
            panic!("wrong event")
        };
        assert_eq!(inner_event, a);
    }
    assert!(recv.try_recv().is_err());
}

// TODO: maybe do others tests but it should the same as a std::sync::mpsc::channel.