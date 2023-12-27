use crate::db_type::{unwrap_item, DatabaseInnerKeyValue, DatabaseInnerKeyValueRange, Input};
use crate::InnerKeyValue;
use std::marker::PhantomData;
use std::ops::RangeBounds;

/// Scan values from the database.
pub struct PrimaryScan<PrimaryTable, T: Input>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
{
    pub(crate) primary_table: PrimaryTable,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, T: Input> PrimaryScan<PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<DatabaseInnerKeyValue, &'static [u8]>,
{
    pub(crate) const fn new(table: PrimaryTable) -> Self {
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
    /// use native_model::{native_model, Model};
    /// use serde::{Deserialize, Serialize};
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
    ///     let mut builder = DatabaseBuilder::new();
    ///     builder.define::<Data>()?;
    ///     let db = builder.create_in_memory()?;
    ///     
    ///     // Open a read transaction
    ///     let r = db.r_transaction()?;
    ///     
    ///     // Get all values
    ///     let _values: Vec<Data> = r.scan().primary()?.all().collect();
    ///     Ok(())
    /// }
    /// ```
    pub fn all(&self) -> PrimaryScanIterator<'_, T> {
        let range = self
            .primary_table
            .range::<DatabaseInnerKeyValue>(..)
            .unwrap();
        PrimaryScanIterator {
            range,
            _marker: PhantomData,
        }
    }

    /// Iterate over all values in a range.
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
    ///     // Get the values from 5 to the end
    ///     let _values: Vec<Data> = r.scan().primary()?.range(5u64..).collect();
    ///     Ok(())
    /// }
    /// ```
    pub fn range<TR: InnerKeyValue, R: RangeBounds<TR>>(
        &self,
        range: R,
    ) -> PrimaryScanIterator<'_, T> {
        let database_inner_key_value_range = DatabaseInnerKeyValueRange::new(range);
        let range = self
            .primary_table
            .range::<DatabaseInnerKeyValue>(database_inner_key_value_range)
            .unwrap();
        PrimaryScanIterator {
            range,
            _marker: PhantomData,
        }
    }

    /// Iterate over all values starting with a prefix.
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
    ///     id: String,
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
    ///     // Get the values starting with "victor"
    ///     let _values: Vec<Data> = r.scan().primary()?.start_with("victor").collect();
    ///     Ok(())
    /// }
    /// ```
    pub fn start_with<'a>(
        &'a self,
        start_with: impl InnerKeyValue + 'a,
    ) -> PrimaryScanIteratorStartWith<'a, T> {
        let start_with = start_with.database_inner_key_value();
        let range = self
            .primary_table
            .range::<DatabaseInnerKeyValue>(start_with.clone()..)
            .unwrap();
        PrimaryScanIteratorStartWith {
            start_with,
            range,
            _marker: PhantomData,
        }
    }
}

pub struct PrimaryScanIterator<'a, T: Input> {
    pub(crate) range: redb::Range<'a, DatabaseInnerKeyValue, &'static [u8]>,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T: Input> Iterator for PrimaryScanIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((_, v))) => unwrap_item(Some(v)),
            _ => None,
        }
    }
}
impl<'a, T: Input> DoubleEndedIterator for PrimaryScanIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(Ok((_, v))) => unwrap_item(Some(v)),
            _ => None,
        }
    }
}

pub struct PrimaryScanIteratorStartWith<'a, T: Input> {
    pub(crate) range: redb::Range<'a, DatabaseInnerKeyValue, &'static [u8]>,
    pub(crate) start_with: DatabaseInnerKeyValue,
    pub(crate) _marker: PhantomData<T>,
}

impl<'a, T: Input> Iterator for PrimaryScanIteratorStartWith<'a, T> {
    type Item = T;

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
