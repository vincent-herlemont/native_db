#![cfg(not(feature = "native_model"))]
mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;
use struct_db_macro::struct_db;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    #[allow(dead_code)]
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn my_item_bincode_encode_to_vec() {
    let my_item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let encoded = my_item.struct_db_bincode_encode_to_vec();
    let decoded: (Item, _) =
        bincode::serde::decode_from_slice(encoded.as_slice(), bincode::config::standard()).unwrap();

    assert_eq!(my_item, decoded.0);
}

#[test]
fn my_item_bincode_decode_from_slice() {
    tests::init();

    let my_item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let encoded = my_item.struct_db_bincode_encode_to_vec();
    let decoded: Item = Item::struct_db_bincode_decode_from_slice(encoded.as_slice());

    assert_eq!(my_item, decoded);
}
