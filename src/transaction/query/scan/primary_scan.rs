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
    pub(crate) model: crate::Model,
    pub(crate) observed_at: Option<u64>,
    pub(crate) _marker: PhantomData<T>,
}

impl<PrimaryTable, T: ToInput> PrimaryScan<PrimaryTable, T>
where
    PrimaryTable: redb::ReadableTable<Key, &'static [u8]>,
{
    pub(crate) fn new(table: PrimaryTable, model: crate::Model, observed_at: Option<u64>) -> Self {
        Self {
            primary_table: table,
            model,
            observed_at,
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
    pub fn all(&self) -> Result<PrimaryScanIterator<'_, T>> {
        let range = self.primary_table.range::<Key>(..)?;
        Ok(PrimaryScanIterator {
            range,
            model: self.model.clone(),
            observed_at: self.observed_at,
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
    pub fn range<R: RangeBounds<impl ToKey>>(
        &self,
        range: R,
    ) -> Result<PrimaryScanIterator<'_, T>> {
        let model = T::native_db_model();
        check_range_key_range_bounds(&model, &range)?;
        let database_inner_key_value_range = KeyRange::new(range);
        let range = self
            .primary_table
            .range::<Key>(database_inner_key_value_range)?;
        Ok(PrimaryScanIterator {
            range,
            model: self.model.clone(),
            observed_at: self.observed_at,
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
    pub fn start_with(
        &self,
        start_with: impl ToKey,
    ) -> Result<PrimaryScanIteratorStartWith<'_, T>> {
        let model = T::native_db_model();
        check_key_type(&model, &start_with)?;
        let start_with = start_with.to_key();
        let range = self.primary_table.range::<Key>(start_with.clone()..)?;

        Ok(PrimaryScanIteratorStartWith {
            range,
            start_with,
            model: self.model.clone(),
            observed_at: self.observed_at,
            _marker: PhantomData,
        })
    }
}

pub struct PrimaryScanIterator<'a, T: ToInput> {
    pub(crate) range: redb::Range<'a, Key, &'static [u8]>,
    pub(crate) model: crate::Model,
    pub(crate) observed_at: Option<u64>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: ToInput> Iterator for PrimaryScanIterator<'_, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.range.next() {
                Some(Ok((_, v))) => match unwrap_item::<T>(Some(v)) {
                    Some(Ok(item)) => {
                        if crate::expiry::visible_at(&self.model, &item, self.observed_at) {
                            return Some(Ok(item));
                        }
                    }
                    other => return other,
                },
                _ => return None,
            }
        }
    }
}
impl<T: ToInput> DoubleEndedIterator for PrimaryScanIterator<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.range.next_back() {
                Some(Ok((_, v))) => match unwrap_item::<T>(Some(v)) {
                    Some(Ok(item)) => {
                        if crate::expiry::visible_at(&self.model, &item, self.observed_at) {
                            return Some(Ok(item));
                        }
                    }
                    other => return other,
                },
                _ => return None,
            }
        }
    }
}

pub struct PrimaryScanIteratorStartWith<'a, T: ToInput> {
    pub(crate) range: redb::Range<'a, Key, &'static [u8]>,
    pub(crate) start_with: Key,
    pub(crate) model: crate::Model,
    pub(crate) observed_at: Option<u64>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: ToInput> Iterator for PrimaryScanIteratorStartWith<'_, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.range.next() {
                Some(Ok((k, v))) => {
                    let k = k.value();
                    if !k.as_slice().starts_with(self.start_with.as_slice()) {
                        return None;
                    }
                    match unwrap_item::<T>(Some(v)) {
                        Some(Ok(item)) => {
                            if crate::expiry::visible_at(&self.model, &item, self.observed_at) {
                                return Some(Ok(item));
                            }
                        }
                        other => return other,
                    }
                }
                _ => return None,
            }
        }
    }
}
