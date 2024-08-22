use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(primary_key(id_key), secondary_key(name_key))]
struct ItemV1 {
    id: u32,
    name: String,
}

impl ItemV1 {
    pub fn id_key(&self) -> u32 {
        self.id
    }

    pub fn name_key(&self) -> String {
        let mut tag = self.name.clone();
        let primary_key = self.id_key().to_string();
        tag.push_str(&primary_key);
        tag
    }
    pub fn inc(&mut self, new_name: &str) -> &Self {
        self.id += 1;
        self.name = new_name.to_string();
        self
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 2, from = ItemV1)]
#[native_db(
    primary_key(id_key),
    secondary_key(first_name_key, unique),
    secondary_key(last_name_key, unique)
)]
struct ItemV2 {
    id: u64,
    first_name: String,
    last_name: String,
}

impl ItemV2 {
    pub fn id_key(&self) -> u64 {
        self.id
    }

    pub fn first_name_key(&self) -> String {
        let mut tag = self.first_name.clone();
        let primary_key = self.id_key().to_string();
        tag.push_str(&primary_key);
        tag
    }

    pub fn last_name_key(&self) -> String {
        let mut tag = self.last_name.clone();
        let primary_key = self.id_key().to_string();
        tag.push_str(&primary_key);
        tag
    }
}

impl From<ItemV1> for ItemV2 {
    fn from(item: ItemV1) -> Self {
        let mut name = item.name.split(' ');
        let first_name = name.next().unwrap_or("").to_string();
        let last_name = name.next().unwrap_or("").to_string();
        ItemV2 {
            id: item.id as u64,
            first_name,
            last_name,
        }
    }
}

impl From<ItemV2> for ItemV1 {
    fn from(item: ItemV2) -> Self {
        ItemV1 {
            id: item.id as u32,
            name: format!("{} {}", item.first_name, item.last_name),
        }
    }
}

#[test]
fn test_migrate() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<ItemV1>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let mut item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let rw_txn = db.rw_transaction().unwrap();
    {
        rw_txn.insert(item.clone()).unwrap();
        rw_txn.insert(item.inc("Victor Hugo").clone()).unwrap();
        rw_txn.insert(item.inc("Jules Verne").clone()).unwrap();
        rw_txn.insert(item.inc("Alexandre Dumas").clone()).unwrap();
        rw_txn.insert(item.inc("Emile Zola").clone()).unwrap();
    }
    rw_txn.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 1);
    assert_eq!(stats.primary_tables[0].name, "1_1_id_key");
    assert_eq!(stats.primary_tables[0].n_entries, Some(5));
    assert_eq!(stats.secondary_tables.len(), 1);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name_key");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(5));

    drop(db);

    let mut models = Models::new();
    models.define::<ItemV1>().unwrap();
    models.define::<ItemV2>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.migrate::<ItemV2>().unwrap();
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 2);
    assert_eq!(stats.primary_tables[1].name, "1_2_id_key");
    assert_eq!(stats.primary_tables[1].n_entries, Some(5));
    assert_eq!(stats.primary_tables[0].name, "1_1_id_key");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.secondary_tables.len(), 3);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name_key");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(0));
    assert_eq!(stats.secondary_tables[1].name, "1_2_first_name_key");
    assert_eq!(stats.secondary_tables[1].n_entries, Some(5));
    assert_eq!(stats.secondary_tables[2].name, "1_2_last_name_key");
    assert_eq!(stats.secondary_tables[2].n_entries, Some(5));

    let r_txn = db.r_transaction().unwrap();

    // Get Victor Hugo by id
    let item: ItemV2 = r_txn.get().primary(2_u64).unwrap().unwrap();
    assert_eq!(
        item,
        ItemV2 {
            id: 2,
            first_name: "Victor".to_string(),
            last_name: "Hugo".to_string(),
        }
    );

    // Get Alexandre Dumas by first name
    let item: Vec<ItemV2> = r_txn
        .scan()
        .secondary(ItemV2Key::first_name_key)
        .unwrap()
        .start_with("Alexandre")
        .unwrap().try_collect()
        .unwrap();
    assert_eq!(
        item,
        vec![ItemV2 {
            id: 4,
            first_name: "Alexandre".to_string(),
            last_name: "Dumas".to_string(),
        }]
    );

    // Get Julien Verne by last name
    let item: Vec<ItemV2> = r_txn
        .scan()
        .secondary(ItemV2Key::last_name_key)
        .unwrap()
        .start_with("Verne")
        .unwrap().try_collect()
        .unwrap();
    assert_eq!(
        item,
        vec![ItemV2 {
            id: 3,
            first_name: "Jules".to_string(),
            last_name: "Verne".to_string(),
        }]
    );
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 3, try_from = (ItemV2, db_type::Error))]
#[native_db]
struct ItemV3 {
    #[primary_key]
    id: u64,
    first_name: String,
    #[secondary_key]
    last_name: String,
}

impl TryFrom<ItemV3> for ItemV2 {
    type Error = db_type::Error;
    fn try_from(item: ItemV3) -> std::result::Result<Self, Self::Error> {
        Ok(ItemV2 {
            id: item.id,
            first_name: item.first_name,
            last_name: item.last_name,
        })
    }
}

impl TryFrom<ItemV2> for ItemV3 {
    type Error = db_type::Error;
    fn try_from(item: ItemV2) -> std::result::Result<Self, Self::Error> {
        Ok(ItemV3 {
            id: item.id,
            first_name: item.first_name,
            last_name: item.last_name,
        })
    }
}

#[test]
fn test_migrate_v3() {
    let tf = TmpFs::new().unwrap();
    let mut models = Models::new();
    models.define::<ItemV1>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let mut item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let rw_txn = db.rw_transaction().unwrap();
    {
        rw_txn.insert(item.clone()).unwrap();
        rw_txn.insert(item.inc("Victor Hugo").clone()).unwrap();
        rw_txn.insert(item.inc("Jules Verne").clone()).unwrap();
        rw_txn.insert(item.inc("Alexandre Dumas").clone()).unwrap();
        rw_txn.insert(item.inc("Emile Zola").clone()).unwrap();
    }
    rw_txn.commit().unwrap();

    drop(db);

    let mut models = Models::new();
    models.define::<ItemV1>().unwrap();
    models.define::<ItemV2>().unwrap();
    models.define::<ItemV3>().unwrap();
    let db = Builder::new()
        .open(&models, tf.path("test").as_std_path())
        .unwrap();

    // Return error because the latest version is Item is ItemV3
    let rw = db.rw_transaction().unwrap();
    let error = rw.migrate::<ItemV2>().unwrap_err();
    assert!(matches!(error, db_type::Error::MigrateLegacyModel(_)));
    rw.commit().unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.migrate::<ItemV3>().unwrap();
    rw.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.primary_tables.len(), 3);
    assert_eq!(stats.primary_tables[0].name, "1_1_id_key");
    assert_eq!(stats.primary_tables[0].n_entries, Some(0));
    assert_eq!(stats.primary_tables[1].name, "1_2_id_key");
    assert_eq!(stats.primary_tables[1].n_entries, Some(0));
    assert_eq!(stats.primary_tables[2].name, "1_3_id");
    assert_eq!(stats.primary_tables[2].n_entries, Some(5));
    assert_eq!(stats.secondary_tables.len(), 4);
    assert_eq!(stats.secondary_tables[0].name, "1_1_name_key");
    assert_eq!(stats.secondary_tables[0].n_entries, Some(0));
    assert_eq!(stats.secondary_tables[1].name, "1_2_first_name_key");
    assert_eq!(stats.secondary_tables[1].n_entries, Some(0));
    assert_eq!(stats.secondary_tables[2].name, "1_2_last_name_key");
    assert_eq!(stats.secondary_tables[2].n_entries, Some(0));
    assert_eq!(stats.secondary_tables[3].name, "1_3_last_name");
    assert_eq!(stats.secondary_tables[3].n_entries, Some(5));
}
