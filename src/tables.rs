use crate::item::BinaryValue;
use crate::watch;
use crate::watch::{Event, WatcherRequest};
use crate::Error::TableDefinitionNotFound;
use crate::Result;
use crate::{ReadableTable, SDBItem, Transaction};
use std::cell::RefCell;
use std::collections::HashMap;

/// A collection of read-write tables. Read operation from [`ReadableTable`](crate::ReadableTable)
/// and write operations [`insert`](crate::Tables::insert), [`update`](crate::Tables::update), [`remove`](crate::Tables::remove)
/// and [`migrate`](crate::Tables::migrate) are available.
pub struct Tables<'db, 'txn> {
    pub(crate) table_definitions:
        &'db HashMap<&'static str, redb::TableDefinition<'static, &'static [u8], &'static [u8]>>,
    pub(crate) opened_tables:
        HashMap<&'static str, redb::Table<'db, 'txn, &'static [u8], &'static [u8]>>,
    pub(crate) batch: &'txn RefCell<watch::Batch>,
}

impl<'db, 'txn> ReadableTable<'db, 'txn> for Tables<'db, 'txn> {
    type Table = redb::Table<'db, 'txn, &'static [u8], &'static [u8]>;
    type Transaction<'x> = Transaction<'db>;

    fn open_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        table_name: &'static str,
    ) -> Result<()> {
        let table = *self
            .table_definitions
            .get(table_name)
            .ok_or(TableDefinitionNotFound {
                table: table_name.to_string(),
            })?;
        if !self.opened_tables.contains_key(table_name) {
            let table = txn.txn.open_table(table)?;
            self.opened_tables.insert(table_name, table);
        }
        Ok(())
    }

    fn get_table(&self, table_name: &'static str) -> Option<&Self::Table> {
        self.opened_tables.get(table_name)
    }
}

impl<'db, 'txn> Tables<'db, 'txn> {
    /// Insert data into the database.
    ///
    /// Send a [`event::Insert`](watch::Insert) event that you can
    /// receive using [`watch`](crate::Db::primary_watch) or others `watch_*` functions.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    /// #[struct_db(fn_primary_key(p_key),fn_secondary_key(s_key))]
    /// struct Data(u32, String);
    /// impl Data {
    ///    pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()}
    ///    pub fn s_key(&self) -> Vec<u8> {self.1.as_bytes().to_vec()}
    /// }
    ///
    /// fn main() {
    ///   let mut db = Db::init_tmp("my_db_t_insert").unwrap();
    ///   // Initialize the table
    ///   db.add_schema(Data::struct_db_schema());
    ///   
    ///   // Insert a new data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///      let mut tables = txn.tables();
    ///      tables.insert(&txn, Data(1, "hello".to_string())).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    /// }
    pub fn insert<T: SDBItem>(&mut self, txn: &'txn Transaction<'db>, item: T) -> Result<()> {
        let (watcher_request, binary_value) = self.internal_insert(txn, item)?;
        let event = Event::new_insert(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    fn internal_insert<T: SDBItem>(
        &mut self,
        txn: &'txn Transaction<'db>,
        item: T,
    ) -> Result<(WatcherRequest, BinaryValue)> {
        let schema = T::struct_db_schema();
        let table_name = schema.table_name;

        let primary_key_value = item.struct_db_primary_key();
        let secondary_keys = item.struct_db_keys();
        let value = item.struct_db_bincode_encode_to_vec();
        let already_exists;
        {
            self.open_table(txn, table_name)?;
            let table = self.opened_tables.get_mut(table_name).unwrap();
            already_exists = table
                .insert(&primary_key_value.as_slice(), &value.as_slice())?
                .is_some();
        }

        for (secondary_table_name, key_value) in &secondary_keys {
            self.open_table(txn, secondary_table_name)?;
            let secondary_table = self.opened_tables.get_mut(secondary_table_name).unwrap();
            let result =
                secondary_table.insert(&key_value.as_slice(), &primary_key_value.as_slice())?;
            if result.is_some() && !already_exists {
                return Err(crate::Error::DuplicateKey {
                    key_name: secondary_table_name,
                }
                .into());
            }
        }

        Ok((
            WatcherRequest::new(table_name, primary_key_value, secondary_keys),
            BinaryValue(value),
        ))
    }

    /// Update data in the database.
    ///
    /// Send a [`event::Update`](watch::Update) event that you can
    /// receive using [`watch`](crate::Db::primary_watch) or others `watch_*` functions.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Data(u32);
    /// impl Data{ pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()} }
    ///
    /// fn main() {
    ///   let mut db = Db::init_tmp("my_db_t_update").unwrap();
    ///   // Initialize the table
    ///   db.add_schema(Data::struct_db_schema());
    ///   
    ///   // Insert a new data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///     let mut tables = txn.tables();
    ///     tables.insert(&txn, Data(1)).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///   // Update the data, e.g: increment the value
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///       let mut tables = txn.tables();
    ///       let old_data = tables.primary_get::<Data>(&txn, &1u32.to_be_bytes()).unwrap().unwrap();
    ///       let new_data = Data(old_data.0 + 1);
    ///       tables.update(&txn, old_data, new_data).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///   // Get the updated data
    ///   let mut txn = db.read_transaction().unwrap();
    ///   let mut tables = txn.tables();
    ///   let data:Data = tables.primary_get(&txn, &2u32.to_be_bytes()).unwrap().unwrap();
    ///   assert_eq!(data, Data(2));
    /// }
    pub fn update<T: SDBItem>(
        &mut self,
        txn: &'txn Transaction<'db>,
        old_item: T,
        updated_item: T,
    ) -> Result<()> {
        let (_, old_binary_value) = self.internal_remove(txn, old_item)?;
        let (watcher_request, new_binary_value) = self.internal_insert(txn, updated_item)?;

        let event = Event::new_update(old_binary_value, new_binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    /// Remove data from the database.
    ///
    /// Send a [`event::Delete`](watch::Delete) event that you can
    /// receive using [`watch`](crate::Db::primary_watch) or others `watch_*` functions.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Data(u32);
    /// impl Data{ pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()} }
    ///
    /// fn main() {
    ///   let mut db = Db::init_tmp("my_db_t_remove").unwrap();
    ///   // Initialize the table
    ///   db.add_schema(Data::struct_db_schema());
    ///   
    ///   // Insert a new data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///      let mut tables = txn.tables();
    ///      tables.insert(&txn, Data(1)).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///   // Remove the data
    ///   let mut txn = db.transaction().unwrap();
    ///   {
    ///      let mut tables = txn.tables();
    ///      tables.remove(&txn, Data(1)).unwrap();
    ///   }
    ///   txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///   // Get the removed data
    ///   let mut txn = db.read_transaction().unwrap();
    ///   let mut tables = txn.tables();
    ///   let data:Option<Data> = tables.primary_get(&txn, &1u32.to_be_bytes()).unwrap();
    ///   assert_eq!(data, None);
    /// }
    pub fn remove<T: SDBItem>(&mut self, txn: &'txn Transaction<'db>, item: T) -> Result<()> {
        let (watcher_request, binary_value) = self.internal_remove(txn, item)?;
        let event = Event::new_delete(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    fn internal_remove<T: SDBItem>(
        &mut self,
        txn: &'txn Transaction<'db>,
        item: T,
    ) -> Result<(WatcherRequest, BinaryValue)> {
        let schema = T::struct_db_schema();
        let table_name = schema.table_name;

        let primary_key_value = item.struct_db_primary_key();
        let keys = item.struct_db_keys();
        let value = item.struct_db_bincode_encode_to_vec();
        {
            self.open_table(txn, table_name)?;
            let table = self.opened_tables.get_mut(table_name).unwrap();
            table.remove(&primary_key_value.as_slice())?;
        }

        for (secondary_table_name, value) in &keys {
            self.open_table(txn, secondary_table_name)?;
            let secondary_table = self.opened_tables.get_mut(secondary_table_name).unwrap();
            secondary_table.remove(&value.as_slice())?;
        }

        Ok((
            WatcherRequest::new(table_name, primary_key_value, keys),
            BinaryValue(value),
        ))
    }

    /// Migration from a table to another.
    ///
    /// Not send any event.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Av1(u32);
    ///
    /// impl Av1 {
    ///     pub fn p_key(&self) -> Vec<u8> {
    ///         self.0.to_be_bytes().to_vec()
    ///     }
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(fn_primary_key(p_key))]
    /// struct Av2(String);
    ///
    /// impl Av2 {
    ///     pub fn p_key(&self) -> Vec<u8> {
    ///         self.0.as_bytes().to_vec()
    ///     }
    /// }
    ///
    /// impl From<Av1> for Av2 {
    ///     fn from(av1: Av1) -> Self {
    ///         Self(av1.0.to_string())
    ///     }
    /// }
    ///
    /// fn main() {
    ///    let mut db = Db::init_tmp("my_db_t_migration").unwrap();
    ///    
    ///    db.add_schema(Av1::struct_db_schema());
    ///    db.add_schema(Av2::struct_db_schema());
    ///    
    ///    let a = Av1(42);
    ///    
    ///    let txn = db.transaction().unwrap();
    ///    {
    ///    let mut tables = txn.tables();
    ///    tables.insert(&txn, a.clone()).unwrap();
    ///    }
    ///    txn.commit().unwrap();
    ///    
    ///    let (recv_av1, _id) = db.primary_watch::<Av1>(None).unwrap();
    ///    let (recv_av2, _id) = db.primary_watch::<Av2>(None).unwrap();
    ///    
    ///    // Migrate
    ///    let txn = db.transaction().unwrap();
    ///    {
    ///       let mut tables = txn.tables();
    ///       tables.migrate::<Av1, Av2>(&txn).unwrap();
    ///    }
    ///    txn.commit().unwrap();
    ///    
    ///    // Check migration
    ///    let txn = db.read_transaction().unwrap();
    ///    {
    ///        let mut tables = txn.tables();
    ///        let len_av1 = tables.len::<Av1>(&txn).unwrap();
    ///        assert_eq!(len_av1, 0);
    ///        let len_av2 = tables.len::<Av2>(&txn).unwrap();
    ///        assert_eq!(len_av2, 1);
    ///    
    ///        let a2 = tables.primary_get::<Av2>(&txn, "42".as_bytes()).unwrap().unwrap();
    ///        assert_eq!(a2, Av2("42".to_string()));
    ///    }
    /// }
    pub fn migrate<TO, TN>(&mut self, txn: &'txn Transaction<'db>) -> Result<()>
    where
        TO: SDBItem + Clone,
        TN: SDBItem + From<TO>,
    {
        let find_all_old: Vec<TO> = self.primary_iter(txn).unwrap().collect();
        for old in find_all_old {
            let new: TN = old.clone().into();
            self.internal_insert(txn, new)?;
            self.internal_remove(txn, old)?;
        }

        Ok(())
    }
}
