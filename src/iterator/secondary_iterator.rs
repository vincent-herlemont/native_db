use crate::common::unwrap_item;
use crate::SDBItem;
use redb::ReadableTable;
use std::marker::PhantomData;

/// Same as [`PrimaryIterator`](crate::PrimaryIterator) but only returns values that match the given secondary key.
pub struct SecondaryIterator<
    'a,
    'txn,
    'db,
    T: SDBItem,
    MT: ReadableTable<&'static [u8], &'static [u8]>,
> {
    pub(crate) range: redb::Range<'a, &'static [u8], &'static [u8]>,
    pub(crate) main_table: &'a MT,
    pub(crate) _marker: PhantomData<(&'db (), &'txn (), T)>,
}

impl<'a, 'txn, 'db, T: SDBItem, MT: ReadableTable<&'static [u8], &'static [u8]>> Iterator
    for SecondaryIterator<'a, 'txn, 'db, T, MT>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((_, v))) => {
                let key: Vec<u8> = v.value().into();
                if let Ok(value) = self.main_table.get(&*key) {
                    unwrap_item(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a, 'txn, 'db, T: SDBItem, MT: ReadableTable<&'static [u8], &'static [u8]>> DoubleEndedIterator
    for SecondaryIterator<'a, 'txn, 'db, T, MT>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.next_back() {
            Some(Ok((_, v))) => {
                let key: Vec<u8> = v.value().into();
                unwrap_item(self.main_table.get(&*key).unwrap())
            }
            _ => None,
        }
    }
}
