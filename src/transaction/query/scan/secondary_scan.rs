use crate::db_type::{unwrap_item, DatabaseInnerKeyValue, DatabaseInnerKeyValueRange, Input};
use crate::InnerKeyValue;
use redb;
use std::marker::PhantomData;
use std::ops::RangeBounds;

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
    pub fn new(primary_table: PrimaryTable, secondary_table: SecondaryTable) -> Self {
        Self {
            primary_table,
            secondary_table,
            _marker: PhantomData::default(),
        }
    }

    pub fn all(&self) -> SecondaryScanIterator<PrimaryTable, T> {
        let range = self
            .secondary_table
            .range::<DatabaseInnerKeyValue>(..)
            .unwrap();
        SecondaryScanIterator {
            primary_table: &self.primary_table,
            range,
            _marker: PhantomData::default(),
        }
    }

    pub fn range<TR: InnerKeyValue, R: RangeBounds<TR>>(
        &self,
        range: R,
    ) -> SecondaryScanIterator<PrimaryTable, T> {
        let database_inner_key_value_range = DatabaseInnerKeyValueRange::new(range);
        let range = self
            .secondary_table
            .range::<DatabaseInnerKeyValue>(database_inner_key_value_range)
            .unwrap();
        SecondaryScanIterator {
            primary_table: &self.primary_table,
            range,
            _marker: PhantomData::default(),
        }
    }

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
            _marker: PhantomData::default(),
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
