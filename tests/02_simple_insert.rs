#![cfg(not(feature = "native_model"))]
mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn test_insert_my_item() {
    let tf = tests::init();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    txn.tables().insert(&txn, item).unwrap();
}
