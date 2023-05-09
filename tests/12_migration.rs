mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct Av1(u32);

impl Av1 {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct Av2(String);

impl Av2 {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl From<Av1> for Av2 {
    fn from(av1: Av1) -> Self {
        Self(av1.0.to_string())
    }
}

#[test]
fn migration() {
    let tf = tests::init();

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.add_schema(Av1::struct_db_schema());
    db.add_schema(Av2::struct_db_schema());

    let a = Av1(42);

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
            .primary_get::<Av1>(&txn, &a.p_key())
            .unwrap()
            .unwrap();
        assert_eq!(a, a1);
    }
    txn.commit().unwrap();

    let (recv_av1, _id) = db.primary_watch::<Av1>(None).unwrap();
    let (recv_av2, _id) = db.primary_watch::<Av2>(None).unwrap();

    // Migrate
    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.migrate::<Av1, Av2>(&txn).unwrap();
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
        let len_av1 = tables.len::<Av1>(&txn).unwrap();
        assert_eq!(len_av1, 0);
        let len_av2 = tables.len::<Av2>(&txn).unwrap();
        assert_eq!(len_av2, 1);

        let a2 = tables
            .primary_get::<Av2>(&txn, "42".as_bytes())
            .unwrap()
            .unwrap();
        assert_eq!(a2, Av2("42".to_string()));
    }
}
