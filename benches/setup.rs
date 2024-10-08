use std::{cell::RefCell, fmt::Debug, rc::Rc};

use native_db::*;
use native_model::{native_model, Model};
use once_cell::sync::Lazy;
use rand::Rng;
use rusqlite::TransactionBehavior;
use serde::{Deserialize, Serialize};
use shortcut_assert_fs::TmpFs;

pub trait Item {
    fn generate_sqlite_table() -> String;
    fn generate_sqlite_insert(&self) -> String;
    fn generate_select_range_sk(sk_name: &str) -> String;
    fn get_pk(&self) -> i64;
    fn update_pk(&mut self, pk: i64);
    fn update_with_random(&mut self);
}

pub const REDB_TABLE: redb::TableDefinition<i64, Vec<u8>> = redb::TableDefinition::new("item");

macro_rules! define_item_struct {
    ($struct_name:ident, $id:expr, $($secondary_key:ident),*) => {
        #[derive(Serialize, Deserialize, Clone, Default, Debug)]
        #[native_model(id = $id, version = 1)]
        #[native_db]
        pub struct $struct_name {
            #[primary_key]
            pub pk: i64,
            $(
                #[secondary_key]
                pub $secondary_key: i64,
            )*
        }

        impl Item for $struct_name {

            fn update_with_random(&mut self) {
                // Random between 0 and 100
                $(
                    let mut rng = rand::thread_rng();
                    self.$secondary_key = rng.gen_range(0..50);
                )*
            }

            fn get_pk(&self) -> i64 {
                self.pk
            }

            fn update_pk(&mut self, pk: i64) {
                self.pk = pk;
            }

            fn generate_sqlite_table() -> String {
                let mut sql = String::new();
                sql.push_str("CREATE TABLE IF NOT EXISTS ");
                sql.push_str(stringify!($struct_name));
                sql.push_str(" (pk INTEGER PRIMARY KEY");
                // Add a binary column
                sql.push_str(", binary BLOB NOT NULL");
                $(
                    sql.push_str(",");
                    sql.push_str(stringify!($secondary_key));
                    sql.push_str(" INTEGER NOT NULL");
                )*
                sql.push_str("
                );");
                // Alter table to add indexes for secondary keys
                $(
                    sql.push_str("CREATE INDEX IF NOT EXISTS ");
                    sql.push_str(&format!("{}_{}_index", stringify!($struct_name), stringify!($secondary_key)));
                    sql.push_str(" ON ");
                    sql.push_str(stringify!($struct_name));
                    sql.push_str(" (");
                    sql.push_str(stringify!($secondary_key));
                    sql.push_str(")");
                )*

                sql
            }

            fn generate_sqlite_insert(&self) -> String {
                let mut sql = String::new();
                sql.push_str("INSERT INTO ");
                sql.push_str(stringify!($struct_name));
                sql.push_str(" (pk");
                sql.push_str(", binary");
                $(
                    sql.push_str(", ");
                    sql.push_str(stringify!($secondary_key));
                )*
                sql.push_str(") VALUES (");
                sql.push_str(&self.pk.to_string());
                sql.push_str(", ?");
                $(
                    sql.push_str(", ");
                    sql.push_str(&self.$secondary_key.to_string());
                )*
                sql.push_str(")");
                sql
            }

            fn generate_select_range_sk(sk_name: &str) -> String {
                let mut sql = String::new();
                sql.push_str("SELECT * FROM ");
                sql.push_str(stringify!($struct_name));
                sql.push_str(" WHERE ");
                sql.push_str(sk_name);
                sql.push_str(" >= :from_sk AND ");
                sql.push_str(sk_name);
                sql.push_str(" <= :to_sk");
                sql
            }
        }
    };
}

// 1 SK
#[rustfmt::skip]
define_item_struct!(Item1SK_NUni_NOpt, 1, sk_1);

// 10 SK
#[rustfmt::skip]
define_item_struct!(Item10SK_NUni_NOpt, 2,
                    sk_1, sk_2, sk_3, sk_4,
                    sk_5, sk_6, sk_7, sk_8, 
                    sk_9, sk_10);
// 50 SK
#[rustfmt::skip]
define_item_struct!(Item50SK_NUni_NOpt, 3,
                    sk_1, sk_2, sk_3, sk_4,
                    sk_5, sk_6, sk_7, sk_8, 
                    sk_9, sk_10, sk_11, sk_12, 
                    sk_13, sk_14, sk_15, sk_16, 
                    sk_17, sk_18, sk_19, sk_20, 
                    sk_21, sk_22, sk_23, sk_24, 
                    sk_25, sk_26, sk_27, sk_28, 
                    sk_29, sk_30, sk_31, sk_32, 
                    sk_33, sk_34, sk_35, sk_36, 
                    sk_37, sk_38, sk_39, sk_40, 
                    sk_41, sk_42, sk_43, sk_44, 
                    sk_45, sk_46, sk_47, sk_48, 
                    sk_49, sk_50);
// 100 SK
#[rustfmt::skip]
define_item_struct!(Item100SK_NUni_NOpt, 4,
                    sk_1, sk_2, sk_3, sk_4,
                    sk_5, sk_6, sk_7, sk_8, 
                    sk_9, sk_10, sk_11, sk_12, 
                    sk_13, sk_14, sk_15, sk_16, 
                    sk_17, sk_18, sk_19, sk_20, 
                    sk_21, sk_22, sk_23, sk_24, 
                    sk_25, sk_26, sk_27, sk_28, 
                    sk_29, sk_30, sk_31, sk_32, 
                    sk_33, sk_34, sk_35, sk_36, 
                    sk_37, sk_38, sk_39, sk_40, 
                    sk_41, sk_42, sk_43, sk_44, 
                    sk_45, sk_46, sk_47, sk_48, 
                    sk_49, sk_50, sk_51, sk_52, 
                    sk_53, sk_54, sk_55, sk_56, 
                    sk_57, sk_58, sk_59, sk_60, 
                    sk_61, sk_62, sk_63, sk_64,
                    sk_65, sk_66, sk_67, sk_68, 
                    sk_69, sk_70, sk_71, sk_72, 
                    sk_73, sk_74, sk_75, sk_76, 
                    sk_77, sk_78, sk_79, sk_80, 
                    sk_81, sk_82, sk_83, sk_84, 
                    sk_85, sk_86, sk_87, sk_88, 
                    sk_89, sk_90, sk_91, sk_92, 
                    sk_93, sk_94, sk_95, sk_96, 
                    sk_97, sk_98, sk_99, sk_100);

pub trait BenchDatabase {
    type DB;

    fn setup() -> Self;
    fn insert<T: native_db::ToInput + Item>(&self, item: T);
    fn db(&self) -> &Self::DB;
    fn insert_bulk<T: native_db::ToInput + Item + Default + Debug>(&self, items: Vec<T>);
    fn insert_bulk_random<T: native_db::ToInput + Item + Default + Clone + Debug>(&self, n: usize);
}

pub struct NativeDBBenchDatabase {
    tmp: TmpFs,
    db: Database<'static>,
}

static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<Item1SK_NUni_NOpt>().unwrap();
    models.define::<Item10SK_NUni_NOpt>().unwrap();
    models.define::<Item50SK_NUni_NOpt>().unwrap();
    models.define::<Item100SK_NUni_NOpt>().unwrap();
    models
});

impl BenchDatabase for NativeDBBenchDatabase {
    type DB = Database<'static>;
    fn setup() -> Self {
        let tmp = TmpFs::new().unwrap();
        let db_path = tmp.path("native_db_bench");
        let db = Builder::new().set_cache_size(0).create(&MODELS, db_path.clone()).unwrap();
        Self { tmp, db }
    }

    fn insert_bulk<T: native_db::ToInput + Item + Debug>(&self, items: Vec<T>) {
        let rw = self.db.rw_transaction().unwrap();
        for item in items {
            rw.insert(item).unwrap();
        }
        rw.commit().unwrap();
    }

    fn insert_bulk_random<T: native_db::ToInput + Item + Default + Clone + Debug>(&self, n: usize) {
        let mut items = vec![T::default(); n];
        for (usize, item) in &mut items.iter_mut().enumerate() {
            item.update_with_random();
            item.update_pk(usize as i64);
        }
        self.insert_bulk(items);
    }

    fn db(&self) -> &Self::DB {
        &self.db
    }

    fn insert<T: native_db::ToInput>(&self, item: T) {
        let rw = self.db.rw_transaction().unwrap();
        rw.insert(item).unwrap();
        rw.commit().unwrap();
    }
}

pub struct SqliteBenchDatabase {
    tmp: TmpFs,
    db: Rc<RefCell<rusqlite::Connection>>,
}

impl BenchDatabase for SqliteBenchDatabase {
    type DB = Rc<RefCell<rusqlite::Connection>>;

    fn setup() -> Self {
        let tmp = TmpFs::new().unwrap();
        let db_path = tmp.path("sqlite_bench");
        let db: rusqlite::Connection = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .unwrap();
        db.set_prepared_statement_cache_capacity(0);
        //db.pragma_update(None, "journal_mode", &"DELETE").unwrap();
        //db.pragma_update(None, "synchronous", &"OFF").unwrap();
        db.pragma_update(None, "cache_size", &"0").unwrap();
        //db.pragma_update(None, "foreign_keys", &"ON").unwrap();
        db.execute(&Item1SK_NUni_NOpt::generate_sqlite_table(), ())
            .unwrap();
        db.execute(&Item10SK_NUni_NOpt::generate_sqlite_table(), ())
            .unwrap();
        db.execute(&Item50SK_NUni_NOpt::generate_sqlite_table(), ())
            .unwrap();
        db.execute(&Item100SK_NUni_NOpt::generate_sqlite_table(), ())
            .unwrap();
        Self {
            tmp,
            db: Rc::new(RefCell::new(db)),
        }
    }

    fn insert_bulk<T: native_db::ToInput + Item + Default>(&self, items: Vec<T>) {
        let mut db = self.db.borrow_mut();
        let transaction = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        for item in items {
            let binary = item.native_model_encode().unwrap();
            transaction
                .execute(&item.generate_sqlite_insert(), (binary,))
                .unwrap();
        }
        transaction.commit().unwrap();
        db.cache_flush().unwrap();
    }

    fn insert_bulk_random<T: native_db::ToInput + Item + Default + Clone + Debug>(&self, n: usize) {
        let mut items = vec![T::default(); n];
        for (usize, item) in &mut items.iter_mut().enumerate() {
            item.update_with_random();
            item.update_pk(usize as i64);
        }
        self.insert_bulk(items);
    }

    fn db(&self) -> &Self::DB {
        &self.db
    }

    fn insert<T: native_db::ToInput + Item>(&self, item: T) {
        let mut db = self.db.borrow_mut();
        let transaction = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let binary = item.native_model_encode().unwrap();
        transaction
            .execute(&item.generate_sqlite_insert(), (binary,))
            .unwrap();
        transaction.commit().unwrap();
        db.cache_flush().unwrap();
    }
}

pub struct RedbBenchDatabase {
    tmp: TmpFs,
    db: redb::Database,
}

impl BenchDatabase for RedbBenchDatabase {
    type DB = redb::Database;

    fn setup() -> Self {
        let tmp = TmpFs::new().unwrap();
        let db_path = tmp.path("redb_bench");
        let db = redb::Database::create(&db_path).unwrap();
        Self { tmp, db }
    }

    fn insert_bulk<T: native_db::ToInput + Item + Default>(&self, items: Vec<T>) {
        let rw = self.db.begin_write().unwrap();
        {
            let mut table = rw.open_table(REDB_TABLE).unwrap();
            for item in items {
                let binary = item.native_model_encode().unwrap();
                table.insert(item.get_pk(), binary).unwrap();
            }
        }
        rw.commit().unwrap();
    }

    fn insert_bulk_random<T: native_db::ToInput + Item + Default + Clone + Debug>(&self, n: usize) {
        let mut data = vec![T::default(); n];
        for (usize, item) in &mut data.iter_mut().enumerate() {
            item.update_with_random();
            item.update_pk(usize as i64);
        }
        self.insert_bulk(data);
    }

    fn db(&self) -> &Self::DB {
        &self.db
    }

    fn insert<T: native_db::ToInput + Item>(&self, item: T) {
        let binary = item.native_model_encode().unwrap();
        let rw = self.db.begin_write().unwrap();
        {
            let mut table = rw.open_table(REDB_TABLE).unwrap();
            table.insert(item.get_pk(), binary).unwrap();
        }
        rw.commit().unwrap();
    }
}
