use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(compute_primary_key))]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn compute_primary_key(&self) -> String {
        format!("{}-{}", self.id, self.name)
    }
}

#[test]
fn test_insert_my_item() {
    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let key = item.native_db_primary_key();
    assert_eq!(key, "1-test".database_inner_key_value());
}
