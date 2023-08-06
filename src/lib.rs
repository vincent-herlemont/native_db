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
//!    - [`Db::init_tmp(<database name>)`](crate::Db::create_tmp) initializes a database at a temporary path.
//!    - [`Db::init(<path>)`](crate::Db::create) initializes a database at a given path.
//! - Define schema
//!    - [`db.define::<your_type>()`](crate::Db::define) initializes a schema.
//! - Transactions
//!    - [`db.transaction()`](crate::Db::transaction) starts a [`read-write transaction`](crate::Transaction).
//!    - [`db.read_transaction()`](crate::Db::read_transaction) starts a [`read-only transaction`](crate::ReadOnlyTransaction).
//! - Tables (`txn` is a [`Transaction`](crate::Transaction) and `read_only_txn` a [`ReadOnlyTransaction`](crate::ReadOnlyTransaction))
//!    - [`txn.tables()`](crate::Transaction::tables) returns a [`Tables`](crate::Tables)
//!    - [`read_only_txn.tables()`](crate::ReadOnlyTransaction::tables) returns a [`ReadOnlyTables`](crate::ReadOnlyTables).
//! - Write operations
//!    - [`tables.insert(&txn,<item>)`](crate::Tables::insert) inserts an item into the database.
//!    - [`tables.update(&txn,<old_item>, <new_item>)`](crate::Tables::update) updates an item in the database.
//!    - [`tables.remove(&txn,<item>)`](crate::Tables::remove) removes an item from the database.
//!    - [`tables.migrate::<old_type, new_type>(&txn)`](crate::Tables::migrate) migrates the schema from `old_type` to `new_type`.
//! - Read operations
//!    - Primary key
//!       - [`tables.primary_get(&txn,<key>)`](crate::ReadableTable::primary_get) get an item.
//!       - [`tables.primary_iter(&txn)`](crate::ReadableTable::primary_iter) iterate all items.
//!       - [`tables.primary_iter_range(&txn,<start_key>..<end_key>)`](crate::ReadableTable::primary_iter_range) all items in range.
//!       - [`tables.primary_iter_start_with(&txn,<key_prefix>)`](crate::ReadableTable::primary_iter_start_with) all items with prefix.
//!    - Secondary key
//!       - [`tables.secondary_get(&txn,<key_def>,<key>)`](crate::ReadableTable::secondary_get) get an item.
//!       - [`tables.secondary_iter(&txn,<key_def>,<key_def>)`](crate::ReadableTable::secondary_iter) iterate all items.
//!       - [`tables.secondary_iter_range(&txn,<key_def>,<start_key>..<end_key>)`](crate::ReadableTable::secondary_iter_range) all items in range.
//!       - [`tables.secondary_iter_start_with(&txn,<key_def>,<key_prefix>)`](crate::ReadableTable::secondary_iter_start_with) all items with prefix.
//!    - Global
//!       - [`tables.len::<your_type>()`](crate::ReadableTable::len)
//! - Watch use [`std::sync::mpsc::Receiver`](std::sync::mpsc::Receiver) or [tokio::sync::mpsc::UnboundedReceiver](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html) to receive [`watch::Event`](crate::watch::Event).
//!    - Primary key
//!       - [`db.primary_watch(Option<key>)`](crate::Db::primary_watch) watch all or a specific item.
//!       - [`db.primary_watch_start_with(<key_prefix>)`](crate::Db::primary_watch_start_with) watch all items with prefix.
//!    - Secondary key
//!       - [`db.secondary_watch(<key_def>,Option<key>)`](crate::Db::secondary_watch) watch all or a specific item.
//!       - [`db.secondary_watch_start_with(<key_def>,<key_prefix>)`](crate::Db::secondary_watch_start_with) watch all items with prefix.
//!    - Global
//!       - [`db.unwatch(<watcher_id>)`](crate::Db::unwatch) stop watching a specific watcher.
//! # Example
//! ```
//! use serde::{Deserialize, Serialize};
//! use struct_db::*;
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! #[struct_db(
//!    fn_primary_key(p_key),  // required
//!    fn_secondary_key(s_key),  // optional
//!    // ... other fn_secondary_key ...
//! )]
//! struct Data(u32, String);
//!
//! impl Data {
//!   // Returns primary key as big-endian bytes for consistent lexicographical ordering.
//!   pub fn p_key(&self) -> Vec<u8> {
//!     self.0.to_be_bytes().to_vec()
//!   }
//!
//!   // Generates a secondary key combining the String field and the big-endian bytes of
//!   // the primary key for versatile queries.
//!   pub fn s_key(&self) -> Vec<u8> {
//!     let mut s_key = self.1.as_bytes().to_vec();
//!     s_key.extend_from_slice(&self.p_key().as_slice());
//!     s_key
//!   }
//!  }
//!
//!  fn main() {
//!   let mut db = Db::create_tmp("my_db_example").unwrap();
//!   // Initialize the schema
//!   db.define::<Data>();
//!
//!   // Insert data
//!   let txn = db.transaction().unwrap();
//!   {
//!      let mut tables = txn.tables();
//!      tables.insert(&txn, Data(1,"red".to_string())).unwrap();
//!      tables.insert(&txn, Data(2,"red".to_string())).unwrap();
//!      tables.insert(&txn, Data(3,"blue".to_string())).unwrap();
//!   }
//!   txn.commit().unwrap();
//!    
//!   let txn_read = db.read_transaction().unwrap();
//!   let mut tables = txn_read.tables();
//!    
//!   // Retrieve data with p_key=3
//!   let retrieve_data: Data = tables.primary_get(&txn_read, &3_u32.to_be_bytes()).unwrap().unwrap();
//!   println!("data p_key='3' : {:?}", retrieve_data);
//!    
//!   // Iterate data with s_key="red" String
//!   for item in tables.secondary_iter_start_with::<Data>(&txn_read, DataKey::s_key, "red".as_bytes()).unwrap() {
//!      println!("data s_key='1': {:?}", item);
//!   }
//!
//!   // Remove data
//!   let txn = db.transaction().unwrap();
//!   {
//!      let mut tables = txn.tables();
//!      tables.remove(&txn, retrieve_data).unwrap();
//!   }
//!   txn.commit().unwrap();
//!  }
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
pub use struct_db_macro::*;
pub use tables::*;
pub use transaction::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Redb error")]
    Redb(#[from] redb::Error),

    #[error("Redb database error")]
    RedbDatabaseError(#[from] redb::DatabaseError),

    #[error("Redb transaction error")]
    RedbTransactionError(#[from] redb::TransactionError),

    #[error("Redb storage error")]
    RedbStorageError(#[from] redb::StorageError),

    #[error("Redb table error")]
    RedbTableError(#[from] redb::TableError),

    #[error("Redb commit error")]
    RedbCommitError(#[from] redb::CommitError),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Table definition not found {table}")]
    TableDefinitionNotFound { table: String },

    #[error("Key not found {key:?}")]
    KeyNotFound { key: Vec<u8> },

    #[error("Primary key associated with the secondary key not found {primary_key:?}")]
    PrimaryKeyNotFound { primary_key: Vec<u8> },

    #[error("Duplicate key for \"{key_name}\"")]
    DuplicateKey { key_name: &'static str },

    #[error("Watch event error")]
    WatchEventError(#[from] watch::WatchEventError),

    #[error("Max watcher reached (should be impossible)")]
    MaxWatcherReached,
}


pub type Result<T> = std::result::Result<T, Error>;
