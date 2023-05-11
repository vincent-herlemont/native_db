use crate::common::unwrap_item;
use crate::SDBItem;
use std::iter;
use std::marker::PhantomData;

/// Provides a way to iterate over the values stored in a database and
/// automatically deserialize them into items of type `T`.
pub struct PrimaryIterator<'a, 'txn, 'db, T: SDBItem> {
    pub(crate) range: redb::Range<'a, &'static [u8], &'static [u8]>,
    pub(crate) _marker: PhantomData<(&'db (), &'txn (), T)>,
}

impl<'a, 'txn, 'db, T: SDBItem> iter::Iterator for PrimaryIterator<'a, 'txn, 'db, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((_, v))) => unwrap_item(Some(v)),
            _ => None,
        }
    }
}

impl<'a, 'txn, 'db, T: SDBItem> DoubleEndedIterator for PrimaryIterator<'a, 'txn, 'db, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(Ok((_, v))) => unwrap_item(Some(v)),
            _ => None,
        }
    }
}
