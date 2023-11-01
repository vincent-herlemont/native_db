use crate::common::unwrap_item;
use crate::Error::TableDefinitionNotFound;
use crate::PrimaryIterator;
use crate::{
    KeyDefinition, PrimaryIteratorStartWith, Result, SDBItem, SecondaryIterator,
    SecondaryIteratorStartWith,
};
use redb::ReadableTable as RedbReadableTable;
use std::marker::PhantomData;
use std::ops::RangeBounds;

pub trait ReadableTable<'db, 'txn> {
    type Table: redb::ReadableTable<&'static [u8], &'static [u8]>;
    type Transaction<'x>;

    fn open_primary_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        table_name: &'static str,
    ) -> Result<()>;

    fn open_secondary_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        primary_table_name: &'static str,
        secondary_table_name: &'static str,
    ) -> Result<()>;

    fn get_table(&self, table_name: &'static str) -> Option<&Self::Table>;

    /// Get a value from the table.
    /// Returns `Ok(None)` if the key does not exist.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
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
    ///    let mut db = Db::create_tmp("my_db_rt_g").unwrap();
    ///    // Initialize the table
    ///    db.define::<Data>();
    ///    
    ///    // Insert a new data
    ///    let mut txn = db.transaction().unwrap();
    ///    {
    ///       let mut tables = txn.tables();
    ///       tables.insert(&txn, Data(1)).unwrap();
    ///    }
    ///     txn.commit().unwrap(); // /!\ Don't forget to commit
    ///
    ///    // Get a value from the table
    ///    let txn_read = db.read_transaction().unwrap();
    ///    let mut tables = txn_read.tables();
    ///
    ///    // Using explicit type (turbofish syntax)
    ///    let value = tables.primary_get::<Data>(&txn_read, &1u32.to_be_bytes());
    ///    
    ///    // Using type inference
    ///    let value: Option<Data> = tables.primary_get(&txn_read, &1u32.to_be_bytes()).unwrap();
    /// }
    fn primary_get<T: SDBItem>(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        key: &[u8],
    ) -> Result<Option<T>> {
        let table_name = T::struct_db_schema().table_name;
        self.open_primary_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let item = table.get(key)?;
        Ok(unwrap_item(item))
    }

    /// Iterate over all the values of the table.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
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
    ///   use std::arch::asm;
    /// let mut db = Db::create_tmp("my_db_p_iter").unwrap();
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
    ///   // Iterate over all the values of the table
    ///   let txn_read = db.read_transaction().unwrap();
    ///   let mut tables = txn_read.tables();
    ///
    ///   for value in tables.primary_iter::<Data>(&txn_read).unwrap() {
    ///         assert_eq!(value, Data(1));
    ///   }
    /// }
    fn primary_iter<'a, T: SDBItem>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
    ) -> Result<PrimaryIterator<'_, 'txn, 'db, T>>
    where
        'db: 'a,
        'txn: 'a,
    {
        self.primary_iter_range(txn, ..)
    }

    /// Iterate over all the values of the table that are in the given range.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`primary_iter`](ReadableTable::primary_iter) but with a range.
    /// - See tests/09_iterator.rs for more examples.
    fn primary_iter_range<'a, 'b, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        range_value: impl RangeBounds<&'a [u8]> + 'a,
    ) -> Result<PrimaryIterator<'_, 'txn, 'db, T>>
    where
        T: SDBItem,
        'db: 'a,
        'txn: 'a,
    {
        let table_name = T::struct_db_schema().table_name;
        self.open_primary_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let range = table.range::<&'_ [u8]>(range_value)?;
        Ok(PrimaryIterator {
            range,
            _marker: PhantomData,
        })
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`primary_iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See tests/09_iterator.rs for more examples.
    fn primary_iter_start_with<'a, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        prefix_value: &'a [u8],
    ) -> Result<PrimaryIteratorStartWith<'_, 'txn, 'db, T>>
    where
        T: SDBItem,
        'db: 'a,
        'txn: 'a,
    {
        let table_name = T::struct_db_schema().table_name;
        self.open_primary_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let range = table.range::<&'_ [u8]>(prefix_value..)?;
        Ok(PrimaryIteratorStartWith {
            range,
            start_with: prefix_value,
            _marker: PhantomData,
        })
    }

    /// Get a value from the table using a secondary key.
    /// Returns `Ok(None)` if the key does not exist.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// Set the key_definition: use the `<your_type>Key` enum generated by the `struct_db`
    /// macro to specify the key. Like this: `<your_type>Key::<your_secondary_key>`.
    ///
    /// E.g: `tables.get_by_key(&txn_read, <your_type>Key::<your_secondary_key>, &your_key)`
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
    ///   let mut db = Db::create_tmp("my_db_rt_gk").unwrap();
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
    ///
    ///   // Get a value from the table
    ///   let txn_read = db.read_transaction().unwrap();
    ///   let mut tables = txn_read.tables();
    ///   // Using explicit type (turbofish syntax)
    ///   let value = tables.secondary_get::<Data>(&txn_read, DataKey::s_key, &"hello".as_bytes());
    ///   
    ///   // Using type inference
    ///   let value: Option<Data> = tables.secondary_get(&txn_read, DataKey::s_key, &"hello".as_bytes()).unwrap();
    /// }
    fn secondary_get<T: SDBItem>(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        key_def: impl KeyDefinition,
        key: &[u8],
    ) -> Result<Option<T>> {
        let table_name = key_def.secondary_table_name();

        let primary_key: Vec<u8> = {
            self.open_secondary_table(txn, T::struct_db_schema().table_name, table_name)?;
            let table = self.get_table(table_name).unwrap();
            let value = table.get(key)?;
            if let Some(value) = value {
                value.value().into()
            } else {
                return Ok(None);
            }
        };

        Ok(Some(self.primary_get(txn, &primary_key)?.ok_or(
            crate::Error::PrimaryKeyNotFound {
                primary_key: primary_key.to_vec(),
            },
        )?))
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`primary_iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See [`get_by_key`](crate::Tables::secondary_get) too know how to set the key_definition.
    /// - See tests/09_iterator.rs for more examples.
    fn secondary_iter<'a, T: SDBItem>(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        key_def: impl KeyDefinition,
    ) -> Result<SecondaryIterator<'_, 'txn, 'db, T, Self::Table>> {
        self.secondary_iter_range(txn, key_def, ..)
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`primary_iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See [`get_by_key`](crate::Tables::secondary_get) too know how to set the key_definition.
    /// - See tests/09_iterator.rs for more examples.
    fn secondary_iter_range<'a, 'b, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        key_def: impl KeyDefinition,
        range_key: impl RangeBounds<&'b [u8]> + 'b,
    ) -> Result<SecondaryIterator<'_, 'txn, 'db, T, Self::Table>>
    where
        T: SDBItem,
        'a: 'b,
    {
        let main_table_name = T::struct_db_schema().table_name;
        self.open_primary_table(txn, main_table_name)?;
        let secondary_table_name = key_def.secondary_table_name();
        self.open_secondary_table(txn, main_table_name, secondary_table_name)?;

        let main_table = self.get_table(main_table_name).unwrap();
        let secondary_table = self.get_table(secondary_table_name).unwrap();
        let range = secondary_table.range::<&'_ [u8]>(range_key)?;

        Ok(SecondaryIterator {
            range,
            main_table,
            _marker: PhantomData,
        })
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`primary_iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See [`get_by_key`](crate::Tables::secondary_get) too know how to set the key_definition.
    /// - See tests/09_iterator.rs for more examples.
    fn secondary_iter_start_with<'a, 'b, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        key_def: impl KeyDefinition,
        key_prefix: &'b [u8],
    ) -> Result<SecondaryIteratorStartWith<'a, 'txn, 'db, T, Self::Table>>
    where
        T: SDBItem,
        'b: 'a,
    {
        let main_table_name = T::struct_db_schema().table_name;
        self.open_primary_table(txn, main_table_name)?;
        let secondary_table_name = key_def.secondary_table_name();
        self.open_secondary_table(txn, main_table_name, secondary_table_name)?;

        let main_table = self.get_table(main_table_name).unwrap();
        let secondary_table = self.get_table(secondary_table_name).unwrap();
        let range = secondary_table.range::<&'_ [u8]>(key_prefix..)?;

        Ok(SecondaryIteratorStartWith {
            range,
            start_with: key_prefix,
            main_table,
            _marker: PhantomData,
        })
    }

    /// Returns the number of elements in the table.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
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
    ///   use std::arch::asm;
    /// let mut db = Db::create_tmp("my_db_len").unwrap();
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
    ///   // Get the number of elements
    ///   let txn_read = db.read_transaction().unwrap();
    ///   let mut tables = txn_read.tables();
    ///   let len = tables.len::<Data>(&txn_read).unwrap();
    ///   assert_eq!(len, 1);
    /// }
    fn len<T: SDBItem>(&mut self, txn: &'txn Self::Transaction<'db>) -> Result<u64> {
        let table_name = T::struct_db_schema().table_name;
        self.open_primary_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let result = table.len()?;
        Ok(result)
    }
}
