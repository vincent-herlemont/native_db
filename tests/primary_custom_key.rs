use native_db::db_type::Result;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(custom_id))]
struct ItemCustomPk {
    id: String,
    name: String,
}

impl ItemCustomPk {
    fn custom_id(&self) -> String {
        self.id.clone()
    }
}

#[test]
fn test_get_primary_key_custom_pk() {
    let item = ItemCustomPk {
        id: "1".to_string(),
        name: "test".to_string(),
    };

    let mut models = Models::new();
    models.define::<ItemCustomPk>().unwrap();
    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let id = String::from("1");
    let result_item = r.get().primary(id).unwrap().unwrap();
    assert_eq!(item, result_item);
}