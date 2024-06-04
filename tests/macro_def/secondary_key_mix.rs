use native_db::db_type::Input;
use native_db::db_type::{KeyDefinition, KeyEntry};
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
    assert_eq!(primary_key, "1-test".to_key());

    let secondary_key: HashMap<_, KeyEntry> = item.native_db_secondary_keys();
    assert_eq!(secondary_key.len(), 2);
    assert_eq!(
        secondary_key.get(&KeyDefinition::new(
            1,
            1,
            "compute_secondary_key",
            Default::default()
        )),
        Some(&KeyEntry::Default("test-1".to_key()))
    );

    assert_eq!(
        secondary_key
            .get(&KeyDefinition::new(1, 1, "name", Default::default()))
            .unwrap(),
        &KeyEntry::Default("test".to_key())
    );
}
