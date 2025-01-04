use crate::db_type::{check_key_type, check_range_key_range_bounds, ToKey};
use crate::db_type::{unwrap_item, Key, KeyRange, Result, ToInput};
use std::marker::PhantomData;
use std::ops::RangeBounds;

/// Scan values from the database.
pub struct PrimaryScan<PrimaryTable, T: ToInput>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    pub(crate) primary_table: PrimaryTable,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, T: ToInput> PrimaryScan<PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    pub(crate) fn new(table: PrimaryTable) -> Self {
        Self {
            primary_table: table,
            _marker: PhantomData,
        }
    }

    /// Iterate over all values.
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
    ///     // Get all values
    ///     let _values: Vec<Data> = r.scan().primary()?.all()?.try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self) -> Result<PrimaryScanIterator<T>> {
        let range = self.primary_table.range::<Key>(..)?;
        Ok(PrimaryScanIterator {
            range,
            _marker: PhantomData,
        })
    }

    /// Iterate over all values in a range.
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
    ///     // Get the values from 5 to the end
    ///     let _values: Vec<Data> = r.scan().primary()?.range(5u64..)?.try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn range<R: RangeBounds<impl ToKey>>(&self, range: R) -> Result<PrimaryScanIterator<T>> {
        let model = T::native_db_model();
        check_range_key_range_bounds(&model, &range)?;
        let database_inner_key_value_range = KeyRange::new(range);
        let range = self
            .primary_table
            .range::<Key>(database_inner_key_value_range)?;
        Ok(PrimaryScanIterator {
            range,
            _marker: PhantomData,
        })
    }

    /// Iterate over all values starting with a prefix.
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
    ///     id: String,
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
    ///     // Get the values starting with "victor"
    ///     let _values: Vec<Data> = r.scan().primary()?.start_with("victor")?.try_collect()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with(&self, start_with: impl ToKey) -> Result<PrimaryScanIteratorStartWith<T>> {
        let model = T::native_db_model();
        check_key_type(&model, &start_with)?;
        let start_with = start_with.to_key();
        let range = self.primary_table.range::<Key>(start_with.clone()..)?;

        Ok(PrimaryScanIteratorStartWith {
            range,
            start_with,
            _marker: PhantomData,
        })
    }
}

pub struct PrimaryScanIterator<'a, T: ToInput> {
    pub(crate) range: redb::Range<'a, Key, &'static [u8]>,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T: ToInput> Iterator for PrimaryScanIterator<'a, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((_, v))) => unwrap_item(Some(v)),
            _ => None,
        }
    }
}
impl<'a, T: ToInput> DoubleEndedIterator for PrimaryScanIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(Ok((_, v))) => unwrap_item(Some(v)),
            _ => None,
        }
    }
}

pub struct PrimaryScanIteratorStartWith<'a, T: ToInput> {
    pub(crate) range: redb::Range<'a, Key, &'static [u8]>,
    pub(crate) start_with: Key,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T: ToInput> Iterator for PrimaryScanIteratorStartWith<'a, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((k, v))) => {
                let k = k.value();
                if k.as_slice().starts_with(self.start_with.as_slice()) {
                    unwrap_item(Some(v))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
