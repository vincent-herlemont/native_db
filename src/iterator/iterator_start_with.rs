use crate::common::get;
use crate::SDBItem;
use std::marker::PhantomData;

/// Same as [`Iterator`] but only returns values which primary key starts with the given prefix.
pub struct IteratorStartWith<'a, 'txn, 'db, T: SDBItem> {
    pub(crate) range: redb::Range<'a, &'static [u8], &'static [u8]>,
    pub(crate) start_with: &'a [u8],
    pub(crate) _marker: PhantomData<(&'db (), &'txn (), T)>,
}

impl<'a, 'txn, 'db, T: SDBItem> Iterator for IteratorStartWith<'a, 'txn, 'db, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((k, v))) => {
                let k = k.value();
                if k.starts_with(self.start_with) {
                    get(Some(v))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// TODO: Found a way to implement DoubleEndedIterator for StructDBIteratorStartWith
