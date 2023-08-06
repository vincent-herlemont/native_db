use crate::watch;
use crate::{Error, KeyDefinition, ReadOnlyTransaction, Result, SDBItem, Transaction};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, RwLock};
use std::u64;
use crate::watch::MpscReceiver;
use crate::builder::Builder;

/// The `Db` struct represents a database instance. It allows add **schema**, create **transactions** and **watcher**.
pub struct Db {
    pub(crate) instance: redb::Database,
    pub(crate) table_definitions:
        HashMap<&'static str, redb::TableDefinition<'static, &'static [u8], &'static [u8]>>,
    pub(crate) watchers: Arc<RwLock<watch::Watchers>>,
    pub(crate) watchers_counter_id: AtomicU64,
}

impl Db {
    /// Creates a new [Db] instance using the given path.
    ///
    /// Use [redb::Builder.create(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.create)
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        Builder::new().create(path)
    }

    /// Creates a new [Db] instance using a temporary directory with the given path.
    ///
    /// Example: `Db::create_tmp('project/my_db')` will create the db to `/tmp/project/my_db`.
    ///
    /// Use [redb::Builder.create(...)](https://docs.rs/redb/latest/redb/struct.Builder.html#method.create)
    pub fn create_tmp(path: impl AsRef<Path>) -> Result<Self> {
        Builder::new().create_tmp(path)
    }

    /// Opens an existing [Db] instance using the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Builder::new().open(path)
    }

    /// Opens an existing [Db] instance using a temporary directory with the given path.
    pub fn open_tmp(path: impl AsRef<Path>) -> Result<Self> {
        Builder::new().open_tmp(path)
    }

    /// Defines a table using the given schema.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Data(u32);
    /// impl Data {pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()}}
    ///
    /// fn main() {
    ///    let mut db = Db::create_tmp("my_db_as").unwrap();
    ///    // Initialize the table
    ///    db.define::<Data>();
    /// }
    pub fn define<T: SDBItem>(&mut self) {
        let schema = T::struct_db_schema();
        let main_table_name = schema.table_name;
        let main_table_definition = redb::TableDefinition::new(main_table_name);
        self.table_definitions
            .insert(main_table_name, main_table_definition);
        for secondary_table_name in schema.secondary_tables_name {
            let secondary_table_definition = redb::TableDefinition::new(secondary_table_name);
            self.table_definitions
                .insert(secondary_table_name, secondary_table_definition);
        }
    }
}

impl Db {
    /// Creates a new read-write transaction.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Data(u32);
    /// impl Data {pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()}}
    ///
    /// fn main() {
    ///   let mut db = Db::create_tmp("my_db_t").unwrap();
    ///   db.define::<Data>();
    ///
    ///   // Use transaction to insert a new data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///     let mut data = Data(42);
    ///     let mut tables = txn.tables();
    ///     tables.insert(&txn, data).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    ///   
    ///   // Use transaction to update a data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///       let mut tables = txn.tables();
    ///       let (new_data, old_data) = {
    ///           let old_data = tables.primary_get::<Data>(&txn, &42_u32.to_be_bytes()).unwrap().unwrap();
    ///           let mut new_data = old_data.clone();
    ///           new_data.0 = 43;
    ///           (new_data, old_data)
    ///       };       
    ///       tables.update(&txn, old_data, new_data).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///   // Use transaction to delete a data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///      let mut tables = txn.tables();
    ///      let data = tables.primary_get::<Data>(&txn, &43_u32.to_be_bytes()).unwrap().unwrap();
    ///      tables.remove(&txn, data).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    /// }
    pub fn transaction(&self) -> Result<Transaction> {
        let txn = self.instance.begin_write()?;
        let write_txn = Transaction {
            table_definitions: &self.table_definitions,
            txn,
            watcher: &self.watchers,
            batch: RefCell::new(watch::Batch::new()),
        };
        Ok(write_txn)
    }

    /// Creates a new read-only transaction.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Data(u32);
    /// impl Data {pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()}}
    ///
    /// fn main() {
    ///  let mut db = Db::create_tmp("my_db_rt").unwrap();
    ///  db.define::<Data>();
    ///  
    ///  // Insert a new data
    ///  let mut txn = db.transaction().unwrap();
    ///  {
    ///    let mut tables = txn.tables();
    ///    tables.insert(&txn, Data(42)).unwrap();
    ///  }
    ///  txn.commit().unwrap(); // /!\ Don't forget to commit
    ///  
    ///  let txn_read = db.read_transaction().unwrap();
    ///  let mut tables = txn_read.tables();
    ///  let len = tables.len::<Data>(&txn_read).unwrap();
    ///  assert_eq!(len, 1);
    /// }
    pub fn read_transaction(&self) -> Result<ReadOnlyTransaction> {
        let txn = self.instance.begin_read()?;
        let read_txn = ReadOnlyTransaction {
            table_definitions: &self.table_definitions,
            txn,
        };
        Ok(read_txn)
    }
}

impl Db {
    fn generate_watcher_id(&self) -> Result<u64> {
        let value = self
            .watchers_counter_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if value == u64::MAX {
            Err(Error::MaxWatcherReached.into())
        } else {
            Ok(value)
        }
    }

    fn watch_generic(
        &self,
        table_filter: watch::TableFilter,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        #[cfg(not(feature = "tokio"))]
        let (event_sender, event_receiver) = std::sync::mpsc::channel();
        #[cfg(feature = "tokio")]
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
        let event_sender = Arc::new(Mutex::new(event_sender));
        let id = self.generate_watcher_id()?;
        let mut watchers = self.watchers.write().unwrap();
        watchers.add_sender(id, &table_filter, Arc::clone(&event_sender));
        drop(watchers);
        Ok((event_receiver, id))
    }

    /// Watches for changes in the specified table for the given primary key.
    /// If the argument `key` is `None` you will receive all the events from the table.
    /// Otherwise you will receive only the events for the given primary key.
    ///
    /// Supported channels to to receive changes:
    ///   - [`std::sync::mpsc::Receiver`](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html) by default
    ///   - [`tokio::sync::mpsc::UnboundedReceiver`](https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html) with the feature (`tokio`).
    ///
    /// To unregister the watcher you need to call [`unwatch`](Db::unwatch)
    /// with the returned `id`.
    ///
    /// Get data from the event, use `inner()` method on:
    /// - [`watch::Insert::inner`](watch::Insert::inner)
    /// - [`watch::Update::inner_new`](watch::Update::inner_new) to get the updated data
    /// - [`watch::Update::inner_old`](watch::Update::inner_old) to get the old data
    /// - [`watch::Delete::inner`](watch::Delete::inner)
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Data(u32);
    /// impl Data {pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()}}
    ///
    /// fn main() {
    ///  let mut db = Db::create_tmp("my_db").unwrap();
    ///  db.define::<Data>();
    ///
    ///  // None you will receive all the events from Data.
    ///  let (event_receiver, _id) = db.primary_watch::<Data>(None).unwrap();
    ///
    ///  // Add a new data
    ///  let mut txn = db.transaction().unwrap();
    ///  {
    ///    let mut tables = txn.tables();
    ///    tables.insert(&txn, Data(42)).unwrap();
    ///  }
    ///  txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///  // Wait for the event
    ///  for _ in 0..1 {
    ///   // With the feature "tokio" you can use async/await pattern
    ///   let event = event_receiver.recv().unwrap();
    ///   if let watch::Event::Insert(insert) = event {
    ///      let data = insert.inner::<Data>();
    ///      assert_eq!(data, Data(42));
    ///    }
    ///  }
    /// }
    pub fn primary_watch<T: SDBItem>(
        &self,
        key: Option<&[u8]>,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::struct_db_schema().table_name;
        let table_filter = watch::TableFilter::new_primary(table_name.as_bytes(), key);
        self.watch_generic(table_filter)
    }

    /// Watches for changes in the specified table for the given prefix.
    /// You will receive all the events for the given prefix.
    ///
    /// To unregister the watcher you need to call [`unwatch`](Db::unwatch)
    /// with the returned `id`.
    ///
    /// # Example
    /// - Similar to [`primary_watch`](#method.primary_watch) but with a prefix.
    pub fn primary_watch_start_with<T: SDBItem>(
        &self,
        key_prefix: &[u8],
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::struct_db_schema().table_name;
        let table_filter =
            watch::TableFilter::new_primary_start_with(table_name.as_bytes(), key_prefix);
        self.watch_generic(table_filter)
    }

    /// Watches for changes in the specified table for the given secondary key.
    /// If the argument `key` is `None` you will receive all the events from the table.
    /// Otherwise you will receive only the events for the given secondary key.
    ///
    /// To unregister the watcher you need to call [`unwatch`](Db::unwatch)
    /// with the returned `id`.
    ///
    /// # Example
    /// - Similar to [`primary_watch`](#method.primary_watch) but with a secondary key.
    pub fn secondary_watch<T: SDBItem>(
        &self,
        key_def: impl KeyDefinition,
        key: Option<&[u8]>,
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::struct_db_schema().table_name;
        let table_filter = watch::TableFilter::new_secondary(table_name.as_bytes(), key_def, key);
        self.watch_generic(table_filter)
    }

    /// Watches for changes in the specified table for the given prefix.
    /// You will receive all the events for the given prefix.
    ///
    /// To unregister the watcher you need to call [`unwatch`](Db::unwatch)
    /// with the returned `id`.
    ///
    /// # Example
    /// - Similar to [`primary_watch`](#method.primary_watch) but with a secondary key and a prefix.
    pub fn secondary_watch_start_with<T: SDBItem>(
        &self,
        key_def: impl KeyDefinition,
        key_prefix: &[u8],
    ) -> Result<(MpscReceiver<watch::Event>, u64)> {
        let table_name = T::struct_db_schema().table_name;
        let table_filter = watch::TableFilter::new_secondary_start_with(
            table_name.as_bytes(),
            key_def,
            key_prefix,
        );
        self.watch_generic(table_filter)
    }

    /// Unwatch the given `id`.
    /// You can get the `id` from the return value of [`primary_watch`](#method.primary_watch).
    /// If the `id` is not valid anymore, this function will do nothing.
    /// If the `id` is valid, the corresponding watcher will be removed.
    pub fn unwatch(&self, id: u64) -> Result<()> {
        let mut watchers = self.watchers.write().unwrap();
        watchers.remove_sender(id);
        Ok(())
    }
}
