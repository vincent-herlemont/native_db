mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key))]
struct O(u32);

impl O {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

#[test]
fn update() {
    let tf = tests::init();

    let o_v1 = O(1);

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.add_schema(O::struct_db_schema());

    // Insert the item
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, o_v1.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check if the item is in the database
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: O = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap().unwrap();
        assert_eq!(o_v1, o2);
    }

    let o_v2 = O(2);

    // Update the item
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.update(&tx, o_v1.clone(), o_v2.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check if the item v1 is not in the database
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<O> = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap();
        assert_eq!(o2, None);
    }
    // Check if the item v2 is in the database
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: O = tables.primary_get(&tx_r, &o_v2.p_key()).unwrap().unwrap();
        assert_eq!(o_v2, o2);
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(p_key), fn_secondary_key(s_key))]
struct O1K(u32, String);

impl O1K {
    pub fn p_key(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    pub fn s_key(&self) -> Vec<u8> {
        self.1.as_bytes().to_vec()
    }
}

#[test]
fn update_1k() {
    let tf = tests::init();

    let o_v1 = O1K(1, "1".to_string());

    let mut db = Db::init(tf.path("test").as_std_path()).unwrap();

    db.add_schema(O1K::struct_db_schema());

    // Insert the item
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, o_v1.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check if the item is in the database by primary key
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: O1K = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap().unwrap();
        assert_eq!(o_v1, o2);
    }
    // Check if the item is in the database by secondary key
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: O1K = tables
            .secondary_get(&tx_r, O1KKey::s_key, &o_v1.s_key())
            .unwrap()
            .unwrap();
        assert_eq!(o_v1, o2);
    }

    let o_v2 = O1K(2, "2".to_string());

    // Update the item
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.update(&tx, o_v1.clone(), o_v2.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check if the item v1 is not in the database by primary key
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<O1K> = tables.primary_get(&tx_r, &o_v1.p_key()).unwrap();
        assert_eq!(o2, None);
    }
    // Check if the item v1 is not in the database by secondary key
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<O1K> = tables
            .secondary_get(&tx_r, O1KKey::s_key, &o_v1.s_key())
            .unwrap();
        assert_eq!(o2, None);
    }

    // Check if the item v2 is in the database
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: O1K = tables.primary_get(&tx_r, &o_v2.p_key()).unwrap().unwrap();
        assert_eq!(o_v2, o2);
    }
}
