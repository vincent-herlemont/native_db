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
    tests::init();

    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let key: Vec<u8> = item.struct_db_primary_key();
    assert_eq!(key, "1-test".as_bytes());
}
