#![cfg(not(feature = "native_model"))]
mod tests;

use serde::{Deserialize, Serialize};
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(
    pk = generate_my_primary_key,
    gk = secondary_key_1,
    gk = secondary_key_2
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
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }
    pub fn secondary_key_1(&self) -> Vec<u8> {
        format!("{}", self.id).into()
    }
    pub fn secondary_key_2(&self) -> Vec<u8> {
        format!("{}", self.name).into()
    }
}

#[test]
fn test_iter() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let result: Vec<Item> = tables.primary_iter(&txn).unwrap().collect();
        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 1);
        assert_eq!(obj1.name, "test");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 2);
        assert_eq!(obj2.name, "test2");
    }
}

// Check if the use of BigEndian is correct
#[test]
fn test_iter_many_items_to_be_bytes() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        // Insert 1000 items
        for i in 0..257 {
            tables
                .insert(&txn, Item::new(i, format!("test_{}", i).as_str()))
                .unwrap();
        }
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let iter: Vec<Item> = tables.primary_iter(&txn).unwrap().collect();
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
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let iter = tables.primary_iter(&txn).unwrap();
        let result: Vec<Item> = iter.rev().collect();

        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();

        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 1);
        assert_eq!(obj2.name, "test");
    }
}

#[test]
fn test_iter_range() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
        tables.insert(&txn, Item::new(3, "test3")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let result: Vec<Item> = tables
            .primary_iter_range(&txn, ..2_i32.to_be_bytes().as_slice())
            .unwrap()
            .collect();
        assert_eq!(result.len(), 1);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 1);
        assert_eq!(obj1.name, "test");

        let result: Vec<Item> = tables
            .primary_iter_range(&txn, 2_i32.to_be_bytes().as_slice()..)
            .unwrap()
            .collect();
        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 3);
        assert_eq!(obj2.name, "test3");

        let result: Vec<Item> = tables
            .primary_iter_range(
                &txn,
                2_i32.to_be_bytes().as_slice()..3_i32.to_be_bytes().as_slice(),
            )
            .unwrap()
            .collect();
        assert_eq!(result.len(), 1);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");
    }
}

#[test]
fn test_iter_by_key() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let result: Vec<Item> = tables
            .secondary_iter(&txn, ItemKey::secondary_key_1)
            .unwrap()
            .collect();

        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 1);
        assert_eq!(obj1.name, "test");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 2);
        assert_eq!(obj2.name, "test2");
    }
}

#[test]
fn test_double_ended_iter_by_key() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let iter = tables
            .secondary_iter(&txn, ItemKey::secondary_key_1)
            .unwrap();
        let result: Vec<Item> = iter.rev().collect();

        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 1);
        assert_eq!(obj2.name, "test");
    }
}

#[test]
fn test_double_ended_iter_by_key_range() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
        tables.insert(&txn, Item::new(3, "test3")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.read_transaction().unwrap();
    {
        let mut tables = txn.tables();
        let iter = tables
            .secondary_iter_range(&txn, ItemKey::secondary_key_1, ..b"2".as_slice())
            .unwrap();
        let result: Vec<Item> = iter.rev().collect();

        assert_eq!(result.len(), 1);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 1);
        assert_eq!(obj1.name, "test");

        let iter = tables
            .secondary_iter_range(&txn, ItemKey::secondary_key_1, b"2".as_slice()..)
            .unwrap();
        let result: Vec<Item> = iter.rev().collect();

        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 3);
        assert_eq!(obj1.name, "test3");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 2);
        assert_eq!(obj2.name, "test2");

        let iter = tables
            .secondary_iter_range(
                &txn,
                ItemKey::secondary_key_1,
                b"2".as_slice()..b"3".as_slice(),
            )
            .unwrap();
        let result: Vec<Item> = iter.rev().collect();

        assert_eq!(result.len(), 1);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(pk = generate_my_primary_key)]
struct ItemFlag {
    name: String,
}

impl ItemFlag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        self.name.clone().into()
    }
}

#[test]
fn test_start_with_scenario() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<ItemFlag>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        // Red flag
        tables.insert(&txn, ItemFlag::new("red:1")).unwrap();
        tables.insert(&txn, ItemFlag::new("red:2")).unwrap();
        tables.insert(&txn, ItemFlag::new("red:3")).unwrap();
        // Blue flag
        tables.insert(&txn, ItemFlag::new("blue:1")).unwrap();
        tables.insert(&txn, ItemFlag::new("blue:2")).unwrap();
        tables.insert(&txn, ItemFlag::new("blue:3")).unwrap();
        // Green flag
        tables.insert(&txn, ItemFlag::new("green:1")).unwrap();
        tables.insert(&txn, ItemFlag::new("green:2")).unwrap();
        tables.insert(&txn, ItemFlag::new("green:3")).unwrap();
    }
    txn.commit().unwrap();

    let prefix = [
        b"red:".as_slice(),
        b"blue:".as_slice(),
        b"green:".as_slice(),
    ];
    for p in prefix.iter() {
        let txn = db.read_transaction().unwrap();
        {
            let mut tables = txn.tables();
            let iter = tables.primary_iter_start_with(&txn, p).unwrap();
            let result: Vec<ItemFlag> = iter.collect();
            assert_eq!(result.len(), 3);

            let obj1 = result.get(0).unwrap();
            assert_eq!(obj1.name, format!("{}1", std::str::from_utf8(p).unwrap()));

            let obj2 = result.get(1).unwrap();
            assert_eq!(obj2.name, format!("{}2", std::str::from_utf8(p).unwrap()));

            let obj3 = result.get(2).unwrap();
            assert_eq!(obj3.name, format!("{}3", std::str::from_utf8(p).unwrap()));
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(pk = generate_my_primary_key, gk = flag)]
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

    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        self.id.clone().into()
    }
    pub fn flag(&self) -> Vec<u8> {
        format!("{}:{}", self.flag, self.id).into()
    }
}

#[test]
fn test_start_with_by_key_scenario_write_txn() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<ItemIdFlag>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        // Red flag
        tables.insert(&txn, ItemIdFlag::new("1", "red")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("2", "red")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("3", "red")).unwrap();
        // Blue flag
        tables.insert(&txn, ItemIdFlag::new("4", "blue")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("5", "blue")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("6", "blue")).unwrap();
        // Green flag
        tables.insert(&txn, ItemIdFlag::new("7", "green")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("8", "green")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("9", "green")).unwrap();
    }
    txn.commit().unwrap();

    let prefix = [
        b"red:".as_slice(),
        b"blue:".as_slice(),
        b"green:".as_slice(),
    ];
    for p in prefix.iter() {
        let txn = db.transaction().unwrap();
        {
            let mut tables = txn.tables();
            let iter = tables
                .secondary_iter_start_with(&txn, ItemIdFlagKey::flag, p)
                .unwrap();
            let result: Vec<ItemIdFlag> = iter.collect();
            assert_eq!(result.len(), 3);

            let obj1 = result.get(0).unwrap();
            assert_eq!(
                format!("{}:", obj1.flag),
                format!("{}", std::str::from_utf8(p).unwrap())
            );

            let obj2 = result.get(1).unwrap();
            assert_eq!(
                format!("{}:", obj2.flag),
                format!("{}", std::str::from_utf8(p).unwrap())
            );

            let obj3 = result.get(2).unwrap();
            assert_eq!(
                format!("{}:", obj3.flag),
                format!("{}", std::str::from_utf8(p).unwrap())
            );
        }
    }
}

#[test]
fn test_start_with_by_key_scenario_readonly_txn() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<ItemIdFlag>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        // Red flag
        tables.insert(&txn, ItemIdFlag::new("1", "red")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("2", "red")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("3", "red")).unwrap();
        // Blue flag
        tables.insert(&txn, ItemIdFlag::new("4", "blue")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("5", "blue")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("6", "blue")).unwrap();
        // Green flag
        tables.insert(&txn, ItemIdFlag::new("7", "green")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("8", "green")).unwrap();
        tables.insert(&txn, ItemIdFlag::new("9", "green")).unwrap();
    }
    txn.commit().unwrap();

    let prefix = [
        b"red:".as_slice(),
        b"blue:".as_slice(),
        b"green:".as_slice(),
    ];
    for p in prefix.iter() {
        let txn = db.read_transaction().unwrap();
        {
            let mut tables = txn.tables();
            let iter = tables
                .secondary_iter_start_with(&txn, ItemIdFlagKey::flag, p)
                .unwrap();
            let result: Vec<ItemIdFlag> = iter.collect();
            assert_eq!(result.len(), 3);

            let obj1 = result.get(0).unwrap();
            assert_eq!(
                format!("{}:", obj1.flag),
                format!("{}", std::str::from_utf8(p).unwrap())
            );

            let obj2 = result.get(1).unwrap();
            assert_eq!(
                format!("{}:", obj2.flag),
                format!("{}", std::str::from_utf8(p).unwrap())
            );

            let obj3 = result.get(2).unwrap();
            assert_eq!(
                format!("{}:", obj3.flag),
                format!("{}", std::str::from_utf8(p).unwrap())
            );
        }
    }
}

#[test]
fn test_txn_write_iter() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let result: Vec<Item> = tables.primary_iter(&txn).unwrap().collect();
        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 1);
        assert_eq!(obj1.name, "test");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 2);
        assert_eq!(obj2.name, "test2");
    }
}

#[test]
fn test_txn_write_iter_range() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<Item>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        tables.insert(&txn, Item::new(1, "test")).unwrap();
        tables.insert(&txn, Item::new(2, "test2")).unwrap();
        tables.insert(&txn, Item::new(3, "test3")).unwrap();
    }
    txn.commit().unwrap();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        let result: Vec<Item> = tables
            .primary_iter_range(&txn, ..2_i32.to_be_bytes().as_slice())
            .unwrap()
            .collect();
        assert_eq!(result.len(), 1);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 1);
        assert_eq!(obj1.name, "test");

        let result: Vec<Item> = tables
            .primary_iter_range(&txn, 2_i32.to_be_bytes().as_slice()..)
            .unwrap()
            .collect();
        assert_eq!(result.len(), 2);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");

        let obj2 = result.get(1).unwrap();
        assert_eq!(obj2.id, 3);
        assert_eq!(obj2.name, "test3");

        let result: Vec<Item> = tables
            .primary_iter_range(
                &txn,
                2_i32.to_be_bytes().as_slice()..3_i32.to_be_bytes().as_slice(),
            )
            .unwrap()
            .collect();
        assert_eq!(result.len(), 1);

        let obj1 = result.get(0).unwrap();
        assert_eq!(obj1.id, 2);
        assert_eq!(obj1.name, "test2");
    }
}

#[test]
fn test_txn_write_start_with_scenario() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();

    db.define::<ItemFlag>();

    let txn = db.transaction().unwrap();
    {
        let mut tables = txn.tables();
        // Red flag
        tables.insert(&txn, ItemFlag::new("red:1")).unwrap();
        tables.insert(&txn, ItemFlag::new("red:2")).unwrap();
        tables.insert(&txn, ItemFlag::new("red:3")).unwrap();
        // Blue flag
        tables.insert(&txn, ItemFlag::new("blue:1")).unwrap();
        tables.insert(&txn, ItemFlag::new("blue:2")).unwrap();
        tables.insert(&txn, ItemFlag::new("blue:3")).unwrap();
        // Green flag
        tables.insert(&txn, ItemFlag::new("green:1")).unwrap();
        tables.insert(&txn, ItemFlag::new("green:2")).unwrap();
        tables.insert(&txn, ItemFlag::new("green:3")).unwrap();
    }
    txn.commit().unwrap();

    let prefix = [
        b"red:".as_slice(),
        b"blue:".as_slice(),
        b"green:".as_slice(),
    ];
    for p in prefix.iter() {
        let txn = db.transaction().unwrap();
        {
            let mut tables = txn.tables();
            let iter = tables.primary_iter_start_with(&txn, p).unwrap();
            let result: Vec<ItemFlag> = iter.collect();
            assert_eq!(result.len(), 3);

            let obj1 = result.get(0).unwrap();
            assert_eq!(obj1.name, format!("{}1", std::str::from_utf8(p).unwrap()));

            let obj2 = result.get(1).unwrap();
            assert_eq!(obj2.name, format!("{}2", std::str::from_utf8(p).unwrap()));

            let obj3 = result.get(2).unwrap();
            assert_eq!(obj3.name, format!("{}3", std::str::from_utf8(p).unwrap()));
        }
    }
}
