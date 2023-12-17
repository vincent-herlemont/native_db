use crate::db_type::{unwrap_item, DatabaseInnerKeyValue, DatabaseInnerKeyValueRange, Input};
use crate::InnerKeyValue;
use std::marker::PhantomData;
use std::ops::RangeBounds;

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
    pub fn new(table: PrimaryTable) -> Self {
        Self {
            primary_table: table,
            _marker: PhantomData::default(),
        }
    }

    pub fn iter(&self) -> PrimaryScanIterator<T> {
        let range = self
            .primary_table
            .range::<DatabaseInnerKeyValue>(..)
            .unwrap();
        PrimaryScanIterator {
            range,
            _marker: PhantomData::default(),
        }
    }

    // pub fn range<RT: InnerKeyValue, R: RangeBounds<RT>>(&self, range: R) -> PrimaryScanIterator<T> {
    pub fn range<TR: InnerKeyValue, R: RangeBounds<TR>>(&self, range: R) -> PrimaryScanIterator<T> {
        let database_inner_key_value_range = DatabaseInnerKeyValueRange::new(range);
        let range = self
            .primary_table
            .range::<DatabaseInnerKeyValue>(database_inner_key_value_range)
            .unwrap();
        PrimaryScanIterator {
            range,
            _marker: PhantomData::default(),
        }
    }

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
            _marker: PhantomData::default(),
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
