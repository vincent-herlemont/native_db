use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(generate_my_primary_key -> Vec<u8>)
)]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        self.id.to_le_bytes().to_vec()
    }

    pub fn inc(&mut self) -> &Self {
        self.id += 1;
        self
    }
}

#[test]
fn drain_all() {
    let tf = TmpFs::new().unwrap();

    let mut item = Item {
        id: 1,
        name: "test".to_string(),
    };

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    // Insert 5 items
    let rw = db.rw_transaction().unwrap();
    rw.insert(item.clone()).unwrap();
    rw.insert(item.inc().clone()).unwrap();
    rw.insert(item.inc().clone()).unwrap();
    rw.insert(item.inc().clone()).unwrap();
    rw.insert(item.inc().clone()).unwrap();
    rw.commit().unwrap();

    // Count items
    let r = db.r_transaction().unwrap();
    let len = r.len().primary::<Item>().unwrap();
    assert_eq!(len, 5);

    // Drain items
    let rw = db.rw_transaction().unwrap();
    let items = rw.drain().primary::<Item>().unwrap();
    assert_eq!(items.len(), 5);
    rw.commit().unwrap();

    // Count items
    let r = db.r_transaction().unwrap();
    let len = r.len().primary::<Item>().unwrap();
    assert_eq!(len, 0);
}
