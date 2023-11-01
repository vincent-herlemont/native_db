#![cfg(not(feature = "native_model"))]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use struct_db::SDBItem;
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[struct_db(
    pk = generate_my_primary_key,
    gk = generate_my_secondary_key
)]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
    pub fn generate_my_secondary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.name, self.id).into()
    }
}

#[test]
fn test_insert_my_item() {
    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let primary_key: Vec<u8> = item.struct_db_pk();
    assert_eq!(primary_key, "1-test".as_bytes());

    let secondary_key: HashMap<_, Vec<u8>> = item.struct_db_gks();
    assert_eq!(secondary_key.len(), 1);
    assert_eq!(
        secondary_key.get("item_generate_my_secondary_key").unwrap(),
        "test-1".as_bytes()
    );
}
