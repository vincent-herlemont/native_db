mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;

type Item = ItemV1;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct ItemV0(u32);

impl ItemV0 {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct ItemV1(String);

impl ItemV1 {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
    pub fn p_key(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl From<ItemV0> for ItemV1 {
    fn from(av1: ItemV0) -> Self {
        Self(av1.0.to_string())
    }
}

#[test]
fn migration() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<ItemV0>();
    db.define::<ItemV1>();

    let a = ItemV0(42);

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, a.clone()).unwrap();
    }
    txn.commit().unwrap();

    // Check if a is saved
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let a1 = tables
            .primary_get::<ItemV0>(&txn, &a.p_key())
            .unwrap()
            .unwrap();
        assert_eq!(a, a1);
    }
    txn.commit().unwrap();

    #[cfg(not(feature = "tokio"))]
    let (recv_av1, _id) = db.primary_watch::<ItemV0>(None).unwrap();
    #[cfg(not(feature = "tokio"))]
    let (recv_av2, _id) = db.primary_watch::<ItemV1>(None).unwrap();

    #[cfg(feature = "tokio")]
    let (mut recv_av1, _id) = db.primary_watch::<ItemV0>(None).unwrap();
    #[cfg(feature = "tokio")]
    let (mut recv_av2, _id) = db.primary_watch::<ItemV0>(None).unwrap();

    // Migrate
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.migrate::<ItemV0, ItemV1>(&txn).unwrap();
    }
    txn.commit().unwrap();

    // Check is there is no event from AV1
    assert!(recv_av1.try_recv().is_err());
    // Check is there is no event from AV2
    assert!(recv_av2.try_recv().is_err());

    // Check migration
    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let len_av1 = tables.len::<ItemV0>(&txn).unwrap();
        assert_eq!(len_av1, 0);
        let len_av2 = tables.len::<Item>(&txn).unwrap();
        assert_eq!(len_av2, 1);

        let a2 = tables
            .primary_get::<Item>(&txn, "42".as_bytes())
            .unwrap()
            .unwrap();
        assert_eq!(a2, Item::new("42"));
    }
}
