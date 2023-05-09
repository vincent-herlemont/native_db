use crate::common::get;
use crate::Iterator;
use crate::{
    IteratorByKey, IteratorStartWith, IteratorStartWithByKey, KeyDefinition, Result, SDBItem,
};
use redb::ReadableTable as RedbReadableTable;
use std::marker::PhantomData;
use std::ops::RangeBounds;

pub trait ReadableTable<'db, 'txn> {
    type Table: redb::ReadableTable<&'static [u8], &'static [u8]>;
    type Transaction<'x>;

    fn open_table(
        &mut self,
        txn: &'txn Self::Transaction<'db>,
        table_name: &'static str,
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
    ///    let mut db = Db::init_tmp("my_db_rt_g").unwrap();
    ///    // Initialize the table
    ///    db.add_schema(Data::struct_db_schema());
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
        self.open_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let value = table.get(key)?;
        Ok(get(value))
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
    /// let mut db = Db::init_tmp("my_db_rt_iter").unwrap();
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
    ) -> Result<Iterator<'_, 'txn, 'db, T>>
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
    /// - Similar to [`iter`](ReadableTable::primary_iter) but with a range.
    /// - See tests/09_iterator.rs for more examples.
    fn primary_iter_range<'a, 'b, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        range: impl RangeBounds<&'a [u8]> + 'a,
    ) -> Result<Iterator<'_, 'txn, 'db, T>>
    where
        T: SDBItem,
        'db: 'a,
        'txn: 'a,
    {
        let table_name = T::struct_db_schema().table_name;
        self.open_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let range = table.range::<&'_ [u8]>(range)?;
        Ok(Iterator {
            range,
            _marker: PhantomData,
        })
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See tests/09_iterator.rs for more examples.
    fn primary_iter_start_with<'a, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        prefix: &'a [u8],
    ) -> Result<IteratorStartWith<'_, 'txn, 'db, T>>
    where
        T: SDBItem,
        'db: 'a,
        'txn: 'a,
    {
        let table_name = T::struct_db_schema().table_name;
        self.open_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let range = table.range::<&'_ [u8]>(prefix..)?;
        Ok(IteratorStartWith {
            range,
            start_with: prefix,
            _marker: PhantomData,
        })
    }

    /// Get a value from the table using a secondary key.
    /// Returns `Ok(None)` if the key does not exist.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// Set the key_definition: use the `<YourTypeName>Key` enum generated by the `struct_db`
    /// macro to specify the key. Like this: `YourTypeNameKey::your_key_name`.
    ///
    /// E.g: `tables.get_by_key(&txn_read, <YourTypeName>Key::your_key_name, &your_key_value)`
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
    ///   let mut db = Db::init_tmp("my_db_rt_gk").unwrap();
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
        key_definition: impl KeyDefinition,
        key: &[u8],
    ) -> Result<Option<T>> {
        let table_name = key_definition.secondary_table_name();

        let key: Vec<u8> = {
            self.open_table(txn, table_name)?;
            let table = self.get_table(table_name).unwrap();
            let value = table.get(key)?;
            if let Some(value) = value {
                value.value().into()
            } else {
                return Ok(None);
            }
        };

        Ok(Some(self.primary_get(txn, &key)?.ok_or(
            crate::Error::PrimaryKeyNotFound {
                secondary_key: key.to_vec(),
            },
        )?))
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See [`get_by_key`](crate::Tables::secondary_get) too know how to set the key_definition.
    /// - See tests/09_iterator.rs for more examples.
    fn secondary_iter<'a, T: SDBItem>(
        &mut self,
        key_definition: impl KeyDefinition,
        txn: &'txn Self::Transaction<'db>,
    ) -> Result<IteratorByKey<'_, 'txn, 'db, T, Self::Table>> {
        self.secondary_iter_range(txn, key_definition, ..)
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See [`get_by_key`](crate::Tables::secondary_get) too know how to set the key_definition.
    /// - See tests/09_iterator.rs for more examples.
    fn secondary_iter_range<'a, 'b, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        key_definition: impl KeyDefinition,
        range: impl RangeBounds<&'b [u8]> + 'b,
    ) -> Result<IteratorByKey<'_, 'txn, 'db, T, Self::Table>>
    where
        T: SDBItem,
        'a: 'b,
    {
        let main_table_name = T::struct_db_schema().table_name;
        self.open_table(txn, main_table_name)?;
        let secondary_table_name = key_definition.secondary_table_name();
        self.open_table(txn, secondary_table_name)?;

        let main_table = self.get_table(main_table_name).unwrap();
        let secondary_table = self.get_table(secondary_table_name).unwrap();
        let range = secondary_table.range::<&'_ [u8]>(range)?;

        Ok(IteratorByKey {
            range,
            main_table,
            _marker: PhantomData,
        })
    }

    /// Iterate over all the values of the table that start with the given prefix.
    /// Available in [`Tables`](crate::Tables) and [`ReadOnlyTables`](crate::ReadOnlyTables).
    ///
    /// # Example
    /// - Similar to [`iter`](ReadableTable::primary_iter) but with a prefix.
    /// - See [`get_by_key`](crate::Tables::secondary_get) too know how to set the key_definition.
    /// - See tests/09_iterator.rs for more examples.
    fn secondary_iter_start_with<'a, 'b, T>(
        &'a mut self,
        txn: &'txn Self::Transaction<'db>,
        key_definition: impl KeyDefinition,
        prefix: &'b [u8],
    ) -> Result<IteratorStartWithByKey<'a, 'txn, 'db, T, Self::Table>>
    where
        T: SDBItem,
        'b: 'a,
    {
        let main_table_name = T::struct_db_schema().table_name;
        self.open_table(txn, main_table_name)?;
        let secondary_table_name = key_definition.secondary_table_name();
        self.open_table(txn, secondary_table_name)?;

        let main_table = self.get_table(main_table_name).unwrap();
        let secondary_table = self.get_table(secondary_table_name).unwrap();
        let range = secondary_table.range::<&'_ [u8]>(prefix..)?;

        Ok(IteratorStartWithByKey {
            range,
            start_with: prefix,
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
    /// let mut db = Db::init_tmp("my_db_rt_iter").unwrap();
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
    ///   // Get the number of elements
    ///   let txn_read = db.read_transaction().unwrap();
    ///   let mut tables = txn_read.tables();
    ///   let len = tables.len::<Data>(&txn_read).unwrap();
    ///   assert_eq!(len, 1);
    /// }
    fn len<T: SDBItem>(&mut self, txn: &'txn Self::Transaction<'db>) -> Result<usize> {
        let table_name = T::struct_db_schema().table_name;
        self.open_table(txn, table_name)?;
        let table = self.get_table(table_name).unwrap();
        let result = table.len()?;
        Ok(result)
    }
}
