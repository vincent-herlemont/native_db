use native_model::native_model;
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use struct_db::{struct_db, Db, ReadableTable};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[struct_db(fn_primary_key(id_key), fn_secondary_key(name_key))]
struct ItemV1 {
    id: u32,
    name: String,
}

impl ItemV1 {
    pub fn id_key(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }

    pub fn name_key(&self) -> Vec<u8> {
        let mut tag = self.name.as_bytes().to_vec();
        let primary_key = self.id_key();
        tag.extend_from_slice(&primary_key);
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
#[struct_db(
    fn_primary_key(id_key),
    fn_secondary_key(first_name_key),
    fn_secondary_key(last_name_key)
)]
struct ItemV2 {
    id: u64,
    first_name: String,
    last_name: String,
}

impl ItemV2 {
    pub fn id_key(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }

    pub fn first_name_key(&self) -> Vec<u8> {
        let mut tag = self.first_name.as_bytes().to_vec();
        let primary_key = self.id_key();
        tag.extend_from_slice(&primary_key);
        tag
    }

    pub fn last_name_key(&self) -> Vec<u8> {
        let mut tag = self.last_name.as_bytes().to_vec();
        let primary_key = self.id_key();
        tag.extend_from_slice(&primary_key);
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
    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<ItemV1>().unwrap();

    let mut item = ItemV1 {
        id: 1,
        name: "test".to_string(),
    };

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, item.clone()).unwrap();
        tables
            .insert(&txn, item.inc("Victor Hugo").clone())
            .unwrap();
        tables
            .insert(&txn, item.inc("Jules Verne").clone())
            .unwrap();
        tables
            .insert(&txn, item.inc("Alexandre Dumas").clone())
            .unwrap();
        tables.insert(&txn, item.inc("Emile Zola").clone()).unwrap();
    }
    txn.commit().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.stats_tables.len(), 2);
    assert_eq!(stats.stats_tables[0].name, "itemv1");
    assert_eq!(stats.stats_tables[0].num_raw, 5);
    assert_eq!(stats.stats_tables[1].name, "itemv1_name_key");
    assert_eq!(stats.stats_tables[1].num_raw, 5);
    drop(db);

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<ItemV1>().unwrap();
    db.define::<ItemV2>().unwrap();

    db.migrate::<ItemV2>().unwrap();

    let stats = db.redb_stats().unwrap();
    assert_eq!(stats.stats_tables.len(), 5);
    assert_eq!(stats.stats_tables[0].name, "itemv1");
    assert_eq!(stats.stats_tables[0].num_raw, 0);
    assert_eq!(stats.stats_tables[1].name, "itemv1_name_key");
    assert_eq!(stats.stats_tables[1].num_raw, 0);
    assert_eq!(stats.stats_tables[2].name, "itemv2");
    assert_eq!(stats.stats_tables[2].num_raw, 5);
    assert_eq!(stats.stats_tables[3].name, "itemv2_first_name_key");
    assert_eq!(stats.stats_tables[3].num_raw, 5);
    assert_eq!(stats.stats_tables[4].name, "itemv2_last_name_key");
    assert_eq!(stats.stats_tables[4].num_raw, 5);

    let txn = db.read_transaction().unwrap();

    // Get Victor Hugo by id
    let item: ItemV2 = txn
        .tables()
        .primary_get(&txn, 2_u64.to_be_bytes().as_slice())
        .unwrap()
        .unwrap();
    assert_eq!(
        item,
        ItemV2 {
            id: 2,
            first_name: "Victor".to_string(),
            last_name: "Hugo".to_string(),
        }
    );

    // Get Alexandre Dumas by first name
    let item: Vec<ItemV2> = txn
        .tables()
        .secondary_iter_start_with(&txn, ItemV2Key::first_name_key, b"Alexandre")
        .unwrap()
        .collect();
    assert_eq!(
        item,
        vec![ItemV2 {
            id: 4,
            first_name: "Alexandre".to_string(),
            last_name: "Dumas".to_string(),
        }]
    );

    // Get Julien Verne by last name
    let item: Vec<ItemV2> = txn
        .tables()
        .secondary_iter_start_with(&txn, ItemV2Key::last_name_key, b"Verne")
        .unwrap()
        .collect();
    assert_eq!(
        item,
        vec![ItemV2 {
            id: 3,
            first_name: "Jules".to_string(),
            last_name: "Verne".to_string(),
        }]
    );
}
