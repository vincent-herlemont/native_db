// TODO: refactor and move to query/ folder

use itertools::Itertools;
use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(generate_my_primary_key -> u32),
    secondary_key(secondary_key_1 -> String, unique),
    secondary_key(secondary_key_2 -> String, unique)
)]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
    pub fn generate_my_primary_key(&self) -> u32 {
        self.id
    }
    pub fn secondary_key_1(&self) -> String {
        format!("{}", self.id)
    }
    pub fn secondary_key_2(&self) -> String {
        self.name.to_string()
    }
}

#[test]
fn test_iter() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Vec<Item> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");
}

// Check if the use of BigEndian is correct
#[test]
fn test_iter_many_items_to_be_bytes() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    // Insert 1000 items
    for i in 0..257 {
        rw.insert(Item::new(i, format!("test_{}", i).as_str()))
            .unwrap();
    }
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    {
        let iter: Vec<Item> = r
            .scan()
            .primary()
            .unwrap()
            .all()
            .unwrap()
            .try_collect()
            .unwrap();
        assert_eq!(iter.len(), 257);

        let obj1 = iter.first().unwrap();
        assert_eq!(obj1.id, 0);
        assert_eq!(obj1.name, "test_0");

        let obj2 = iter.get(1).unwrap();
        assert_eq!(obj2.id, 1);
        assert_eq!(obj2.name, "test_1");

        let obj3 = iter.get(256).unwrap();
        assert_eq!(obj3.id, 256);
        assert_eq!(obj3.name, "test_256");
    }
}

#[test]
fn test_double_ended_iter() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let scan = r.scan().primary().unwrap();
    let iter = scan.all().unwrap();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();

    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 1);
    assert_eq!(obj2.name, "test");
}

#[test]
fn test_iter_range() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.insert(Item::new(3, "test3")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Vec<Item> = r
        .scan()
        .primary()
        .unwrap()
        .range(..2u32)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let result: Vec<Item> = r
        .scan()
        .primary()
        .unwrap()
        .range(2u32..)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 3);
    assert_eq!(obj2.name, "test3");

    let result: Vec<Item> = r
        .scan()
        .primary()
        .unwrap()
        .range(2u32..3u32)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");
}

#[test]
fn test_iter_by_key() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Vec<Item> = r
        .scan()
        .secondary(ItemKey::secondary_key_1)
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();

    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");
}

#[test]
fn test_double_ended_iter_by_key() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.all().unwrap();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 1);
    assert_eq!(obj2.name, "test");
}

#[test]
fn test_double_ended_iter_by_key_range() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.insert(Item::new(3, "test3")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.range(.."2").unwrap();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 1);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.range("2"..).unwrap();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 3);
    assert_eq!(obj1.name, "test3");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");

    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.range("2".."3").unwrap();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 1);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db(primary_key(generate_my_primary_key -> String))]
struct ItemFlag {
    name: String,
}

impl ItemFlag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn generate_my_primary_key(&self) -> String {
        self.name.to_string()
    }
}

#[test]
fn test_start_with_scenario() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<ItemFlag>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    // Red flag
    rw.insert(ItemFlag::new("red:1")).unwrap();
    rw.insert(ItemFlag::new("red:2")).unwrap();
    rw.insert(ItemFlag::new("red:3")).unwrap();
    // Blue flag
    rw.insert(ItemFlag::new("blue:1")).unwrap();
    rw.insert(ItemFlag::new("blue:2")).unwrap();
    rw.insert(ItemFlag::new("blue:3")).unwrap();
    // Green flag
    rw.insert(ItemFlag::new("green:1")).unwrap();
    rw.insert(ItemFlag::new("green:2")).unwrap();
    rw.insert(ItemFlag::new("green:3")).unwrap();
    rw.commit().unwrap();

    for p in ["red:", "blue:", "green:"] {
        let r = db.r_transaction().unwrap();

        let result: Vec<ItemFlag> = r
            .scan()
            .primary()
            .unwrap()
            .start_with(p.to_string().as_str())
            .unwrap()
            .try_collect()
            .unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.first().unwrap();
        assert_eq!(obj1.name, format!("{}1", p));

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.name, format!("{}2", p));

        let obj3 = result.get(2).unwrap();
        assert_eq!(obj3.name, format!("{}3", p));
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db(primary_key(generate_my_primary_key -> String), secondary_key(flag -> String, unique))]
struct ItemIdFlag {
    id: String,
    flag: String,
}

impl ItemIdFlag {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            flag: name.to_string(),
        }
    }

    pub fn generate_my_primary_key(&self) -> String {
        self.id.clone()
    }
    pub fn flag(&self) -> String {
        format!("{}:{}", self.flag, self.id)
    }
}

#[test]
fn test_start_with_by_key_scenario_write_txn() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<ItemIdFlag>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();

    // Red flag
    rw.insert(ItemIdFlag::new("1", "red")).unwrap();
    rw.insert(ItemIdFlag::new("2", "red")).unwrap();
    rw.insert(ItemIdFlag::new("3", "red")).unwrap();
    // Blue flag
    rw.insert(ItemIdFlag::new("4", "blue")).unwrap();
    rw.insert(ItemIdFlag::new("5", "blue")).unwrap();
    rw.insert(ItemIdFlag::new("6", "blue")).unwrap();
    // Green flag
    rw.insert(ItemIdFlag::new("7", "green")).unwrap();
    rw.insert(ItemIdFlag::new("8", "green")).unwrap();
    rw.insert(ItemIdFlag::new("9", "green")).unwrap();

    rw.commit().unwrap();

    for p in ["red:", "blue:", "green:"] {
        let rw = db.rw_transaction().unwrap();

        let result: Vec<ItemIdFlag> = rw
            .scan()
            .secondary(ItemIdFlagKey::flag)
            .unwrap()
            .start_with(p.to_string().as_str())
            .unwrap()
            .try_collect()
            .unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.first().unwrap();
        assert_eq!(format!("{}:", obj1.flag), format!("{}", p));

        let obj2 = result.get(1).unwrap();
        assert_eq!(format!("{}:", obj2.flag), format!("{}", p));

        let obj3 = result.get(2).unwrap();
        assert_eq!(format!("{}:", obj3.flag), format!("{}", p));
    }
}

#[test]
fn test_start_with_by_key_scenario_readonly_txn() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<ItemIdFlag>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    // Red flag
    rw.insert(ItemIdFlag::new("1", "red")).unwrap();
    rw.insert(ItemIdFlag::new("2", "red")).unwrap();
    rw.insert(ItemIdFlag::new("3", "red")).unwrap();
    // Blue flag
    rw.insert(ItemIdFlag::new("4", "blue")).unwrap();
    rw.insert(ItemIdFlag::new("5", "blue")).unwrap();
    rw.insert(ItemIdFlag::new("6", "blue")).unwrap();
    // Green flag
    rw.insert(ItemIdFlag::new("7", "green")).unwrap();
    rw.insert(ItemIdFlag::new("8", "green")).unwrap();
    rw.insert(ItemIdFlag::new("9", "green")).unwrap();
    rw.commit().unwrap();

    for p in ["red:", "blue:", "green:"] {
        let r = db.r_transaction().unwrap();
        let result: Vec<ItemIdFlag> = r
            .scan()
            .secondary(ItemIdFlagKey::flag)
            .unwrap()
            .start_with(p.to_string().as_str())
            .unwrap()
            .try_collect()
            .unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.first().unwrap();
        assert_eq!(format!("{}:", obj1.flag), format!("{}", p));

        let obj2 = result.get(1).unwrap();
        assert_eq!(format!("{}:", obj2.flag), format!("{}", p));

        let obj3 = result.get(2).unwrap();
        assert_eq!(format!("{}:", obj3.flag), format!("{}", p));
    }
}

#[test]
fn test_txn_write_iter() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let rw = db.rw_transaction().unwrap();
    let result: Vec<Item> = rw
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");
}

#[test]
fn test_txn_write_iter_range() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<Item>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.insert(Item::new(3, "test3")).unwrap();
    rw.commit().unwrap();

    let rw = db.rw_transaction().unwrap();
    let result: Vec<Item> = rw
        .scan()
        .primary()
        .unwrap()
        .range(..2u32)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let result: Vec<Item> = rw
        .scan()
        .primary()
        .unwrap()
        .range(2u32..)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 3);
    assert_eq!(obj2.name, "test3");

    let result: Vec<Item> = rw
        .scan()
        .primary()
        .unwrap()
        .range(2u32..3u32)
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.first().unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");
}

#[test]
fn test_txn_write_start_with_scenario() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<ItemFlag>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    // Red flag
    rw.insert(ItemFlag::new("red:1")).unwrap();
    rw.insert(ItemFlag::new("red:2")).unwrap();
    rw.insert(ItemFlag::new("red:3")).unwrap();
    // Blue flag
    rw.insert(ItemFlag::new("blue:1")).unwrap();
    rw.insert(ItemFlag::new("blue:2")).unwrap();
    rw.insert(ItemFlag::new("blue:3")).unwrap();
    // Green flag
    rw.insert(ItemFlag::new("green:1")).unwrap();
    rw.insert(ItemFlag::new("green:2")).unwrap();
    rw.insert(ItemFlag::new("green:3")).unwrap();
    rw.commit().unwrap();

    for p in ["red:", "blue:", "green:"] {
        let rw = db.rw_transaction().unwrap();

        let result: Vec<ItemFlag> = rw
            .scan()
            .primary()
            .unwrap()
            .start_with(p.to_string().as_str())
            .unwrap()
            .try_collect()
            .unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.first().unwrap();
        assert_eq!(obj1.name, format!("{}1", p));

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.name, format!("{}2", p));

        let obj3 = result.get(2).unwrap();
        assert_eq!(obj3.name, format!("{}3", p));
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 4, version = 1)]
#[native_db]
struct ItemScanRange {
    #[primary_key]
    id: u32,

    #[secondary_key]
    nr: u32,

    #[secondary_key(unique)]
    unique_nr: u32,
}

#[test]
fn test_scan_range() {
    let item_1 = ItemScanRange {
        id: 1,
        nr: 1,
        unique_nr: 1,
    };
    let item_2 = ItemScanRange {
        id: 2,
        nr: 2,
        unique_nr: 2,
    };
    let item_3 = ItemScanRange {
        id: 3,
        nr: 2,
        unique_nr: 3,
    };
    let item_4 = ItemScanRange {
        id: 4,
        nr: 3,
        unique_nr: 4,
    };

    let mut models = Models::new();
    models.define::<ItemScanRange>().unwrap();
    let db = Builder::new().create_in_memory(&models).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(item_1.clone()).unwrap();
    rw.insert(item_2.clone()).unwrap();
    rw.insert(item_3.clone()).unwrap();
    rw.insert(item_4.clone()).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result = r
        .scan()
        .secondary(ItemScanRangeKey::nr)
        .unwrap()
        .range(0u32..10u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 2, 3], "range 0..10 for nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::nr)
        .unwrap()
        .range(2u32..3u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![2, 2], "range 2..3 for nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::unique_nr)
        .unwrap()
        .range(1u32..3u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.unique_nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2], "range 1..3 for unique_nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::unique_nr)
        .unwrap()
        .range(1u32..=3u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.unique_nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 3], "range 1..=3 for unique_nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::unique_nr)
        .unwrap()
        .range(3u32..=3u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.unique_nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![3], "range 3..=3 for unique_nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::nr)
        .unwrap()
        .range(2u32..=3u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![2, 2, 3], "range 2..=3 for nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::nr)
        .unwrap()
        .range(2u32..=2u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![2, 2], "range 2..=2 for nr");

    let result = r
        .scan()
        .secondary(ItemScanRangeKey::nr)
        .unwrap()
        .range(0u32..=2u32)
        .unwrap()
        .collect::<Result<Vec<ItemScanRange>, _>>()
        .unwrap()
        .iter()
        .map(|x| x.nr)
        .collect::<Vec<_>>();
    assert_eq!(result, vec![1, 2, 2], "range 0..=2 for nr");
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[native_model(id = 4, version = 1)]
#[native_db]
struct ItemLowLevelScanRange {
    #[primary_key]
    primary_key: Vec<u8>,

    #[secondary_key]
    secondary_key: Vec<u8>,

    #[secondary_key(unique)]
    secondary_key_unique: Vec<u8>,

    #[secondary_key(optional)]
    secondary_key_optional: Option<Vec<u8>>,

    #[secondary_key(unique, optional)]
    secondary_key_unique_optional: Option<Vec<u8>>,
}

#[test]
fn test_low_level_scan_range() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<ItemLowLevelScanRange>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(ItemLowLevelScanRange {
        primary_key: vec![255],
        secondary_key: vec![2],
        secondary_key_unique: vec![2],
        secondary_key_optional: None,
        secondary_key_unique_optional: None,
    })
    .unwrap();
    rw.insert(ItemLowLevelScanRange {
        primary_key: vec![2],
        secondary_key: vec![2, 0, 254],
        secondary_key_unique: vec![255],
        secondary_key_optional: None,
        secondary_key_unique_optional: None,
    })
    .unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Vec<ItemLowLevelScanRange> = r
        .scan()
        .secondary(ItemLowLevelScanRangeKey::secondary_key)
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].secondary_key, vec![2]);
    assert_eq!(result[1].secondary_key, vec![2, 0, 254]);

    // Maybe this idea does not work because it's lexicographical order, and the fact to have th same size of the primary key part
    // change nothing.
    // Use XXhash for the primary_key part https://docs.rs/xxhash-rust/latest/xxhash_rust/xxh64/index.html of the secondary_key.
    // And detect colision, use the return value of https://docs.rs/redb/latest/redb/struct.Table.html#method.insert
    // and re-insert the item if the return value is not null, recompute the primary key hash with a timestamp.
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 10, version = 1)]
#[native_db]
struct ItemOptionalSecondaryKeyRangeTest {
    #[primary_key]
    id: u32,
    #[secondary_key(optional)]
    optional_string_key: Option<String>,
    #[secondary_key(optional)]
    optional_u32_key: Option<u32>,
}

/// Test that demonstrates the expected behavior when querying for None values
/// in optional secondary keys using range syntax.
///
/// IMPORTANT: Native DB does NOT support querying for None/NULL values in
/// optional secondary indexes using range syntax. This is expected behavior.
///
/// When an optional secondary key has a None value, it is not indexed in the
/// secondary index table, making it impossible to query for these entries
/// using range queries like `None..=None`.
///
/// To find items with None values in optional secondary keys, you must:
/// 1. Query all items and filter in application code
/// 2. Use a different indexing strategy (e.g., use a sentinel value instead of None)
/// 3. Add a separate boolean field to track presence/absence
///
/// ## Sentinel Value Approach (Option 2)
///
/// A sentinel value is a special marker value that represents "no data" or "null"
/// while still being a valid, indexable value. Instead of using `Option<T>` with
/// potential `None` values that won't be indexed, you use a regular type with a
/// reserved value to indicate absence.
///
/// ### Example:
/// ```rust
/// // Instead of Option<String> with None values that can't be queried:
/// #[secondary_key(optional)]
/// name: Option<String>,  // None values are NOT indexed
///
/// // Use a sentinel value that CAN be queried:
/// #[secondary_key]
/// name: String,  // Empty string "" acts as sentinel for "no name"
/// ```
///
/// ### Common Sentinel Values:
/// - For `String`: Empty string `""`
/// - For unsigned integers: `u32::MAX`, `u64::MAX`
/// - For signed integers: `-1`, `i32::MIN`
/// - For custom types: Define a special "NULL" variant
///
/// ### Benefits:
/// - All values are indexed and queryable (including "null" sentinels)
/// - Can use efficient range queries to find "null" values
/// - No Option wrapper overhead
///
/// ### Trade-offs:
/// - Must reserve sentinel values (can't use them as real data)
/// - Less type-safe than Option<T>
/// - Requires documentation of which values are sentinels
///
/// See `test_sentinel_value_approach()` below for a complete working example.
#[test]
fn test_optional_secondary_key_none_range() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models
        .define::<ItemOptionalSecondaryKeyRangeTest>()
        .unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    // Insert test data with mix of None and Some values
    let rw = db.rw_transaction().unwrap();

    // Items with None values
    rw.insert(ItemOptionalSecondaryKeyRangeTest {
        id: 1,
        optional_string_key: None,
        optional_u32_key: None,
    })
    .unwrap();

    rw.insert(ItemOptionalSecondaryKeyRangeTest {
        id: 2,
        optional_string_key: None,
        optional_u32_key: Some(100),
    })
    .unwrap();

    // Items with Some values
    rw.insert(ItemOptionalSecondaryKeyRangeTest {
        id: 3,
        optional_string_key: Some("apple".to_string()),
        optional_u32_key: Some(50),
    })
    .unwrap();

    rw.insert(ItemOptionalSecondaryKeyRangeTest {
        id: 4,
        optional_string_key: Some("banana".to_string()),
        optional_u32_key: None,
    })
    .unwrap();

    rw.insert(ItemOptionalSecondaryKeyRangeTest {
        id: 5,
        optional_string_key: Some("cherry".to_string()),
        optional_u32_key: Some(200),
    })
    .unwrap();

    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();

    // Test 1: Query for exactly None values using None..=None range
    // EXPECTED: This will return 0 items because None values are not indexed
    println!(
        "Testing range query for None values: Option::<String>::None..=Option::<String>::None"
    );
    let result: Result<Vec<ItemOptionalSecondaryKeyRangeTest>, _> = r
        .scan()
        .secondary(ItemOptionalSecondaryKeyRangeTestKey::optional_string_key)
        .unwrap()
        .range(Option::<String>::None..=Option::<String>::None)
        .unwrap()
        .try_collect();

    match result {
        Ok(items) => {
            println!(
                "Successfully queried None range, found {} items",
                items.len()
            );
            let ids: Vec<u32> = items.iter().map(|item| item.id).collect();
            println!("IDs of items with None optional_string_key: {:?}", ids);
            // EXPECTED BEHAVIOR: Will find 0 items because None values are not indexed
            // in optional secondary keys. Items with id 1 and 2 have None values
            // but cannot be queried this way.
            assert_eq!(
                ids.len(),
                0,
                "Expected 0 items: None values are not indexed in optional secondary keys"
            );
        }
        Err(e) => {
            println!("Error querying None range: {:?}", e);
            panic!("Range query for None values failed: {:?}", e);
        }
    }

    // Test 2: Query for None using different syntax with u32 type
    // EXPECTED: This will also return 0 items
    println!("Testing range query for None values using None..=None");
    let result: Result<Vec<ItemOptionalSecondaryKeyRangeTest>, _> = r
        .scan()
        .secondary(ItemOptionalSecondaryKeyRangeTestKey::optional_u32_key)
        .unwrap()
        .range(Option::<u32>::None..=Option::<u32>::None)
        .unwrap()
        .try_collect();

    match result {
        Ok(items) => {
            println!(
                "Successfully queried None range for u32, found {} items",
                items.len()
            );
            let ids: Vec<u32> = items.iter().map(|item| item.id).collect();
            println!("IDs of items with None optional_u32_key: {:?}", ids);
            // EXPECTED BEHAVIOR: Will find 0 items because None values are not indexed
            // Items with id 1 and 4 have None values but cannot be queried this way
            assert_eq!(
                ids.len(),
                0,
                "Expected 0 items: None values are not indexed in optional secondary keys"
            );
        }
        Err(e) => {
            println!("Error querying None range for u32: {:?}", e);
            panic!("Range query for None values failed: {:?}", e);
        }
    }

    // Test 3: Query range from None to Some value
    // EXPECTED: This will only return items with Some values in the range, not None values
    println!("Testing range query from None to Some value");
    let result: Result<Vec<ItemOptionalSecondaryKeyRangeTest>, _> = r
        .scan()
        .secondary(ItemOptionalSecondaryKeyRangeTestKey::optional_u32_key)
        .unwrap()
        .range(Option::<u32>::None..=Some(100u32))
        .unwrap()
        .try_collect();

    match result {
        Ok(items) => {
            println!(
                "Successfully queried None to Some(100) range, found {} items",
                items.len()
            );
            let ids: Vec<u32> = items.iter().map(|item| item.id).collect();
            println!("IDs of items in None..=Some(100) range: {:?}", ids);
            // EXPECTED BEHAVIOR: Will find items with Some values <= 100 (items 2 and 3)
            // but NOT items with None values (items 1 and 4), even though None
            // is logically "less than" any Some value
            assert_eq!(ids.len(), 2, "Expected 2 items with Some values in range");
            assert!(ids.contains(&2), "Should find item 2 with Some(100)");
            assert!(ids.contains(&3), "Should find item 3 with Some(50)");
            assert!(!ids.contains(&1), "Should NOT find item 1 with None");
            assert!(!ids.contains(&4), "Should NOT find item 4 with None");
        }
        Err(e) => {
            println!("Error querying None to Some range: {:?}", e);
            panic!("Unexpected error in range query: {:?}", e);
        }
    }

    // Test 4: Demonstrate correct way to query items with Some values
    println!("\nDemonstrating correct way to query optional secondary keys:");
    let result: Vec<ItemOptionalSecondaryKeyRangeTest> = r
        .scan()
        .secondary(ItemOptionalSecondaryKeyRangeTestKey::optional_string_key)
        .unwrap()
        .range(Some("apple".to_string())..=Some("cherry".to_string()))
        .unwrap()
        .try_collect()
        .unwrap();

    println!(
        "Items with string keys between 'apple' and 'cherry': {} items",
        result.len()
    );
    assert_eq!(
        result.len(),
        3,
        "Should find all items with Some string values"
    );

    // Test 5: Show that querying all items and filtering is the way to find None values
    println!("\nDemonstrating how to find items with None values:");
    let all_items: Vec<ItemOptionalSecondaryKeyRangeTest> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();

    let items_with_none_string: Vec<u32> = all_items
        .iter()
        .filter(|item| item.optional_string_key.is_none())
        .map(|item| item.id)
        .collect();

    println!(
        "Items with None optional_string_key (via filtering): {:?}",
        items_with_none_string
    );
    assert_eq!(
        items_with_none_string.len(),
        2,
        "Should find 2 items with None string keys"
    );
    assert!(items_with_none_string.contains(&1));
    assert!(items_with_none_string.contains(&2));
}

/// Model using sentinel values instead of Option types
/// This allows all values (including "null" sentinels) to be indexed and queryable
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 11, version = 1)]
#[native_db]
struct ItemWithSentinelValues {
    #[primary_key]
    id: u32,
    #[secondary_key]
    name: String, // Uses "" as sentinel for "no name"
    #[secondary_key]
    priority: u32, // Uses u32::MAX as sentinel for "no priority"
    #[secondary_key]
    score: i32, // Uses -1 as sentinel for "no score"
}

/// Constants defining our sentinel values
const SENTINEL_NAME: &str = ""; // Empty string represents "no name"
const SENTINEL_PRIORITY: u32 = u32::MAX; // MAX value represents "no priority"
const SENTINEL_SCORE: i32 = -1; // -1 represents "no score"

/// Test demonstrating the sentinel value approach as an alternative to Option<T>
/// for handling "null" values in secondary indexes.
///
/// This approach allows you to query for "null" values using range syntax,
/// which is not possible with Option<T> secondary keys.
#[test]
fn test_sentinel_value_approach() {
    let tf = TmpFs::new().unwrap();

    let mut models = Models::new();
    models.define::<ItemWithSentinelValues>().unwrap();
    let db = Builder::new()
        .create(&models, tf.path("test").as_std_path())
        .unwrap();

    // Insert test data using sentinel values to represent "null"
    let rw = db.rw_transaction().unwrap();

    // Item with all sentinel values (equivalent to all None)
    rw.insert(ItemWithSentinelValues {
        id: 1,
        name: SENTINEL_NAME.to_string(),
        priority: SENTINEL_PRIORITY,
        score: SENTINEL_SCORE,
    })
    .unwrap();

    // Item with mixed sentinel and real values
    rw.insert(ItemWithSentinelValues {
        id: 2,
        name: SENTINEL_NAME.to_string(), // No name
        priority: 100,                   // Has priority
        score: 85,                       // Has score
    })
    .unwrap();

    // Item with all real values
    rw.insert(ItemWithSentinelValues {
        id: 3,
        name: "Alice".to_string(),
        priority: 50,
        score: 92,
    })
    .unwrap();

    // Another item with different sentinel values
    rw.insert(ItemWithSentinelValues {
        id: 4,
        name: "Bob".to_string(),
        priority: SENTINEL_PRIORITY, // No priority
        score: SENTINEL_SCORE,       // No score
    })
    .unwrap();

    rw.insert(ItemWithSentinelValues {
        id: 5,
        name: "Charlie".to_string(),
        priority: 75,
        score: SENTINEL_SCORE, // No score
    })
    .unwrap();

    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();

    // Test 1: Query for items with "no name" (sentinel value)
    // THIS WORKS with sentinel values, unlike Option<String>::None
    println!("\n=== Sentinel Value Approach Demo ===");
    println!("Querying for items with no name (empty string sentinel):");

    let items_with_no_name: Vec<ItemWithSentinelValues> = r
        .scan()
        .secondary(ItemWithSentinelValuesKey::name)
        .unwrap()
        .range(SENTINEL_NAME.to_string()..=SENTINEL_NAME.to_string())
        .unwrap()
        .try_collect()
        .unwrap();

    println!("Found {} items with no name", items_with_no_name.len());
    let ids: Vec<u32> = items_with_no_name.iter().map(|item| item.id).collect();
    println!("IDs of items with no name: {:?}", ids);

    // EXPECTED: Successfully finds items 1 and 2 which have empty string names
    assert_eq!(
        items_with_no_name.len(),
        2,
        "Should find 2 items with sentinel name"
    );
    assert!(ids.contains(&1), "Should find item 1");
    assert!(ids.contains(&2), "Should find item 2");

    // Test 2: Query for items with "no priority" (u32::MAX sentinel)
    println!("\nQuerying for items with no priority (u32::MAX sentinel):");

    let items_with_no_priority: Vec<ItemWithSentinelValues> = r
        .scan()
        .secondary(ItemWithSentinelValuesKey::priority)
        .unwrap()
        .range(SENTINEL_PRIORITY..=SENTINEL_PRIORITY)
        .unwrap()
        .try_collect()
        .unwrap();

    println!(
        "Found {} items with no priority",
        items_with_no_priority.len()
    );
    let ids: Vec<u32> = items_with_no_priority.iter().map(|item| item.id).collect();
    println!("IDs of items with no priority: {:?}", ids);

    assert_eq!(
        items_with_no_priority.len(),
        2,
        "Should find 2 items with sentinel priority"
    );
    assert!(ids.contains(&1), "Should find item 1");
    assert!(ids.contains(&4), "Should find item 4");

    // Test 3: Query for items with "no score" (-1 sentinel)
    println!("\nQuerying for items with no score (-1 sentinel):");

    let items_with_no_score: Vec<ItemWithSentinelValues> = r
        .scan()
        .secondary(ItemWithSentinelValuesKey::score)
        .unwrap()
        .range(SENTINEL_SCORE..=SENTINEL_SCORE)
        .unwrap()
        .try_collect()
        .unwrap();

    println!("Found {} items with no score", items_with_no_score.len());
    let ids: Vec<u32> = items_with_no_score.iter().map(|item| item.id).collect();
    println!("IDs of items with no score: {:?}", ids);

    assert_eq!(
        items_with_no_score.len(),
        3,
        "Should find 3 items with sentinel score"
    );
    assert!(ids.contains(&1), "Should find item 1");
    assert!(ids.contains(&4), "Should find item 4");
    assert!(ids.contains(&5), "Should find item 5");

    // Test 4: Range query including sentinel values
    // This demonstrates that sentinels participate in normal range queries
    println!("\nRange query from real value to sentinel:");

    let range_items: Vec<ItemWithSentinelValues> = r
        .scan()
        .secondary(ItemWithSentinelValuesKey::priority)
        .unwrap()
        .range(60..=SENTINEL_PRIORITY) // From 60 to MAX (includes sentinel)
        .unwrap()
        .try_collect()
        .unwrap();

    println!(
        "Found {} items in priority range 60..=MAX",
        range_items.len()
    );
    for item in &range_items {
        println!("  ID: {}, priority: {}", item.id, item.priority);
    }

    // Should find items with priority 75, 100, and MAX (sentinel)
    // Item 3 has priority 50 which is below 60, so not included
    // Item 1: priority = u32::MAX (sentinel)
    // Item 2: priority = 100
    // Item 3: priority = 50 (not in range)
    // Item 4: priority = u32::MAX (sentinel)
    // Item 5: priority = 75
    assert_eq!(
        range_items.len(),
        4,
        "Should find 4 items in range (75, 100, and 2x MAX)"
    );

    // Test 5: Demonstrate helper functions for cleaner code
    println!("\n=== Using Helper Functions ===");

    // In practice, you'd typically wrap sentinel checks in helper methods
    let all_items: Vec<ItemWithSentinelValues> = r
        .scan()
        .primary()
        .unwrap()
        .all()
        .unwrap()
        .try_collect()
        .unwrap();

    // Helper closures to check for sentinel values
    let has_name = |item: &ItemWithSentinelValues| -> bool { item.name != SENTINEL_NAME };

    let has_priority =
        |item: &ItemWithSentinelValues| -> bool { item.priority != SENTINEL_PRIORITY };

    let has_score = |item: &ItemWithSentinelValues| -> bool { item.score != SENTINEL_SCORE };

    // Count items with real values
    let items_with_names = all_items.iter().filter(|i| has_name(i)).count();
    let items_with_priorities = all_items.iter().filter(|i| has_priority(i)).count();
    let items_with_scores = all_items.iter().filter(|i| has_score(i)).count();

    println!("Items with real names: {}", items_with_names);
    println!("Items with real priorities: {}", items_with_priorities);
    println!("Items with real scores: {}", items_with_scores);

    assert_eq!(items_with_names, 3, "3 items have real names");
    assert_eq!(items_with_priorities, 3, "3 items have real priorities");
    assert_eq!(items_with_scores, 2, "2 items have real scores");

    println!("\n✅ Sentinel value approach successfully demonstrates queryable 'null' values!");
}
