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
fn remove() {
    let tf = tests::init();

    let o = O(1);

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<O>();

    // Insert the item
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.insert(&tx, o.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check if the item is in the database
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: O = tables.primary_get(&tx_r, &o.p_key()).unwrap().unwrap();
        assert_eq!(o, o2);
    }

    // Remove the item
    let tx = db.transaction().unwrap();
    {
        let mut tables = tx.tables();
        tables.remove(&tx, o.clone()).unwrap();
    }
    tx.commit().unwrap();

    // Check if the item is not in the database
    let tx_r = db.read_transaction().unwrap();
    {
        let mut tables = tx_r.tables();
        let o2: Option<O> = tables.primary_get(&tx_r, &o.p_key()).unwrap();
        assert_eq!(o2, None);
    }
}
