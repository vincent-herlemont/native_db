use crate::common::unwrap_item;
use crate::SDBItem;
use redb::ReadableTable;
use std::marker::PhantomData;

/// Same as [`PrimaryIterator`](crate::PrimaryIterator) but only returns values with secondary keys that start with the given
/// prefix.
pub struct SecondaryIteratorStartWith<
    'a,
    'txn,
    'db,
    T: SDBItem,
    MT: ReadableTable<&'static [u8], &'static [u8]>,
> {
    pub(crate) range: redb::Range<'a, &'static [u8], &'static [u8]>,
    pub(crate) start_with: &'a [u8],
    pub(crate) main_table: &'a MT,
    pub(crate) _marker: PhantomData<(&'db (), &'txn (), T)>,
}

impl<'a, 'txn, 'db, T: SDBItem, MT: ReadableTable<&'static [u8], &'static [u8]>> Iterator
    for SecondaryIteratorStartWith<'a, 'txn, 'db, T, MT>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.next() {
            Some(Ok((k, v))) => {
                let k = k.value();
                if k.starts_with(self.start_with) {
                    let key: Vec<u8> = v.value().into();
                    unwrap_item(self.main_table.get(&*key).unwrap())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// TODO: Found a way to implement DoubleEndedIterator for StructDBIteratorStartWithByKey
