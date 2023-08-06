mod tests;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use struct_db::*;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[struct_db(fn_primary_key(generate_my_primary_key))]
struct Item {
    id: u32,
    name: String,
}

impl Item {
    pub fn generate_my_primary_key(&self) -> Vec<u8> {
        format!("{}-{}", self.id, self.name).into()
    }
}

#[test]
fn multi_threads() {
    let tf = tests::init();

    let mut db = Db::create(tf.path("test").as_std_path()).unwrap();
    db.define::<Item>();

    let db = Arc::new(db);

    let db_thread_1 = db.clone();
    let handle_thread_1 = std::thread::spawn(move || {
        let item_a = Item {
            id: 1,
            name: "a".to_string(),
        };
        {
            let tx_write = db_thread_1.transaction().unwrap();
            {
                let mut tables = tx_write.tables();
                tables.insert(&tx_write, item_a).unwrap();
            }
            tx_write.commit().unwrap();
        }
    });

    let db_thread_2 = db.clone();
    let handle_thread_2 = std::thread::spawn(move || {
        let item_b = Item {
            id: 1,
            name: "b".to_string(),
        };
        {
            let tx_write = db_thread_2.transaction().unwrap();
            {
                let mut tables = tx_write.tables();
                tables.insert(&tx_write, item_b).unwrap();
            }
            tx_write.commit().unwrap();
        }
    });

    handle_thread_1.join().unwrap();
    handle_thread_2.join().unwrap();

    {
        let txn_read = db.read_transaction().unwrap();
        let len = txn_read.tables().len::<Item>(&txn_read).unwrap();
        assert_eq!(len, 2);

        let item_a = txn_read
            .tables()
            .primary_get::<Item>(&txn_read, b"1-a")
            .unwrap()
            .unwrap();
        assert_eq!(item_a.name, "a".to_string());

        let item_b = txn_read
            .tables()
            .primary_get::<Item>(&txn_read, b"1-b")
            .unwrap()
            .unwrap();
        assert_eq!(item_b.name, "b".to_string());
    }
}
