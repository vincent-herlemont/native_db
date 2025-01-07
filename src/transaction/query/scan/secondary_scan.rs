use crate::db_type::{
    check_key_type_from_key_definition, check_range_key_range_bounds_from_key_definition,
    KeyDefinition, KeyOptions, ToKey, ToKeyDefinition,
};
use crate::db_type::{unwrap_item, Key, KeyRange, Result, ToInput};
use redb::{self};
use std::marker::PhantomData;
use std::ops::RangeBounds;

/// Scan values from the database by secondary key.
pub struct SecondaryScan<PrimaryTable, SecondaryTable, T: ToInput>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
    SecondaryTable: redb::ReadableMultimapTable<Key, Key>,
{
    pub(crate) primary_table: PrimaryTable,
    pub(crate) secondary_table: SecondaryTable,
    pub(crate) key_def: KeyDefinition<KeyOptions>,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, SecondaryTable, T: ToInput> SecondaryScan<PrimaryTable, SecondaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
    SecondaryTable: redb::ReadableMultimapTable<Key, Key>,
{
    pub(crate) fn new(
        primary_table: PrimaryTable,
        secondary_table: SecondaryTable,
        key_def: impl ToKeyDefinition<KeyOptions>,
    ) -> Self {
        Self {
            primary_table,
            secondary_table,
            key_def: key_def.key_definition(),
            _marker: PhantomData,
        }
    }

    /// Iterate over all values by secondary key.
    ///
    /// If the secondary key is [`optional`](struct.Models.html#optional) you will
    /// get all values that have the secondary key set.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///     let mut models = Models::new();
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get only values that have the secondary key set (name is not None)
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.all()?.try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self) -> Result<SecondaryScanIterator<PrimaryTable, T>> {
        let mut primary_keys = vec![];
        for keys in self.secondary_table.iter()? {
            let (_, l_primary_keys) = keys?;
            for primary_key in l_primary_keys {
                let primary_key = primary_key?;
                primary_keys.push(primary_key);
            }
        }

        Ok(SecondaryScanIterator {
            primary_table: &self.primary_table,
            primary_keys: primary_keys.into_iter(),
            _marker: PhantomData,
        })
    }

    /// Iterate over all values by secondary key in a range.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///     let mut models = Models::new();
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get only values that have the secondary key name from C to the end
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.range("C"..)?.try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn range<R: RangeBounds<impl ToKey>>(
        &self,
        range: R,
    ) -> Result<SecondaryScanIterator<PrimaryTable, T>> {
        check_range_key_range_bounds_from_key_definition(&self.key_def, &range)?;
        let mut primary_keys = vec![];
        let database_inner_key_value_range = KeyRange::new(range);
        for keys in self
            .secondary_table
            .range::<Key>(database_inner_key_value_range)?
        {
            let (_, l_primary_keys) = keys?;
            for primary_key in l_primary_keys {
                let primary_key = primary_key?;
                primary_keys.push(primary_key);
            }
        }

        Ok(SecondaryScanIterator {
            primary_table: &self.primary_table,
            primary_keys: primary_keys.into_iter(),
            _marker: PhantomData,
        })
    }

    /// Iterate over all values by secondary key starting with a prefix.
    ///
    /// Anatomy of a secondary key it is a `enum` with the following structure: `<table_name>Key::<name>`.
    ///
    /// # Example
    /// ```rust
    /// use native_db::*;
    /// use native_db::native_model::{native_model, Model};
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
    ///     let mut models = Models::new();
    ///     models.define::<Data>()?;
    ///     let db = Builder::new().create_in_memory(&models)?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get only values that have the secondary key name starting with "hello"
    ///     let _values: Vec<Data> = r.scan().secondary(DataKey::name)?.start_with("hello")?.try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with(
        &self,
        start_with: impl ToKey,
    ) -> Result<SecondaryScanIterator<PrimaryTable, T>> {
        check_key_type_from_key_definition(&self.key_def, &start_with)?;
        let start_with = start_with.to_key();
        let mut primary_keys = vec![];
        for keys in self.secondary_table.range::<Key>(start_with.clone()..)? {
            let (l_secondary_key, l_primary_keys) = keys?;
            if !l_secondary_key
                .value()
                .as_slice()
                .starts_with(start_with.as_slice())
            {
                break;
            }
            for primary_key in l_primary_keys {
                let primary_key = primary_key?;
                primary_keys.push(primary_key);
            }
        }

        Ok(SecondaryScanIterator {
            primary_table: &self.primary_table,
            primary_keys: primary_keys.into_iter(),
            _marker: PhantomData,
        })
    }
}

use std::vec::IntoIter;

pub struct SecondaryScanIterator<'a, PrimaryTable, T: ToInput>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    pub(crate) primary_table: &'a PrimaryTable,
    pub(crate) primary_keys: IntoIter<redb::AccessGuard<'a, Key>>,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, T: ToInput> Iterator for SecondaryScanIterator<'_, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.primary_keys.next() {
            Some(primary_key) => {
                if let Ok(value) = self.primary_table.get(primary_key.value()) {
                    unwrap_item(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<PrimaryTable, T: ToInput> DoubleEndedIterator
    for SecondaryScanIterator<'_, PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.primary_keys.next_back() {
            Some(primary_key) => {
                if let Ok(value) = self.primary_table.get(primary_key.value()) {
                    unwrap_item(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
