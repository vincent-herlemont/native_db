use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// TODO somehow test visibility of keys enum from a sibling/child crate?

/// Test struct to ensure `#[native_db(export_keys = true)]` compiles successfully
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(export_keys = true)]
struct Item {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String,
}

#[test]
fn test_insert_my_item() {
    let item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let key = item.native_db_primary_key();
    assert_eq!(key, 1_u32.to_key());
}
