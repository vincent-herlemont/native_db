use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(custom_id -> u32),
    // secondary_key(custom_sk -> Vec<std::string::String>),
    // secondary_key(custom_sk_o -> Option<Vec<std::string::String>>, optional),
    // secondary_key(custom_sk_u -> Vec<std::string::String>, unique),
    // secondary_key(custom_sk_o_u -> Option<Vec<std::string::String>>, unique, optional),
    secondary_key(custom_sk_no_u -> Option<Vec<std::string::String>>, unique),
)]
struct ItemCustomPk {
    id: u32,
    all_sk: Vec<std::string::String>,
}

impl ItemCustomPk {
    fn custom_id(&self) -> u32 {
        self.id
    }

    // fn custom_sk(&self) -> Vec<String> {
    //     self.all_sk.clone()
    // }

    // fn custom_sk_u(&self) -> Vec<String> {
    //     self.all_sk.clone()
    // }

    // fn custom_sk_o(&self) -> Option<Vec<String>> {
    //     Some(self.all_sk.clone())
    // }

    // fn custom_sk_o_u(&self) -> Option<Vec<String>> {
    //     Some(self.all_sk.clone())
    // }

    fn custom_sk_no_u(&self) -> Option<Vec<String>> {
        Some(self.all_sk.clone())
    }
}

#[test]
fn test_get_primary_key_custom_pk() {
    let item = ItemCustomPk {
        id: 1,
        all_sk: vec!["1".to_string()],
    };

    let mut models = Models::new();
    models.define::<ItemCustomPk>().unwrap();
    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    // // Get primary key for read transaction for unique
    // let r = db.r_transaction().unwrap();
    // let id = vec!["1".to_string()];
    // let result_item = r
    //     .get()
    //     .secondary(ItemCustomPkKey::custom_sk_u, id)
    //     .unwrap()
    //     .unwrap();

    // assert_eq!(item, result_item);

    // // Get primary key for read transaction for unique optional
    // let r = db.r_transaction().unwrap();
    // let id = vec!["1".to_string()];
    // let result_item = r
    //     .get()
    //     .secondary(ItemCustomPkKey::custom_sk_o_u, id)
    //     .unwrap()
    //     .unwrap();

    // assert_eq!(item, result_item);

    // Get primary key for read transaction for unique not optional
    let r = db.r_transaction().unwrap();
    let id = Some(vec!["1".to_string()]);
    let result_item = r
        .get()
        .secondary(ItemCustomPkKey::custom_sk_no_u, id)
        .unwrap()
        .unwrap();

    assert_eq!(item, result_item);
}
