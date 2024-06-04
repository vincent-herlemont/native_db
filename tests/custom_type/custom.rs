use native_db::{
    db_type::{Key, ToKey},
    native_db, DatabaseBuilder,
};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
struct City(String);

impl ToKey for &City {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }
}

// Test genrate fields:
// - primary_key
// - secondary_keys (unique)
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct ItemFields {
    #[primary_key]
    city1: City,
    #[secondary_key(unique)]
    city2: City,
    #[secondary_key(optional)]
    city3: Option<City>,
}

#[test]
fn insert_item_fields() {
    let item = ItemFields {
        city1: City("New York".to_string()),
        city2: City("New York".to_string()),
        city3: Some(City("New York".to_string())),
    };

    let tf = TmpFs::new().unwrap();
    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemFields>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(ItemFieldsKey::city2, &item.city2)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
}

// Test genrate functions:
// - primary_key
// - secondary_keys (unique)
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(m_city1),
    secondary_key(m_city2, unique),
    secondary_key(m_city2_ref, unique),
    secondary_key(m_city3, optional)
)]
struct ItemFunctions {
    city1: City,
    city2: City,
    city3: Option<City>,
}

impl ItemFunctions {
    fn m_city1(&self) -> City {
        self.city1.clone()
    }

    fn m_city2(&self) -> City {
        self.city2.clone()
    }

    fn m_city2_ref(&self) -> &City {
        &self.city2
    }

    fn m_city3(&self) -> Option<City> {
        self.city3.clone()
    }
}

#[test]
fn test_item_functions() {
    let item = ItemFunctions {
        city1: City("New York".to_string()),
        city2: City("New York".to_string()),
        city3: Some(City("New York".to_string())),
    };

    let tf = TmpFs::new().unwrap();
    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemFunctions>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result_item = r
        .get()
        .secondary(ItemFunctionsKey::m_city2, &item.city2)
        .unwrap()
        .unwrap();
    assert_eq!(item, result_item);
}
