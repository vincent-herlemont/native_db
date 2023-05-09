//! Struct DB is a Rust library that provides a simple, fast, and embedded database solution,
//! focusing on maintaining coherence between Rust types and stored data with minimal boilerplate.
//! It supports multiple indexes, real-time watch with filters, schema migration.
//!
//! Use macro `struct_db`:
//!
//! - required: `fn_primary_key(<function name>)` associates a function of the struct that generates the **primary key** of the struct. Allows **only one** `fn_primary_key` declaration.
//! - optional: `fn_secondary_key(<function name>)` associates a function of the struct that generates a **secondary key** of the struct. Allows **multiple** `fn_secondary_key` declarations.
//!
//! `struct_db` generates an enum `<your_type>` with the suffix `Key` that contains all the secondary keys like: E.g. `<your_type>Key::<your_secondary_key>` more details [`here`](crate::ReadableTable::secondary_get).
//!
//! ## API
//! - Initialize a database:
//!    - [`Db::init_tmp(<database name>)`](crate::Db::init_tmp) initializes a database at a temporary path.
//!    - [`Db::init(<path>)`](crate::Db::init) initializes a database at a given path.
//! - Initialize schema
//!    - [`Db::add_schema(`](crate::Db::add_schema)[`<your_item>::struct_db_schema()`](crate::SDBItem::struct_db_schema)`)` initializes a schema.
//! - Transactions
//!    - [`db.transaction()`](crate::Db::transaction) starts a read-write transaction.
//!    - [`db.read_transaction()`](crate::Db::read_transaction) starts a read-only transaction.
//! - Tables
//!    - [`transaction.tables()`](crate::Transaction::tables) returns a [`Tables`](crate::Tables)
//!    - [`read_only_transaction::tables()`](crate::ReadOnlyTransaction::tables) returns a [`ReadOnlyTables`](crate::ReadOnlyTables).
//! - Write operations
//!    - [`tables.insert(&txn,<item>)`](crate::Tables::insert) inserts an item into the database.
//!    - [`tables.update(&txn,<old_item>, <new_item>)`](crate::Tables::update) updates an item in the database.
//!    - [`tables.remove(&txn,<item>)`](crate::Tables::remove) removes an item from the database.
//!    - [`tables.migrate::<old_type, new_type>(&txn)`](crate::Tables::migrate) migrates the schema from `old_type` to `new_type`.
//! - Read operations by
//!    - Primary key
//!       - [`tables.primary_get(&txn,<value>)`](crate::ReadableTable::primary_get) get an item.
//!       - [`tables.primary_iter(&txn)`](crate::ReadableTable::primary_iter) iterate all items.
//!       - [`tables.primary_iter_range(&txn,<start_value>..<end_value>)`](crate::ReadableTable::primary_iter_range) all items in range.
//!       - [`tables.primary_iter_start_with(&txn,<prefix_value>)`](crate::ReadableTable::primary_iter_start_with) all items with prefix.
//!    - Secondary key
//!       - [`tables.secondary_get(&txn,<key_def>,<value>)`](crate::ReadableTable::secondary_get) get an item.
//!       - [`tables.secondary_iter(&txn,<key_def>,<key_def>)`](crate::ReadableTable::secondary_iter) iterate all items.
//!       - [`tables.secondary_iter_range(&txn,<key_def>,<start_value>..<end_value>)`](crate::ReadableTable::secondary_iter_range) all items in range.
//!       - [`tables.secondary_iter_start_with(&txn,<key_def>,<prefix_value>)`](crate::ReadableTable::secondary_iter_start_with) all items with prefix.
//!    - Global
//!       - [`tables.len()`](crate::ReadableTable::len)
//! - Watch use [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver) to receive [`watch::Event`](crate::watch::Event).
//!    - Primary key
//!       - [`db.primary_watch(Option<<value>>)`](crate::Db::primary_watch) watch all or a specific item.
//!       - [`db.primary_watch_start_with(<prefix_value>)`](crate::Db::primary_watch_start_with) watch all items with prefix.
//!    - Secondary key
//!       - [`db.secondary_watch(<key_def>,Option<value>)`](crate::Db::secondary_watch) watch all or a specific item.
//!       - [`db.secondary_watch_start_with(<key_def>,<prefix_value>)`](crate::Db::secondary_watch_start_with) watch all items with prefix.
//!    - Global
//!       - [`db.unwatch(<watcher_id>)`](crate::Db::unwatch) stop watching a specific watcher.
//! # Example
//! ```
//! use serde::{Deserialize, Serialize};
//! use struct_db::*;
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! #[struct_db(
//!     fn_primary_key(p_key),
//!     fn_secondary_key(s_key),
//! )]
//! struct Data(u32, String);
//!
//! impl Data {
//!     // `p_key` returns the primary key of the `Data` struct as a vector of bytes.
//!     // In this case, it is the big-endian byte representation of the `i32` value.
//!     // Using big-endian representation for the primary key maintains a consistent
//!     // lexicographical ordering of the keys, which is useful for ordered key-value
//!     // stores and efficient range queries.
//!    pub fn p_key(&self) -> Vec<u8> {
//!        self.0.to_be_bytes().to_vec()
//!    }
//!   
//!     // `s_key` generates a secondary key for the `Data` struct as a vector of bytes.
//!     // The secondary key consists of the big-endian byte representation of the `i32` value
//!     // (the primary key) followed by the String field. This combined key allows for more
//!     // versatile querying options.
//!    pub fn s_key(&self) -> Vec<u8> {
//!        let mut p_key = self.p_key();
//!        p_key.extend(self.1.as_bytes());
//!        p_key
//!    }
//! }
//!
//! fn main() {
//!     let mut db = Db::init_tmp("my_db_example").unwrap();
//!     // Initialize the schema
//!     db.add_schema(Data::struct_db_schema());
//!
//!     let data = Data(1,"test".to_string());
//!     // Insert data
//!     let txn = db.transaction().unwrap();
//!     {
//!       let mut tables = txn.tables();
//!       tables.insert(&txn, data).unwrap();
//!     }
//!     txn.commit().unwrap();
//!
//!     // Get data
//!     let txn_read = db.read_transaction().unwrap();
//!     let retrieve_data: Data = txn_read.tables().primary_get(&txn_read, &1_u32.to_be_bytes()).unwrap().unwrap();
//!     assert_eq!(&retrieve_data, &Data(1,"test".to_string()));
//!   
//!     // Remove data
//!     let txn = db.transaction().unwrap();
//!     {
//!       let mut tables = txn.tables();
//!       tables.remove(&txn, retrieve_data).unwrap();
//!     }
//!     txn.commit().unwrap();
//! }
//! ```

mod common;
mod db;
mod item;
mod iterator;
mod readable_table;
mod readonly_tables;
mod readonly_transaction;
mod schema;
mod serialization;
mod tables;
mod transaction;
pub mod watch;

pub use db::*;
pub use item::*;
pub use iterator::*;
pub use readable_table::*;
pub use readonly_tables::*;
pub use readonly_transaction::*;
pub use schema::*;
pub use serialization::*;
use std::path::PathBuf;
pub use struct_db_macro::*;
pub use tables::*;
pub use transaction::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Redb error")]
    Redb(#[from] redb::Error),

    #[error("Init database error {path:?}")]
    InitDbBackendError { source: redb::Error, path: PathBuf },

    #[error("Table definition not found {table}")]
    TableDefinitionNotFound { table: String },

    #[error("Key not found {key:?}")]
    KeyNotFound { key: Vec<u8> },

    #[error("Primary key associated with the secondary key not found {secondary_key:?}")]
    PrimaryKeyNotFound { secondary_key: Vec<u8> },

    #[error("Duplicate key for \"{key_name}\"")]
    DuplicateKey { key_name: &'static str },

    #[error("Watch event error")]
    WatchEventError(#[from] watch::WatchEventError),

    #[error("Max watcher reached (should be impossible)")]
    MaxWatcherReached,
}

#[cfg(feature = "eyre_support")]
use eyre::Report;
#[cfg(feature = "eyre_support")]
pub type ReportError = Report;
#[cfg(feature = "eyre_support")]
pub type Result<T> = std::result::Result<T, ReportError>;

#[cfg(not(feature = "eyre_support"))]
pub type Result<T> = std::result::Result<T, Error>;
