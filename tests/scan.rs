// TODO: refactor and move to query/ folder

use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;
use native_db::db_type::Result;
use itertools::Itertools;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db(
    primary_key(generate_my_primary_key),
    secondary_key(secondary_key_1, unique),
    secondary_key(secondary_key_2, unique)
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
        format!("{}", self.id).into()
    }
    pub fn secondary_key_2(&self) -> String {
        format!("{}", self.name)
    }
}

#[test]
fn test_iter() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Vec<Item>= r.scan().primary().unwrap().all().try_collect().unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
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

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    // Insert 1000 items
    for i in 0..257 {
        rw.insert(Item::new(i, format!("test_{}", i).as_str()))
            .unwrap();
    }
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    {
        let iter: Vec<Item> = r.scan().primary().unwrap().all().try_collect().unwrap();
        assert_eq!(iter.len(), 257);

        let obj1 = iter.get(0).unwrap();
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

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let scan = r.scan().primary().unwrap();
    let iter = scan.all();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();

    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 1);
    assert_eq!(obj2.name, "test");
}

#[test]
fn test_iter_range() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.insert(Item::new(3, "test3")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let result: Vec<Item> = r.scan().primary().unwrap().range(..2_i32).try_collect().unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let result: Vec<Item> = r.scan().primary().unwrap().range(2_i32..).try_collect().unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 3);
    assert_eq!(obj2.name, "test3");

    let result: Vec<Item> = r.scan().primary().unwrap().range(2_i32..3_i32).try_collect().unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");
}

#[test]
fn test_iter_by_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

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
        .try_collect().unwrap();

    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");
}

#[test]
fn test_double_ended_iter_by_key() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.all();
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 1);
    assert_eq!(obj2.name, "test");
}

#[test]
fn test_double_ended_iter_by_key_range() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.insert(Item::new(3, "test3")).unwrap();
    rw.commit().unwrap();

    let r = db.r_transaction().unwrap();
    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.range(..b"2".as_slice());
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 1);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.range(b"2".as_slice()..);
    let result: Vec<Item> = iter.rev().try_collect().unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 3);
    assert_eq!(obj1.name, "test3");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");

    let scan = r.scan().secondary(ItemKey::secondary_key_1).unwrap();
    let iter = scan.range(b"2".as_slice()..b"3".as_slice());
    let result: Vec<Item> = iter.rev().try_collect().unwrap();

    assert_eq!(result.len(), 1);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 2, version = 1)]
#[native_db(primary_key(generate_my_primary_key))]
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

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemFlag>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

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
            .try_collect().unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.name, format!("{}1", p));

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.name, format!("{}2", p));

        let obj3 = result.get(2).unwrap();
        assert_eq!(obj3.name, format!("{}3", p));
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db(primary_key(generate_my_primary_key), secondary_key(flag, unique))]
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

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemIdFlag>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

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
            .try_collect().unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.get(0).unwrap();
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

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemIdFlag>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

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
            .try_collect().unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.get(0).unwrap();
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

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

    let rw = db.rw_transaction().unwrap();
    rw.insert(Item::new(1, "test")).unwrap();
    rw.insert(Item::new(2, "test2")).unwrap();
    rw.commit().unwrap();

    let rw = db.rw_transaction().unwrap();
    let result: Vec<Item> = rw.scan().primary().unwrap().all().try_collect().unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, "test2");
}

#[test]
fn test_txn_write_iter_range() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<Item>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

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
        .range(..2_i32.to_be_bytes().as_slice())
        .try_collect().unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, "test");

    let result: Vec<Item> = rw
        .scan()
        .primary()
        .unwrap()
        .range(2_i32.to_be_bytes().as_slice()..)
        .try_collect().unwrap();
    assert_eq!(result.len(), 2);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");

    let obj2 = result.get(1).unwrap();
    assert_eq!(obj2.id, 3);
    assert_eq!(obj2.name, "test3");

    let result: Vec<Item> = rw
        .scan()
        .primary()
        .unwrap()
        .range(2_i32.to_be_bytes().as_slice()..3_i32.to_be_bytes().as_slice())
        .try_collect().unwrap();
    assert_eq!(result.len(), 1);

    let obj1 = result.get(0).unwrap();
    assert_eq!(obj1.id, 2);
    assert_eq!(obj1.name, "test2");
}

#[test]
fn test_txn_write_start_with_scenario() {
    let tf = TmpFs::new().unwrap();

    let mut builder = DatabaseBuilder::new();
    builder.define::<ItemFlag>().unwrap();
    let db = builder.create(tf.path("test").as_std_path()).unwrap();

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
            .try_collect().unwrap();
        assert_eq!(result.len(), 3);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.name, format!("{}1", p));

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.name, format!("{}2", p));

        let obj3 = result.get(2).unwrap();
        assert_eq!(obj3.name, format!("{}3", p));
    }
}
