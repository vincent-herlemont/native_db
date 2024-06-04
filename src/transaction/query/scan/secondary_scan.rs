use crate::db_type::ToKey;
use crate::db_type::{unwrap_item, Input, Key, KeyRange, Result};
use redb;
use std::marker::PhantomData;
use std::ops::RangeBounds;

/// Scan values from the database by secondary key.
pub struct SecondaryScan<PrimaryTable, SecondaryTable, T: Input>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
    SecondaryTable: redb::ReadableTable<Key, Key>,
{
    pub(crate) primary_table: PrimaryTable,
    pub(crate) secondary_table: SecondaryTable,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, SecondaryTable, T: Input> SecondaryScan<PrimaryTable, SecondaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
    SecondaryTable: redb::ReadableTable<Key, Key>,
{
    pub(crate) fn new(primary_table: PrimaryTable, secondary_table: SecondaryTable) -> Self {
        Self {
            primary_table,
            secondary_table,
            _marker: PhantomData::default(),
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
    /// use itertools::Itertools;
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
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.all().try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self) -> SecondaryScanIterator<PrimaryTable, T> {
        let range = self.secondary_table.range::<Key>(..).unwrap();
        SecondaryScanIterator {
            primary_table: &self.primary_table,
            range,
            _marker: PhantomData::default(),
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
    /// use itertools::Itertools;
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
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.range("C"..).try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn range<TR: ToKey, R: RangeBounds<TR>>(
        &self,
        range: R,
    ) -> SecondaryScanIterator<PrimaryTable, T> {
        let database_inner_key_value_range = KeyRange::new(range);
        let range = self
            .secondary_table
            .range::<Key>(database_inner_key_value_range)
            .unwrap();
        SecondaryScanIterator {
            primary_table: &self.primary_table,
            range,
            _marker: PhantomData::default(),
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
    /// use itertools::Itertools;
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
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.start_with("hello").try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with<'a>(
        &'a self,
        start_with: impl ToKey + 'a,
    ) -> SecondaryScanIteratorStartWith<'a, PrimaryTable, T> {
        let start_with = start_with.to_key();
        let range = self
            .secondary_table
            .range::<Key>(start_with.clone()..)
            .unwrap();
        SecondaryScanIteratorStartWith {
            primary_table: &self.primary_table,
            start_with,
            range,
            _marker: PhantomData::default(),
        }
    }
}

pub struct SecondaryScanIterator<'a, PrimaryTable, T: Input>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    pub(crate) primary_table: &'a PrimaryTable,
    pub(crate) range: redb::Range<'a, Key, Key>,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, PrimaryTable, T: Input> Iterator for SecondaryScanIterator<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((_, key))) => {
                if let Ok(value) = self.primary_table.get(key.value()) {
                    unwrap_item(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a, PrimaryTable, T: Input> DoubleEndedIterator for SecondaryScanIterator<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
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
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
    T: Input,
{
    pub(crate) primary_table: &'a PrimaryTable,
    pub(crate) start_with: Key,
    pub(crate) range: redb::Range<'a, Key, Key>,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, PrimaryTable, T> Iterator for SecondaryScanIteratorStartWith<'a, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
    T: Input,
{
    type Item = Result<T>;

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
