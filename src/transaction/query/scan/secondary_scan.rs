use crate::db_type::{unwrap_item, DatabaseInnerKeyValue, DatabaseInnerKeyValueRange, Input};
use crate::InnerKeyValue;
use redb;
use std::marker::PhantomData;
use std::ops::RangeBounds;

/// Scan values from the database by secondary key.
pub struct SecondaryScan<PrimaryTable, SecondaryTable, T: Input>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
    SecondaryTable: redb::ReadableTable<DatabaseInnerKeyValue, DatabaseInnerKeyValue>,
{
    pub(crate) primary_table: PrimaryTable,
    pub(crate) secondary_table: SecondaryTable,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, SecondaryTable, T: Input> SecondaryScan<PrimaryTable, SecondaryTable, T>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
    SecondaryTable: redb::ReadableTable<DatabaseInnerKeyValue, DatabaseInnerKeyValue>,
{
    pub(crate) const fn new(primary_table: PrimaryTable, secondary_table: SecondaryTable) -> Self {
        Self {
            primary_table,
            secondary_table,
            _marker: PhantomData,
        }
    }

    /// Iterate over all values by secondary key.
    ///
    /// If the secondary key is [`optional`](struct.DatabaseBuilder.html#optional) you will
    /// get all values that have the secondary key set.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    ///     #[secondary_key(optional)]
    ///     name: Option<String>,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get only values that have the secondary key set (name is not None)
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.all().collect();
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self) -> SecondaryScanIterator<'_, PrimaryTable, T> {
        let range = self
            .secondary_table
            .range::<DatabaseInnerKeyValue>(..)
            .unwrap();
        SecondaryScanIterator {
            primary_table: &self.primary_table,
            range,
            _marker: PhantomData,
        }
    }

    /// Iterate over all values by secondary key.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    ///     #[secondary_key]
    ///     name: String,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get only values that have the secondary key name from C to the end
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.range("C"..).collect();
    ///     Ok(())
    /// }
    /// ```
    pub fn range<TR: InnerKeyValue, R: RangeBounds<TR>>(
        &self,
        range: R,
    ) -> SecondaryScanIterator<'_, PrimaryTable, T> {
        let database_inner_key_value_range = DatabaseInnerKeyValueRange::new(range);
        let range = self
            .secondary_table
            .range::<DatabaseInnerKeyValue>(database_inner_key_value_range)
            .unwrap();
        SecondaryScanIterator {
            primary_table: &self.primary_table,
            range,
            _marker: PhantomData,
        }
    }

    /// Iterate over all values by secondary key.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// #[native_model(id=1, version=1)]
    /// #[native_db]
    /// struct Data {
    ///     #[primary_key]
    ///     id: u64,
    ///     #[secondary_key]
    ///     name: String,
    /// }
    ///
    /// fn main() -> Result<(), db_type::Error> {
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get only values that have the secondary key name starting with "hello"
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.start_with("hello").collect();
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with<'a>(
        &'a self,
        start_with: impl InnerKeyValue + 'a,
    ) -> SecondaryScanIteratorStartWith<'a, PrimaryTable, T> {
        let start_with = start_with.database_inner_key_value();
        let range = self
            .secondary_table
            .range::<DatabaseInnerKeyValue>(start_with.clone()..)
            .unwrap();
        SecondaryScanIteratorStartWith {
            primary_table: &self.primary_table,
            start_with,
            range,
            _marker: PhantomData,
        }
    }
}

pub struct SecondaryScanIterator<'a, PrimaryTable, T: Input>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
{
    pub(crate) primary_table: &'a PrimaryTable,
    pub(crate) range: redb::Range<'a, DatabaseInnerKeyValue, DatabaseInnerKeyValue>,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, PrimaryTable, T: Input> Iterator for SecondaryScanIterator<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((_, key))) => self
                .primary_table
                .get(key.value())
                .map_or_else(|_| None, |value| unwrap_item(value)),
            _ => None,
        }
    }
}

impl<'a, PrimaryTable, T: Input> DoubleEndedIterator for SecondaryScanIterator<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(Ok((_, key))) => unwrap_item(self.primary_table.get(key.value()).unwrap()),
            _ => None,
        }
    }
}

pub struct SecondaryScanIteratorStartWith<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
    T: Input,
{
    pub(crate) primary_table: &'a PrimaryTable,
    pub(crate) start_with: DatabaseInnerKeyValue,
    pub(crate) range: redb::Range<'a, DatabaseInnerKeyValue, DatabaseInnerKeyValue>,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, PrimaryTable, T> Iterator for SecondaryScanIteratorStartWith<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
    T: Input,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((secondary_key, primary_key))) => {
                if secondary_key
                    .value()
                    .as_slice()
                    .starts_with(self.start_with.as_slice())
                {
                    unwrap_item(self.primary_table.get(primary_key.value()).unwrap())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
