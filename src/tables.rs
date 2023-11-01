use crate::item::{BinaryValue, Item};
use crate::table_definition::PrimaryTableDefinition;
use crate::watch;
use crate::watch::{Event, WatcherRequest};
use crate::Error::TableDefinitionNotFound;
use crate::Result;
use crate::{ReadableTable, SDBItem, Transaction};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::RangeBounds;

/// A collection of read-write tables. Read operation from [`ReadableTable`](crate::ReadableTable)
/// and write operations [`insert`](crate::Tables::insert), [`update`](crate::Tables::update), [`remove`](crate::Tables::remove)
/// and [`migrate`](crate::Tables::migrate) are available.
pub struct Tables<'db, 'txn> {
    pub(crate) table_definitions: &'db HashMap<&'static str, PrimaryTableDefinition>,
    pub(crate) opened_tables:
        HashMap<&'static str, redb::Table<'db, 'txn, &'static [u8], &'static [u8]>>,
    pub(crate) batch: &'txn RefCell<watch::Batch>,
}

impl<'db, 'txn> ReadableTable<'db, 'txn> for Tables<'db, 'txn> {
    type Table = redb::Table<'db, 'txn, &'static [u8], &'static [u8]>;
    type Transaction<'x> = Transaction<'db>;

    fn open_primary_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        primary_table_name: &'static str,
    ) -> Result<()> {
        let table =
            self.table_definitions
                .get(primary_table_name)
                .ok_or(TableDefinitionNotFound {
                    table: primary_table_name.to_string(),
                })?;
        if !self.opened_tables.contains_key(primary_table_name) {
            let table = txn.txn.open_table(table.redb)?;
            self.opened_tables.insert(primary_table_name, table);
        }
        Ok(())
    }

    fn open_secondary_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        primary_table_name: &'static str,
        secondary_table_name: &'static str,
    ) -> Result<()> {
        let primary_table =
            self.table_definitions
                .get(primary_table_name)
                .ok_or(TableDefinitionNotFound {
                    table: primary_table_name.to_string(),
                })?;
        let secondary_table = primary_table
            .secondary_tables
            .get(secondary_table_name)
            .ok_or(TableDefinitionNotFound {
                table: secondary_table_name.to_string(),
            })?;
        if !self.opened_tables.contains_key(secondary_table_name) {
            let table = txn.txn.open_table(secondary_table.rdb())?;
            self.opened_tables.insert(secondary_table_name, table);
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
    /// #[struct_db(pk = p_key,gk = s_key)]
    /// struct Data(u32, String);
    /// impl Data {
    ///    pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()}
    ///    pub fn s_key(&self) -> Vec<u8> {self.1.as_bytes().to_vec()}
    /// }
    ///
    /// fn main() {
    ///   let mut db = Db::create_tmp("my_db_t_insert").unwrap();
    ///   // Initialize the table
    ///   db.define::<Data>();
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
        let (watcher_request, binary_value) =
            self.internal_insert(txn, T::struct_db_schema(), item.to_item())?;
        let event = Event::new_insert(binary_value);
        self.batch.borrow_mut().add(watcher_request, event);
        Ok(())
    }

    // fn internal_insert<T: SDBItem>(
    //     &mut self,
    //     txn: &'txn Transaction<'db>,
    //     item: T,
    // ) -> Result<(WatcherRequest, BinaryValue)> {
    //     let item: Item = item.to_item();
    //     let schema = T::struct_db_schema();
    //     self.internal_insert_2(txn, schema, item)
    // }

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
    /// #[struct_db(pk = p_key)]
    /// struct Data(u32);
    /// impl Data{ pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()} }
    ///
    /// fn main() {
    ///   let mut db = Db::create_tmp("my_db_t_update").unwrap();
    ///   // Initialize the table
    ///   db.define::<Data>();
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
        let (watcher_request, new_binary_value) =
            self.internal_insert(txn, T::struct_db_schema(), updated_item.to_item())?;

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
    /// #[struct_db(pk = p_key)]
    /// struct Data(u32);
    /// impl Data{ pub fn p_key(&self) -> Vec<u8> {self.0.to_be_bytes().to_vec()} }
    ///
    /// fn main() {
    ///   let mut db = Db::create_tmp("my_db_t_remove").unwrap();
    ///   // Initialize the table
    ///   db.define::<Data>();
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

        let primary_key = item.struct_db_pk();
        let keys = item.struct_db_gks();
        let value = item.struct_db_bincode_encode_to_vec();
        {
            self.open_primary_table(txn, table_name)?;
            let table = self.opened_tables.get_mut(table_name).unwrap();
            table.remove(&primary_key.as_slice())?;
        }

        for (secondary_table_name, value) in &keys {
            self.open_secondary_table(txn, table_name, secondary_table_name)?;
            let secondary_table = self.opened_tables.get_mut(secondary_table_name).unwrap();
            secondary_table.remove(&value.as_slice())?;
        }

        Ok((
            WatcherRequest::new(table_name, primary_key, keys),
            BinaryValue(value),
        ))
    }

    /// Migration from a type to another.
    ///
    /// Not send any event.
    ///
    /// # Example
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use struct_db::*;
    ///
    /// type Data = DataV2;
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(pk = p_key)]
    /// struct DataV1(u32);
    ///
    /// impl DataV1 {
    ///     pub fn p_key(&self) -> Vec<u8> {
    ///         self.0.to_be_bytes().to_vec()
    ///     }
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    /// #[struct_db(pk = p_key)]
    /// struct DataV2(String);
    ///
    /// impl DataV2 {
    ///     pub fn p_key(&self) -> Vec<u8> {
    ///         self.0.as_bytes().to_vec()
    ///     }
    /// }
    ///
    /// impl From<DataV1> for DataV2 {
    ///     fn from(av1: DataV1) -> Self {
    ///         Self(av1.0.to_string())
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut db = Db::create_tmp("my_db_t_migration").unwrap();
    ///
    ///   db.define::<DataV1>();
    ///   db.define::<DataV2>();
    ///
    ///   let data = DataV1(42);
    ///
    ///   let txn = db.transaction().unwrap();
    ///   {
    ///     let mut tables = txn.tables();
    ///     tables.insert(&txn, data).unwrap();
    ///   }
    ///   txn.commit().unwrap();
    ///
    ///   // Migrate
    ///   let txn = db.transaction().unwrap();
    ///   {
    ///     let mut tables = txn.tables();
    ///     tables.migrate::<DataV1, DataV2>(&txn).unwrap();
    ///   }
    ///   txn.commit().unwrap();
    ///
    ///   // Check migration
    ///   let txn = db.read_transaction().unwrap();
    ///   let mut tables = txn.tables();
    ///   let data = tables.primary_get::<Data>(&txn, "42".as_bytes()).unwrap().unwrap();
    ///   println!("migrated data: {:?}", data);
    /// }
    pub fn migrate<OldType, NewType>(&mut self, txn: &'txn Transaction<'db>) -> Result<()>
    where
        OldType: SDBItem + Clone,
        NewType: SDBItem + From<OldType>,
    {
        let find_all_old: Vec<OldType> = self.primary_iter(txn).unwrap().collect();
        for old in find_all_old {
            let new: NewType = old.clone().into();
            self.internal_insert(txn, NewType::struct_db_schema(), new.to_item())?;
            self.internal_remove(txn, old)?;
        }

        Ok(())
    }

    // TODO: rename to drain, add add to argument a range
    pub fn primary_drain<'a, T: SDBItem>(
        &mut self,
        txn: &'txn Transaction<'db>,
        range_value: impl RangeBounds<&'a [u8]> + 'a + Copy,
    ) -> Result<Vec<T>> {
        let drained_data =
            self.internal_primary_drain(txn, T::struct_db_schema().table_name, range_value)?;
        let mut items = vec![];
        for binary_value in drained_data {
            let item = T::struct_db_bincode_decode_from_slice(binary_value.0.as_slice());
            items.push(item);
        }
        Ok(items)
    }
}
