use native_db::db_type::Input;
use native_db::db_type::{DatabaseKeyDefinition, DatabaseKeyValue};
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(compute_primary_key), secondary_key(compute_secondary_key))]
struct ItemSecondaryMix {
    id: u32,
    #[secondary_key(unique)]
    name: String,
}

impl ItemSecondaryMix {
    pub fn compute_primary_key(&self) -> String {
        format!("{}-{}", self.id, self.name)
    }
    pub fn compute_secondary_key(&self) -> String {
        format!("{}-{}", self.name, self.id)
    }
}

#[test]
fn test_secondary() {
    let item = ItemSecondaryMix {
        id: 1,
        name: "test".to_string(),
    };

    let primary_key = item.native_db_primary_key();
    assert_eq!(primary_key, "1-test".database_inner_key_value());

    let secondary_key: HashMap<_, DatabaseKeyValue> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 2);
    assert_eq!(
        secondary_key.get(&DatabaseKeyDefinition::new(
            1,
            1,
            "compute_secondary_key",
            Default::default()
        )),
        Some(&DatabaseKeyValue::Default(
            "test-1".database_inner_key_value()
        ))
    );

    assert_eq!(
        secondary_key
            .get(&DatabaseKeyDefinition::new(
                1,
                1,
                "name",
                Default::default()
            ))
            .unwrap(),
        &DatabaseKeyValue::Default("test".database_inner_key_value())
    );
}
