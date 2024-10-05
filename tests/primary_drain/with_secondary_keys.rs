use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(generate_my_primary_key -> u32),
    secondary_key(generate_my_secondary_key -> String, unique)
)]
struct Item {
    id: u32,
    name: String,
    tag: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> u32 {
        self.id
    }

    pub fn generate_my_secondary_key(&self) -> String {
        let mut tag = self.tag.clone();
        let primary_key = self.generate_my_primary_key().to_string();
        tag.push_str(&primary_key);
        tag
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
        tag: "red".to_string(),
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

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_generate_my_primary_key");
    assert_eq!(stats.primary_tables[0].n_entries, Some(5));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(
        stats.secondary_tables[0].name,
        "1_1_generate_my_secondary_key"
    );
    assert_eq!(stats.secondary_tables[0].n_entries, Some(5));

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

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_generate_my_primary_key");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(
        stats.secondary_tables[0].name,
        "1_1_generate_my_secondary_key"
    );
    assert_eq!(stats.secondary_tables[0].n_entries, Some(0));
}
